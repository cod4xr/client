// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual UDP/process challenge probes against independent source-shaped services.
#![allow(
    clippy::expect_used,
    reason = "Finite loopback services use explicit receive deadlines."
)]
use std::{
    net::UdpSocket,
    process::{Command, Output},
    sync::mpsc,
    thread,
    time::Duration,
};

fn server() -> UdpSocket {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Loopback peer");
    socket
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("Deadline");
    socket
}
fn run(peer: &str, options: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cod4xr-client"))
        .args(["probe-challenge", peer])
        .args(options)
        .env("NO_COLOR", "1")
        .output()
        .expect("Challenge process")
}
fn nonce(request: &[u8]) -> i32 {
    assert!(request.starts_with(b"\xff\xff\xff\xffgetchallenge "));
    std::str::from_utf8(&request[17..])
        .expect("ASCII nonce")
        .parse()
        .expect("Exactly one signed nonce, no credentials")
}
fn response(nonce: i32) -> Vec<u8> {
    let mut packet = vec![0xff; 4];
    packet
        .extend_from_slice(format!("challengeResponse 2147481234 {nonce} 0 xproto 18").as_bytes());
    packet
}
fn report(output: &Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("One JSON report")
}

#[test]
fn correlated_reply_rejects_stale_and_unsupported_packets_without_disclosing_cookies() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer").to_string();
    let responder = thread::spawn(move || {
        let mut request = [0; 128];
        let (length, client) = socket.recv_from(&mut request).expect("Query");
        let nonce = nonce(&request[..length]);
        socket
            .send_to(&response(nonce.wrapping_add(1)), client)
            .expect("Stale echo");
        let mut unsupported = vec![0xff; 4];
        unsupported.extend_from_slice(
            format!("challengeResponse 2147481234 {nonce} 0 xproto 21").as_bytes(),
        );
        socket
            .send_to(&unsupported, client)
            .expect("Unknown profile");
        socket
            .send_to(&response(nonce), client)
            .expect("Correct echo");
    });
    let output = run(&peer, &["--log-format", "json"]);
    responder.join().expect("Responder");
    let report = report(&output);
    assert_eq!(report["phase"], "challenge_probe");
    assert_eq!(report["profile"], "0 xproto 18");
    assert_eq!(report["attempts"], 1);
    assert_eq!(report["received_datagrams"], 3);
    assert_eq!(report["rejected_datagrams"], 2);
    assert_eq!(report["gameplay_admission"], "not_attempted");
    assert_eq!(report["server_identity"], "unverified");
    for bytes in [&output.stdout, &output.stderr] {
        let text = String::from_utf8_lossy(bytes);
        assert!(!text.contains("2147481234"));
        assert!(!text.contains("client_challenge"));
    }
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        let _: serde_json::Value = serde_json::from_str(line).expect("JSON logs only");
    }
}

#[test]
fn dropping_first_request_retries_same_nonce_and_socket_before_accepting() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer").to_string();
    let responder = thread::spawn(move || {
        let mut first = [0; 128];
        let (length, client) = socket.recv_from(&mut first).expect("First attempt");
        let mut second = [0; 128];
        let (next_length, next_client) = socket.recv_from(&mut second).expect("Retried attempt");
        assert_eq!(next_client, client);
        assert_eq!(&first[..length], &second[..next_length]);
        socket
            .send_to(&response(nonce(&second[..next_length])), client)
            .expect("Reply after retry");
    });
    let output = run(
        &peer,
        &[
            "--retry-ms",
            "20",
            "--max-attempts",
            "2",
            "--timeout-ms",
            "1000",
        ],
    );
    responder.join().expect("Responder");
    let report = report(&output);
    assert_eq!(report["attempts"], 2);
    assert_eq!(report["received_datagrams"], 1);
}

#[test]
fn one_attempt_still_accepts_a_reply_after_retry_time_but_before_absolute_expiry() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer").to_string();
    let responder = thread::spawn(move || {
        let mut request = [0; 128];
        let (length, client) = socket.recv_from(&mut request).expect("Only attempt");
        thread::sleep(Duration::from_millis(40));
        socket
            .send_to(&response(nonce(&request[..length])), client)
            .expect("Late valid reply");
    });
    let output = run(
        &peer,
        &[
            "--retry-ms",
            "10",
            "--max-attempts",
            "1",
            "--timeout-ms",
            "1000",
        ],
    );
    responder.join().expect("Responder");
    assert_eq!(report(&output)["attempts"], 1);
}

#[test]
fn correct_nonce_from_a_different_endpoint_is_filtered_before_acceptance() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer").to_string();
    let responder = thread::spawn(move || {
        let mut request = [0; 128];
        let (length, client) = socket.recv_from(&mut request).expect("Query");
        let bytes = response(nonce(&request[..length]));
        server().send_to(&bytes, client).expect("Other endpoint");
        socket.send_to(&bytes, client).expect("Selected endpoint");
    });
    let output = run(&peer, &[]);
    responder.join().expect("Responder");
    let report = report(&output);
    assert_eq!(report["received_datagrams"], 1);
    assert_eq!(report["rejected_datagrams"], 0);
}

#[test]
fn receive_budget_failure_reports_counts_and_exits_without_a_success_report() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer").to_string();
    let responder = thread::spawn(move || {
        let mut request = [0; 128];
        let (_, client) = socket.recv_from(&mut request).expect("Query");
        for _ in 0..2 {
            socket.send_to(b"malformed", client).expect("Noise");
        }
    });
    let output = run(&peer, &["--max-datagrams", "2", "--log-format", "json"]);
    responder.join().expect("Responder");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let logs = String::from_utf8(output.stderr).expect("Logs");
    let records: Vec<serde_json::Value> = logs
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON log"))
        .collect();
    let summary = records
        .iter()
        .find(|record| record["fields"]["event"] == "challenge.summary")
        .expect("Work summary");
    assert_eq!(summary["fields"]["received"], 2);
    assert_eq!(summary["fields"]["rejected"], 2);
    assert!(
        records
            .iter()
            .any(|record| record["fields"]["event"] == "challenge.failed")
    );
}

#[test]
fn unsupported_profile_times_out_without_a_legacy_or_gameplay_fallback() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer").to_string();
    let (finished, completion) = mpsc::sync_channel(0);
    let responder = thread::spawn(move || {
        let mut request = [0; 128];
        let (length, client) = socket.recv_from(&mut request).expect("Query");
        let mut reply = vec![0xff; 4];
        reply.extend_from_slice(
            format!(
                "challengeResponse 1 {} 0 xproto 21",
                nonce(&request[..length])
            )
            .as_bytes(),
        );
        socket.send_to(&reply, client).expect("Unsupported profile");
        completion
            .recv_timeout(Duration::from_secs(10))
            .expect("Client completion");
    });
    let output = run(
        &peer,
        &[
            "--timeout-ms",
            "1000",
            "--max-attempts",
            "1",
            "--log-format",
            "json",
        ],
    );
    finished.send(()).expect("Release responder");
    responder.join().expect("Responder");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let logs = String::from_utf8(output.stderr).expect("Logs");
    assert!(logs.contains("UnsupportedProfile"));
    assert!(logs.contains("Expired"));
}
