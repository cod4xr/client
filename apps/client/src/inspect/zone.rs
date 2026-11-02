// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{InspectError, file};
use cod4xr_content::{
    fastfile5::CheckedFastfile5,
    zone5::{
        PreparedZone5Directory, Zone5AssetSlot, Zone5Budget, Zone5DirectoryMetadata, Zone5Output,
        Zone5SerializedLink, Zone5StringSlot,
    },
};
use sha2::{Digest, Sha256};

pub(super) struct Summary {
    metadata: Zone5DirectoryMetadata,
    kinds: [u64; 33],
    links: [u64; 4],
    string_digest: [u8; 32],
}
pub(super) fn inspect(container: &CheckedFastfile5<'_>) -> Result<Summary, InspectError> {
    // Inspection policy; caller storage uses actual checked counts, never these maxima.
    let budget = Zone5Budget {
        max_script_strings: 1_000_000,
        max_assets: 1_000_000,
        max_string_bytes: 32 * 1024 * 1024,
        max_prefix_bytes: 64 * 1024 * 1024,
    };
    let counts = PreparedZone5Directory::counts(container, budget).map_err(InspectError::Zone)?;
    let mut scratch_strings = Vec::new();
    let mut scratch_assets = Vec::new();
    file::resize(
        &mut scratch_strings,
        counts.script_strings,
        Zone5StringSlot::default(),
    )?;
    file::resize(
        &mut scratch_assets,
        counts.assets,
        Zone5AssetSlot::default(),
    )?;
    let plan =
        PreparedZone5Directory::parse(container, budget, &mut scratch_strings, &mut scratch_assets)
            .map_err(InspectError::Zone)?;
    let metadata = plan.metadata();
    let mut strings = Vec::new();
    let mut assets = Vec::new();
    let mut bytes = Vec::new();
    file::resize(
        &mut strings,
        counts.script_strings,
        Zone5StringSlot::default(),
    )?;
    file::resize(&mut assets, counts.assets, Zone5AssetSlot::default())?;
    file::resize(&mut bytes, metadata.string_bytes, 0)?;
    let directory = plan
        .publish(Zone5Output {
            strings: &mut strings,
            assets: &mut assets,
            bytes: &mut bytes,
        })
        .map_err(InspectError::Zone)?;
    let mut kinds = [0; 33];
    let mut links = [0; 4];
    for asset in directory.assets() {
        kinds[usize::from(asset.kind().code())] += 1;
        links[match asset.serialized_link() {
            Zone5SerializedLink::Null => 0,
            Zone5SerializedLink::Following => 1,
            Zone5SerializedLink::Insert => 2,
            Zone5SerializedLink::Offset { .. } => 3,
        }] += 1;
    }
    let string_digest = Sha256::digest(&bytes).into();
    Ok(Summary {
        metadata,
        kinds,
        links,
        string_digest,
    })
}
impl Summary {
    pub fn report(&self) -> serde_json::Value {
        serde_json::json!({
            "script_strings":self.metadata.counts.script_strings,
            "null_strings":self.metadata.null_strings,"asset_records":self.metadata.counts.assets,
            "inline_string_bytes":self.metadata.string_bytes,"body_prefix_bytes":self.metadata.body_prefix_bytes,
            "virtual_prefix_bytes":self.metadata.virtual_prefix_bytes,"asset_type_counts":self.kinds.as_slice(),
            "asset_link_counts":{"null":self.links[0],"following":self.links[1],"insert":self.links[2],"offset":self.links[3]},
            "script_bytes_sha256":self.string_digest
        })
    }
}
