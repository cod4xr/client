// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_client_core::{
    commands::LegacyClientCommands,
    server_commands::ServerCommandHistory,
    service_controls::{LegacyServicePlan, ServiceControlBudget},
};
use cod4xr_client_net::{ChannelEvent, ClientChannel};
use std::time::{Duration, Instant};

pub(super) fn admit(
    channel: &mut ClientChannel<'_>,
    outgoing: &mut LegacyClientCommands<'_>,
    incoming: &mut ServerCommandHistory<'_>,
) {
    let payload_codec = channel
        .server_decoder()
        .expect("Original established decoder");
    let mut text = [0xa5; 128];
    let controls = channel
        .split()
        .receiver
        .receive_until(Instant::now() + Duration::from_secs(3), |event| {
            let ChannelEvent::Message(message) = event else {
                return None;
            };
            let mut work = [0; 2048];
            let mut decoded = [0; 2048];
            let payload = payload_codec
                .decode(
                    message.sequence(),
                    message.body(),
                    outgoing,
                    &mut work,
                    &mut decoded,
                )
                .expect("Original outgoing key");
            Some(LegacyServicePlan::parse(
                &payload,
                ServiceControlBudget::default(),
                &mut [0; 128],
                &mut text,
            ))
        })
        .expect("Receive")
        .expect("Before deadline")
        .expect("Ordinary body")
        .expect("Complete controls through EOF");
    let prepared = incoming
        .prepare(&controls)
        .expect("Incoming resource preflight");
    outgoing
        .commit_acknowledgement(controls.client_command_acknowledge())
        .expect("Checked original ACK before infallible incoming commit");
    let _ = prepared.commit();
}
