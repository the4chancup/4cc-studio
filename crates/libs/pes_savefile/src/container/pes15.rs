//! The PES 15 container: a keyless chunk format. One seed byte, three MD5 digests
//! (description, logo, payload), the fixed 384-byte description, then `u32 length +
//! data` for the logo and the payload. Every section is XORed with the same
//! byte-wise stream restarted at the seed: `c = (c * 21 + 7) % 32768`, byte key
//! `c % 255`.

use md5::{Digest, Md5};

use super::{ContainerError, SaveContainer, Scheme};

/// Seed byte + three 16-byte digests + the 384-byte description.
const HEAD_SIZE: usize = 1 + 48 + 384;
/// The description's fixed byte count.
const DESCRIPTION_SIZE: usize = 384;

/// The LCG stream XORed over `data`, restarted at `seed` for every section. The same
/// routine encrypts and decrypts.
pub(crate) fn lcg(seed: u8, data: &[u8]) -> Vec<u8> {
    let mut c = u32::from(seed);
    data.iter()
        .map(|byte| {
            c = (c * 21 + 7) % 32768;
            byte ^ (c % 255) as u8
        })
        .collect()
}

/// A 16-byte MD5 digest of `data`.
fn digest(data: &[u8]) -> [u8; 16] {
    Md5::digest(data).into()
}

/// `bytes` decrypted as a PES 15 save. The description's digest identifies the
/// format (a mismatch is `Unrecognized`); the declared lengths must then land
/// exactly on the end of the input and the remaining two digests must match.
pub(crate) fn decrypt(bytes: &[u8]) -> Result<SaveContainer, ContainerError> {
    // Without the description the format cannot be identified at all.
    if bytes.len() < HEAD_SIZE {
        return Err(ContainerError::Unrecognized);
    }
    let seed = bytes[0];
    let digests = &bytes[1..49];
    let description = lcg(seed, &bytes[49..49 + DESCRIPTION_SIZE]);
    if digest(&description) != digests[..16] {
        return Err(ContainerError::Unrecognized);
    }
    let mut offset = HEAD_SIZE;
    let length = |bytes: &[u8], offset: &mut usize| -> Result<usize, ContainerError> {
        let needed = *offset + 4;
        if bytes.len() < needed {
            return Err(ContainerError::Truncated {
                needed,
                available: bytes.len(),
            });
        }
        let len =
            u32::from_le_bytes(bytes[*offset..*offset + 4].try_into().expect("4 bytes")) as usize;
        *offset = needed;
        Ok(len)
    };
    let logo_len = length(bytes, &mut offset)?;
    let logo_end = offset.checked_add(logo_len);
    let payload_len_at = logo_end.and_then(|end| end.checked_add(4));
    let Some(payload_len_at) = payload_len_at else {
        return Err(ContainerError::Truncated {
            needed: usize::MAX,
            available: bytes.len(),
        });
    };
    if bytes.len() < payload_len_at {
        return Err(ContainerError::Truncated {
            needed: payload_len_at,
            available: bytes.len(),
        });
    }
    let logo = lcg(seed, &bytes[offset..offset + logo_len]);
    offset += logo_len;
    let payload_len = length(bytes, &mut offset)?;
    let Some(end) = offset.checked_add(payload_len) else {
        return Err(ContainerError::Truncated {
            needed: usize::MAX,
            available: bytes.len(),
        });
    };
    if bytes.len() < end {
        return Err(ContainerError::Truncated {
            needed: end,
            available: bytes.len(),
        });
    }
    if bytes.len() > end {
        return Err(ContainerError::TrailingBytes(bytes.len() - end));
    }
    let payload = lcg(seed, &bytes[offset..end]);
    if digest(&logo) != digests[16..32] || digest(&payload) != digests[32..48] {
        return Err(ContainerError::Unrecognized);
    }
    Ok(SaveContainer {
        scheme: Scheme::Pes15,
        description,
        logo,
        payload,
        identifier: Vec::new(),
        serial: Vec::new(),
    })
}

/// `container` encrypted with `seed` as byte 0; the three digests are recomputed.
pub(crate) fn encrypt(container: &SaveContainer, seed: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(
        HEAD_SIZE
            + 8
            + container.description.len()
            + container.logo.len()
            + container.payload.len(),
    );
    out.push(seed);
    out.extend_from_slice(&digest(&container.description));
    out.extend_from_slice(&digest(&container.logo));
    out.extend_from_slice(&digest(&container.payload));
    out.extend_from_slice(&lcg(seed, &container.description));
    out.extend_from_slice(&(container.logo.len() as u32).to_le_bytes());
    out.extend_from_slice(&lcg(seed, &container.logo));
    out.extend_from_slice(&(container.payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&lcg(seed, &container.payload));
    out
}
