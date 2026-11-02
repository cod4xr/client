// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Iwi6WaveletBudget, Iwi6WaveletRequirements, WaveletError};
use crate::iwi6::{Iwi6Format, Iwi6Header, Iwi6Shape, mip::MipSlot};

#[derive(Clone, Copy, Debug)]
pub(super) struct Layout {
    pub header: Iwi6Header,
    pub format: Iwi6Format,
    pub channels: usize,
    pub stride: usize,
    pub levels: [MipSlot; 16],
    pub required: Iwi6WaveletRequirements,
}
impl Layout {
    pub fn parse(input: &[u8], budget: Iwi6WaveletBudget) -> Result<Self, WaveletError> {
        let header = Iwi6Header::parse(input, budget.image).map_err(WaveletError::Header)?;
        let (format, channels, stride) = match header.format() {
            Iwi6Format::WaveletRgba => (Iwi6Format::Bgra8, 4, 4),
            Iwi6Format::WaveletRgb => (Iwi6Format::Bgra8, 3, 4),
            Iwi6Format::WaveletLuminanceAlpha => (Iwi6Format::LuminanceAlpha8, 2, 2),
            Iwi6Format::WaveletLuminance => (Iwi6Format::Luminance8, 1, 1),
            Iwi6Format::WaveletAlpha => (Iwi6Format::Alpha8, 1, 1),
            _ => return Err(WaveletError::Format),
        };
        let size = header.dimensions();
        if header.shape() == Iwi6Shape::Volume3D
            || !size.width.is_power_of_two()
            || !size.height.is_power_of_two()
            || (header.mip_levels() == 1 && size.width > 1 && size.height > 1)
        {
            return Err(WaveletError::Geometry);
        }
        let mut levels = [MipSlot::default(); 16];
        let mut decoded_bytes = 0_usize;
        let mut delta_bytes = 0;
        for level in (0..header.mip_levels()).rev() {
            let dimensions = size.mip(u32::try_from(level).map_err(|_| WaveletError::Geometry)?);
            let face_bytes = u64::from(dimensions.width)
                .checked_mul(u64::from(dimensions.height))
                .and_then(|n| n.checked_mul(stride as u64))
                .and_then(|n| usize::try_from(n).ok())
                .ok_or(WaveletError::DecodedBudget)?;
            levels[level] = MipSlot {
                offset: decoded_bytes,
                face_bytes,
                dimensions,
            };
            decoded_bytes = face_bytes
                .checked_mul(usize::from(header.shape().face_count()))
                .and_then(|n| decoded_bytes.checked_add(n))
                .filter(|&n| n <= budget.max_decoded_bytes)
                .ok_or(WaveletError::DecodedBudget)?;
            if dimensions.width > 1 && dimensions.height > 1 {
                delta_bytes = delta_bytes.max(face_bytes / 4);
            }
        }
        let scratch_bytes = decoded_bytes
            .checked_add(delta_bytes)
            .filter(|&n| n <= budget.max_scratch_bytes)
            .ok_or(WaveletError::ScratchBudget)?;
        Ok(Self {
            header,
            format,
            channels,
            stride,
            levels,
            required: Iwi6WaveletRequirements {
                decoded_bytes,
                scratch_bytes,
            },
        })
    }
}
