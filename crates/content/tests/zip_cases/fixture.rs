// SPDX-License-Identifier: AGPL-3.0-or-later
pub(super) struct Spec<'a> {
    pub name: &'a [u8],
    pub packed: &'a [u8],
    pub unpacked: u32,
    pub crc: u32,
    pub flags: u16,
    pub method: u16,
    pub extra: &'a [u8],
    pub descriptor_signature: bool,
}
impl<'a> Spec<'a> {
    pub fn stored(name: &'a [u8], payload: &'a [u8]) -> Self {
        Self {
            name,
            packed: payload,
            unpacked: u32::try_from(payload.len()).expect("Small fixture"),
            crc: 0x1234_5678,
            flags: 0,
            method: 0,
            extra: &[],
            descriptor_signature: false,
        }
    }
}
#[allow(
    dead_code,
    reason = "Routing fields serve different shared integration-test binaries."
)]
pub(super) struct Fixture {
    pub bytes: Vec<u8>,
    pub locals: Vec<usize>,
    pub central: Vec<usize>,
    pub end: usize,
}
pub(super) fn short(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
pub(super) fn word(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
fn narrow(value: usize) -> u16 {
    u16::try_from(value).expect("Small fixture")
}
fn wide(value: usize) -> u32 {
    u32::try_from(value).expect("Small fixture")
}

pub(super) fn build(specs: &[Spec<'_>], order: &[usize], comment: &[u8]) -> Fixture {
    let mut bytes = Vec::new();
    let mut locals = Vec::new();
    for spec in specs {
        let start = bytes.len();
        locals.push(start);
        bytes.resize(start + 30, 0);
        bytes[start..start + 4].copy_from_slice(b"PK\x03\x04");
        short(&mut bytes, start + 4, 20);
        short(&mut bytes, start + 6, spec.flags);
        short(&mut bytes, start + 8, spec.method);
        if spec.flags & 8 == 0 {
            word(&mut bytes, start + 14, spec.crc);
            word(&mut bytes, start + 18, wide(spec.packed.len()));
            word(&mut bytes, start + 22, spec.unpacked);
        }
        short(&mut bytes, start + 26, narrow(spec.name.len()));
        short(&mut bytes, start + 28, narrow(spec.extra.len()));
        bytes.extend_from_slice(spec.name);
        bytes.extend_from_slice(spec.extra);
        bytes.extend_from_slice(spec.packed);
        if spec.flags & 8 != 0 {
            if spec.descriptor_signature {
                bytes.extend_from_slice(b"PK\x07\x08");
            }
            bytes.extend_from_slice(&spec.crc.to_le_bytes());
            bytes.extend_from_slice(&wide(spec.packed.len()).to_le_bytes());
            bytes.extend_from_slice(&spec.unpacked.to_le_bytes());
        }
    }
    let directory = bytes.len();
    let mut central = Vec::new();
    let natural: Vec<_> = (0..specs.len()).collect();
    let order = if order.is_empty() { &natural } else { order };
    for &index in order {
        let spec = &specs[index];
        let start = bytes.len();
        central.push(start);
        bytes.resize(start + 46, 0);
        bytes[start..start + 4].copy_from_slice(b"PK\x01\x02");
        short(&mut bytes, start + 4, 20);
        short(&mut bytes, start + 6, 20);
        short(&mut bytes, start + 8, spec.flags);
        short(&mut bytes, start + 10, spec.method);
        word(&mut bytes, start + 16, spec.crc);
        word(&mut bytes, start + 20, wide(spec.packed.len()));
        word(&mut bytes, start + 24, spec.unpacked);
        short(&mut bytes, start + 28, narrow(spec.name.len()));
        short(&mut bytes, start + 30, narrow(spec.extra.len()));
        word(&mut bytes, start + 42, wide(locals[index]));
        bytes.extend_from_slice(spec.name);
        bytes.extend_from_slice(spec.extra);
    }
    let end = bytes.len();
    bytes.resize(end + 22, 0);
    bytes[end..end + 4].copy_from_slice(b"PK\x05\x06");
    short(&mut bytes, end + 8, narrow(order.len()));
    short(&mut bytes, end + 10, narrow(order.len()));
    word(&mut bytes, end + 12, wide(end - directory));
    word(&mut bytes, end + 16, wide(directory));
    short(&mut bytes, end + 20, narrow(comment.len()));
    bytes.extend_from_slice(comment);
    Fixture {
        bytes,
        locals,
        central,
        end,
    }
}
