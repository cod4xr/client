// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{InspectError, file};
use cod4xr_content::iwi6::{
    Iwi6Budget, Iwi6DirectImage, Iwi6Error, Iwi6Header, Iwi6WaveletBudget, Iwi6WaveletImage,
};
use sha2::{Digest, Sha256};

#[derive(Default)]
pub(super) struct Workspace {
    scratch: Vec<u8>,
    output: Vec<u8>,
    digest: Sha256,
}

#[derive(Clone, Copy, Default)]
pub(super) struct Summary {
    pub headers: usize,
    pub direct_layouts: usize,
    pub wavelet_headers: usize,
    pub wavelet_decoded: usize,
    pub wavelet_decoded_bytes: u64,
    pub wavelet_pixel_sha256: Option<[u8; 32]>,
    pub wavelet_consumed_bits: u64,
    pub wavelet_encoded_bytes: u64,
    pub mip_levels: u64,
    pub texels: u64,
    pub largest_dimension: u32,
}
impl Summary {
    pub fn inspect(&mut self, bytes: &[u8], work: &mut Workspace) -> Result<(), InspectError> {
        let budget = Iwi6Budget {
            max_file_bytes: 64 * 1024 * 1024,
            max_texels: 256 * 1024 * 1024,
            max_mip_levels: 16,
        };
        let header = Iwi6Header::parse(bytes, budget).map_err(InspectError::Image)?;
        if header.format().is_wavelet() {
            self.wavelet_headers += 1;
            let budget = Iwi6WaveletBudget {
                image: budget,
                max_decoded_bytes: 256 * 1024 * 1024,
                max_scratch_bytes: 320 * 1024 * 1024,
            };
            let required =
                Iwi6WaveletImage::requirements(bytes, budget).map_err(InspectError::Wavelet)?;
            file::resize(&mut work.scratch, required.scratch_bytes, 0)?;
            file::resize(&mut work.output, required.decoded_bytes, 0)?;
            let image =
                Iwi6WaveletImage::decode(bytes, budget, &mut work.scratch, &mut work.output)
                    .map_err(InspectError::Wavelet)?;
            for level in (0..header.mip_levels()).rev() {
                let mip = image
                    .mip(level)
                    .ok_or(InspectError::Image(Iwi6Error::Extent))?;
                work.digest.update(mip.bytes());
            }
            self.wavelet_pixel_sha256 = Some(work.digest.clone().finalize().into());
            self.wavelet_decoded += 1;
            self.wavelet_decoded_bytes = add(self.wavelet_decoded_bytes, image.decoded_bytes())?;
            self.wavelet_encoded_bytes = add(self.wavelet_encoded_bytes, image.encoded_bytes())?;
            let (whole, bit) = image.consumed();
            let consumed = u64::try_from(whole)
                .ok()
                .and_then(|n| n.checked_mul(8))
                .and_then(|n| n.checked_add(u64::from(bit)))
                .ok_or(InspectError::Image(Iwi6Error::Extent))?;
            self.wavelet_consumed_bits = self
                .wavelet_consumed_bits
                .checked_add(consumed)
                .ok_or(InspectError::Image(Iwi6Error::Extent))?;
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
fn add(current: u64, value: usize) -> Result<u64, InspectError> {
    u64::try_from(value)
        .ok()
        .and_then(|n| current.checked_add(n))
        .ok_or(InspectError::Image(Iwi6Error::Extent))
}
