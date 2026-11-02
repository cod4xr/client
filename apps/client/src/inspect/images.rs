// SPDX-License-Identifier: AGPL-3.0-or-later
use super::InspectError;
use cod4xr_content::iwi6::{Iwi6Budget, Iwi6DirectImage, Iwi6Error, Iwi6Header};

#[derive(Clone, Copy, Default)]
pub(super) struct Summary {
    pub headers: usize,
    pub direct_layouts: usize,
    pub wavelet_headers: usize,
    pub mip_levels: u64,
    pub texels: u64,
    pub largest_dimension: u32,
}
impl Summary {
    pub fn inspect(&mut self, bytes: &[u8]) -> Result<(), InspectError> {
        let budget = Iwi6Budget {
            max_file_bytes: 64 * 1024 * 1024,
            max_texels: 256 * 1024 * 1024,
            max_mip_levels: 16,
        };
        let header = Iwi6Header::parse(bytes, budget).map_err(InspectError::Image)?;
        if header.format().is_wavelet() {
            self.wavelet_headers += 1;
        } else {
            Iwi6DirectImage::parse(bytes, budget).map_err(InspectError::Image)?;
            self.direct_layouts += 1;
        }
        self.headers += 1;
        self.mip_levels = self
            .mip_levels
            .checked_add(
                u64::try_from(header.mip_levels())
                    .map_err(|_| InspectError::Image(Iwi6Error::Extent))?,
            )
            .ok_or(InspectError::Image(Iwi6Error::Extent))?;
        self.texels = self
            .texels
            .checked_add(header.texels())
            .ok_or(InspectError::Image(Iwi6Error::Extent))?;
        let dims = header.dimensions();
        self.largest_dimension = self
            .largest_dimension
            .max(dims.width)
            .max(dims.height)
            .max(dims.depth);
        Ok(())
    }
}
