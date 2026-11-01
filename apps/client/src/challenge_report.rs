// SPDX-License-Identifier: AGPL-3.0-or-later
use std::io::{self, Write};

use cod4xr_client_core::handshake::ChallengeReceipt;

use crate::error::ClientError;

pub(crate) fn write(receipt: &ChallengeReceipt) -> Result<(), ClientError> {
    // The cookie/nonce never enter reports or operational logs.
    let usage = receipt.usage();
    let report = serde_json::json!({
        "project": "CoD4XR",
        "phase": "challenge_probe",
        "profile": "0 xproto 18",
        "peer": receipt.peer().to_string(),
        "elapsed_micros": receipt.elapsed().as_micros(),
        "attempts": usage.attempts,
        "received_datagrams": usage.received,
        "rejected_datagrams": usage.rejected,
        "client_authentication": "not_used",
        "gameplay_admission": "not_attempted",
        "server_identity": "unverified"
    });
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &report).map_err(ClientError::Output)?;
    writeln!(stdout)?;
    Ok(())
}
