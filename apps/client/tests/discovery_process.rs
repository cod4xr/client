// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real process/UDP tests against independent source-shaped responder services.
#![allow(
    clippy::expect_used,
    reason = "Finite loopback fixtures use explicit bounded socket deadlines."
)]
use std::{
    net::UdpSocket,
    process::{Command, Output},
    thread,
    time::{Duration, Instant},
};

fn client(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cod4xr-client"))
        .args(arguments)
        .env("NO_COLOR", "1")
        .output()
        .expect("Client process")
}

fn server() -> UdpSocket {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Loopback responder");
    socket
        .set_read_timeout(Some(Duration::from_secs(4)))
        .expect("Finite responder wait");
    socket
}

fn response(challenge: &[u8]) -> Vec<u8> {
    let mut response = b"\xff\xff\xff\xffinfoResponse\n\\challenge\\".to_vec();
    response.extend_from_slice(challenge);
    response.extend_from_slice(b"\\hostname\\PrivateRemoteName\\protocol\\6\\mapname\\mp_crash");
    response
}

#[test]
fn real_udp_query_rejects_stale_noise_and_prints_metadata_separately_from_json_logs() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer").to_string();
    let responder = thread::spawn(move || {
        let mut request = [0; 128];
        let (length, client) = socket.recv_from(&mut request).expect("Actual query");
        assert_eq!(length, 44);
        assert_eq!(&request[..12], b"\xff\xff\xff\xffgetinfo ");
        assert!(request[12..length].iter().all(u8::is_ascii_hexdigit));
        socket.send_to(b"unrelated", client).expect("Noise");
        socket
            .send_to(&response(b"stale"), client)
            .expect("Wrong challenge");
        socket
            .send_to(&response(&request[12..length]), client)
            .expect("Correlated reply");
    });
    let output = client(&["query", &peer, "--log-format", "json"]);
    responder.join().expect("Responder");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("One JSON report");
    assert_eq!(report["project"], "CoD4XR");
    assert_eq!(report["hostname"], "PrivateRemoteName");
    assert_eq!(report["advertised_protocol"], 6);
    assert_eq!(report["gameplay_compatibility"], "unverified");
    assert_eq!(report["received_datagrams"], 3);
    assert_eq!(report["rejected_datagrams"], 2);
    let logs = String::from_utf8(output.stderr).expect("UTF-8 log records");
    assert!(!logs.contains("PrivateRemoteName"));
    let records: Vec<serde_json::Value> = logs
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON log"))
        .collect();
    assert!(
        records
            .iter()
            .any(|record| record["fields"]["event"] == "query.completed")
    );
    assert!(
        records
            .iter()
            .any(|record| record["fields"]["event"] == "client.stopped")
    );
}

#[test]
fn connected_socket_ignores_a_correct_token_from_another_endpoint() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer").to_string();
    let responder = thread::spawn(move || {
        let mut request = [0; 128];
        let (length, client) = socket.recv_from(&mut request).expect("Query");
        let intruder = server();
        intruder
            .send_to(&response(&request[12..length]), client)
            .expect("Wrong endpoint");
        socket
            .send_to(&response(&request[12..length]), client)
            .expect("Right endpoint");
    });
    let output = client(&["query", &peer, "--log-format", "human"]);
    responder.join().expect("Responder");
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("Report");
    assert_eq!(report["received_datagrams"], 1);
    assert_eq!(report["rejected_datagrams"], 0);
    let logs = String::from_utf8(output.stderr).expect("Human logs");
    assert!(logs.contains("Server discovery completed"));
    assert!(!logs.contains('\u{1b}'));
}

#[test]
fn receive_budget_terminates_a_stream_of_invalid_replies() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer").to_string();
    let responder = thread::spawn(move || {
        let mut request = [0; 128];
        let (_, client) = socket.recv_from(&mut request).expect("Query");
        for _ in 0..2 {
            socket.send_to(b"malformed", client).expect("Invalid reply");
        }
    });
    let output = client(&["query", &peer, "--max-datagrams", "2"]);
    responder.join().expect("Responder");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("receive budget exhausted"));
}

#[test]
fn oversized_udp_response_is_rejected_without_accepting_its_valid_prefix() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer").to_string();
    let responder = thread::spawn(move || {
        let mut request = [0; 128];
        let (length, client) = socket.recv_from(&mut request).expect("Query");
        let valid = response(&request[12..length]);
        let mut oversized = valid.clone();
        oversized.extend_from_slice(b"\\padding\\");
        oversized.resize(8000, b'x');
        socket.send_to(&oversized, client).expect("Oversized UDP");
        socket.send_to(&valid, client).expect("Bounded response");
    });
    let output = client(&["query", &peer]);
    responder.join().expect("Responder");
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("Report");
    assert_eq!(report["received_datagrams"], 2);
    assert_eq!(report["rejected_datagrams"], 1);
}

#[test]
fn absolute_query_timeout_does_not_wait_forever_on_a_silent_server() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer").to_string();
    let started = Instant::now();
    let output = client(&["query", &peer, "--timeout-ms", "40"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("timed out"));
    assert!(started.elapsed() < Duration::from_secs(4));
}

#[test]
fn initialized_json_failure_logging_contains_only_json_and_retains_failure_exit_status() {
    let socket = server();
    let peer = socket.local_addr().expect("Silent server").to_string();
    let output = client(&["query", &peer, "--timeout-ms", "20", "--log-format", "json"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let logs = String::from_utf8(output.stderr).expect("UTF-8 JSON records");
    let records: Vec<serde_json::Value> = logs
        .lines()
        .map(|line| serde_json::from_str(line).expect("Every failure record is JSON"))
        .collect();
    assert!(
        records
            .iter()
            .any(|record| record["fields"]["event"] == "query.failed")
    );
    assert!(
        records
            .iter()
            .any(|record| record["fields"]["event"] == "client.stopped")
    );
}

#[test]
fn invalid_options_fail_before_network_work_and_help_states_current_scope() {
    for arguments in [
        vec!["connect", "127.0.0.1:28960"],
        vec!["query", "127.0.0.1:0"],
        vec!["query", "0.0.0.0:28960"],
        vec!["query", "224.0.0.1:28960"],
        vec!["query", "127.0.0.1:28960", "--timeout-ms", "0"],
        vec!["query", "127.0.0.1:28960", "--timeout-ms", "30001"],
        vec!["query", "127.0.0.1:28960", "--max-datagrams", "4097"],
        vec!["query", "127.0.0.1:28960", "--log-format", "xml"],
        vec![
            "query",
            "127.0.0.1:28960",
            "--timeout-ms",
            "1",
            "--timeout-ms",
            "2",
        ],
    ] {
        let output = client(&arguments);
        assert!(!output.status.success(), "{arguments:?}");
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("client.started"));
    }
    let help = client(&["--help"]);
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).expect("Help");
    assert!(text.contains("CoD4XR"));
    assert!(text.contains("gameplay/rendering are not implemented"));
}
