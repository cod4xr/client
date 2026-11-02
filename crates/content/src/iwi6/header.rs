// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Iwi6Budget, Iwi6Error, Iwi6Format, read};

/// Effective logical dimensions, widened from the version-6 stored fields.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Iwi6Dimensions {
    /// Logical width in texels.
    pub width: u32,
    /// Logical height in texels.
    pub height: u32,
    /// Volume depth, or one for ordinary/cube faces.
    pub depth: u32,
}
impl Iwi6Dimensions {
    pub(super) fn mip(self, level: u32) -> Self {
        Self {
            width: self.width.checked_shr(level).unwrap_or(0).max(1),
            height: self.height.checked_shr(level).unwrap_or(0).max(1),
            depth: self.depth.checked_shr(level).unwrap_or(0).max(1),
        }
    }
    fn texels(self) -> Option<u64> {
        u64::from(self.width)
            .checked_mul(u64::from(self.height))?
            .checked_mul(u64::from(self.depth))
    }
}
/// Stored image topology, without claiming backend upload support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Iwi6Shape {
    /// One two-dimensional face.
    Image2D,
    /// One three-dimensional volume.
    Volume3D,
    /// Six contiguous faces per mip; axis interpretation belongs to rendering.
    Cube,
}
impl Iwi6Shape {
    /// Returns the number of faces encoded for each mip.
    #[must_use]
    pub const fn face_count(self) -> u8 {
        if matches!(self, Self::Cube) { 6 } else { 1 }
    }
}
/// Fully checked header/geometry metadata, not proof of complete pixel data.
/// Includes known wavelet formats without treating them as direct bitmap/BC bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Iwi6Header {
    format: Iwi6Format,
    flags: u8,
    stored: Iwi6Dimensions,
    dimensions: Iwi6Dimensions,
    shape: Iwi6Shape,
    mip_levels: usize,
    texels: u64,
    pub(super) picmip_sizes: [u32; 4],
}
impl Iwi6Header {
    /// Checks the explicit PC version-6 header and complete interpreted geometry.
    /// Effective depth is one for 2D/cube images; unused stored depth remains metadata.
    /// Cube takes precedence over the volume bit, following the reviewed loader.
    /// No flags execute, allocate renderer resources or select runtime capabilities.
    ///
    /// # Errors
    /// Rejects byte/texel/mip budgets, incomplete/mismatched headers, unknown encoding
    /// and zero interpreted dimensions. Pixel body validation is a separate operation.
    pub fn parse(input: &[u8], budget: Iwi6Budget) -> Result<Self, Iwi6Error> {
        if input.len() > budget.max_file_bytes {
            return Err(Iwi6Error::FileBudget);
        }
        let bytes = read::span(input, 0, 28)?;
        if &bytes[..3] != b"IWi" || bytes[3] != 6 {
            return Err(Iwi6Error::Profile);
        }
        let format = Iwi6Format::parse(bytes[4])?;
        let flags = bytes[5];
        let stored = Iwi6Dimensions {
            width: u32::from(read::short(bytes, 6)?),
            height: u32::from(read::short(bytes, 8)?),
            depth: u32::from(read::short(bytes, 10)?),
        };
        let shape = if flags & 4 != 0 {
            Iwi6Shape::Cube
        } else if flags & 8 != 0 {
            Iwi6Shape::Volume3D
        } else {
            Iwi6Shape::Image2D
        };
        let dimensions = Iwi6Dimensions {
            depth: if shape == Iwi6Shape::Volume3D {
                stored.depth
            } else {
                1
            },
            ..stored
        };
        if dimensions.width == 0 || dimensions.height == 0 || dimensions.depth == 0 {
            return Err(Iwi6Error::Dimensions);
        }
        let largest = dimensions
            .width
            .max(dimensions.height)
            .max(dimensions.depth);
        let mip_levels = if flags & 2 != 0 {
            1
        } else {
            usize::try_from(u32::BITS - largest.leading_zeros()).map_err(|_| Iwi6Error::Extent)?
        };
        if mip_levels > budget.max_mip_levels {
            return Err(Iwi6Error::MipBudget);
        }
        let mut texels = 0_u64;
        for level in 0..mip_levels {
            let level = u32::try_from(level).map_err(|_| Iwi6Error::Extent)?;
            texels = dimensions
                .mip(level)
                .texels()
                .and_then(|count| count.checked_mul(u64::from(shape.face_count())))
                .and_then(|count| texels.checked_add(count))
                .filter(|&count| count <= budget.max_texels)
                .ok_or(Iwi6Error::TexelBudget)?;
        }
        Ok(Self {
            format,
            flags,
            stored,
            dimensions,
            shape,
            mip_levels,
            texels,
            picmip_sizes: [
                read::word(bytes, 12)?,
                read::word(bytes, 16)?,
                read::word(bytes, 20)?,
                read::word(bytes, 24)?,
            ],
        })
    }
    /// Returns the explicit encoding, including known wavelet variants.
    #[must_use]
    pub fn format(&self) -> Iwi6Format {
        self.format
    }
    /// Returns original eight-bit format flags as inert metadata.
    #[must_use]
    pub fn flags(&self) -> u8 {
        self.flags
    }
    /// Returns all original dimensions, including inactive stored depth.
    #[must_use]
    pub fn stored_dimensions(&self) -> Iwi6Dimensions {
        self.stored
    }
    /// Returns effective base dimensions used for geometry and byte extents.
    #[must_use]
    pub fn dimensions(&self) -> Iwi6Dimensions {
        self.dimensions
    }
    /// Returns interpreted topology, without renderer resource admission.
    #[must_use]
    pub fn shape(&self) -> Iwi6Shape {
        self.shape
    }
    /// Returns interpreted mip count; the no-picmip flag does not disable mipmaps.
    #[must_use]
    pub fn mip_levels(&self) -> usize {
        self.mip_levels
    }
    /// Returns total logical texels across interpreted mips/faces/slices.
    #[must_use]
    pub fn texels(&self) -> u64 {
        self.texels
    }
}
