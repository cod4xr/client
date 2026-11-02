// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::quiet_peer;
use super::Buffers;
use crate::support::server;
use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4xr_client_net::NetworkError;
use cod4xr_reliable::{
    endpoint::{EndpointConfig, EndpointError},
    receive::ReceiveError,
    schedule::{ScheduleConfig, ScheduleError},
};
use std::time::Duration;

#[test]
fn unsupported_original_profiles_fail_before_resetting_any_supplied_endpoint_storage() {
    for protocol in [6, 18, 42] {
        let socket = server();
        let address = socket.local_addr().expect("Peer");
        let (responder, finished) = quiet_peer(socket);
        let mut buffers = Buffers::new();
        buffers.prime();
        let before = buffers.clone();
        assert!(matches!(
            buffers.connect(address, protocol, EndpointConfig::default()),
            Err(NetworkError::AuxiliaryProfile)
        ));
        assert!(buffers == before);
        finished.send(()).expect("Release peer");
        responder.join().expect("No auxiliary I/O");
    }
}

#[test]
fn late_channel_endpoint_and_timer_validation_errors_preserve_live_buffer_contents() {
    for failure in 0..3 {
        let socket = server();
        let address = socket.local_addr().expect("Peer");
        let (responder, finished) = quiet_peer(socket);
        let mut buffers = Buffers::new();
        buffers.prime();
        let mut config = EndpointConfig::default();
        match failure {
            0 => buffers.ordinary.resize(MAX_MESSAGE_BYTES + 1, 0xa5),
            1 => {
                let _ = buffers.incoming.pop();
            }
            _ => {
                config.schedule = ScheduleConfig {
                    data_interval: Duration::ZERO,
                    ..ScheduleConfig::default()
                }
            }
        }
        let before = buffers.clone();
        let error = buffers
            .connect(address, 21, config)
            .expect_err("Invalid resource policy");
        match failure {
            0 => assert!(matches!(error, NetworkError::Channel(_))),
            1 => assert!(matches!(
                error,
                NetworkError::Endpoint(EndpointError::Receive(ReceiveError::Storage))
            )),
            _ => assert!(matches!(
                error,
                NetworkError::Endpoint(EndpointError::Schedule(ScheduleError::Configuration))
            )),
        }
        assert!(buffers == before);
        finished.send(()).expect("Release peer");
        responder.join().expect("No auxiliary I/O");
    }
}
