// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Pending state fixtures own private fields built from a checked receipt."
)]
use cod4x_protocol::{InfoLimit, InfoString};
use cod4xr_client_core::connect::{NoAuthConnectPlan, NoAuthConnectSettings};

pub fn plan() -> NoAuthConnectPlan {
    NoAuthConnectPlan::new(
        super::connect_support::receipt(i32::MIN),
        NoAuthConnectSettings {
            protocol: 21,
            qport: 40_000,
            version: b"21.8",
            userinfo: InfoString::parse(
                b"\\name\\PrivatePlayer\\password\\PrivatePassword",
                InfoLimit::Normal,
            )
            .expect("Caller fields"),
        },
    )
    .expect("Owned private connect request")
}
