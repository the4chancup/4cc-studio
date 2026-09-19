//! The PES 16-21 keyed container: a 320-byte salt encoding the file key, an
//! encrypted file header (176 bytes on 16/17, 208 on 18-21), then four sections
//! (description, logo, payload, serial) each encrypted under a per-section key.

use super::keys::MasterKey;
use super::mt19937::crypt;
use super::{ContainerError, SaveContainer, Scheme};

/// The encryption-header ("salt") byte count at the start of every keyed save.
pub(crate) const SALT_SIZE: usize = 320;
/// The file header carries the effective key in `[0..64]`, four u32 sizes in
/// `[64..80]` (payload, logo, description, serial units), then the opaque identifier.
const SIZES_OFFSET: usize = 64;
const IDENTIFIER_OFFSET: usize = 80;

/// The decrypted salt: `header_key = effective ^ salt[256..320]`, the keystream under
/// `header_key` XORed onto `salt[0..256]`, then `salt[256..320]` verbatim. The same
/// routine runs on write, so the file key derivation is shared by decrypt and encrypt.
fn decrypt_salt(key: MasterKey, salt: &[u8]) -> [u8; SALT_SIZE] {
    let effective = key.effective();
    let mut header_key = [0u8; 64];
    for (index, byte) in header_key.iter_mut().enumerate() {
        *byte = effective[index] ^ salt[256 + index];
    }
    let mut decrypted = [0u8; SALT_SIZE];
    decrypted[..256].copy_from_slice(&crypt(&header_key, &salt[..256]));
    decrypted[256..].copy_from_slice(&salt[256..]);
    decrypted
}

/// The file key: the XOR of the decrypted salt's five 64-byte blocks.
fn file_key(decrypted_salt: &[u8; SALT_SIZE]) -> [u8; 64] {
    let mut key = [0u8; 64];
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = decrypted_salt[index]
            ^ decrypted_salt[64 + index]
            ^ decrypted_salt[128 + index]
            ^ decrypted_salt[192 + index]
            ^ decrypted_salt[256 + index];
    }
    key
}

/// The file key of `salt` under `key`, without decrypting anything else (tests).
pub(crate) fn salt_file_key(key: MasterKey, salt: &[u8]) -> [u8; 64] {
    file_key(&decrypt_salt(key, salt))
}

/// Section key `n`: every 8-byte lane of the file key XOR `n` as a little-endian u64.
/// `n` is the header's byte length for the header itself, then 0 description, 1 logo,
/// 2 payload, 3 serial.
pub(crate) fn section_key(file_key: &[u8; 64], n: u64) -> [u8; 64] {
    let mut key = *file_key;
    for lane in key.as_chunks_mut::<8>().0 {
        let value = u64::from_le_bytes(*lane) ^ n;
        *lane = value.to_le_bytes();
    }
    key
}

/// `bytes` decrypted under `key`, or `Unrecognized` when the header's first 64 bytes
/// do not come out as the effective key. A matching key whose declared sizes do not
/// land exactly on the end of the input is `Truncated`/`TrailingBytes`.
pub(crate) fn decrypt(bytes: &[u8], key: MasterKey) -> Result<SaveContainer, ContainerError> {
    let header_size = key.header_size();
    let Some(header_bytes) = bytes.len().checked_sub(SALT_SIZE) else {
        return Err(ContainerError::Unrecognized);
    };
    // The 64-byte compare is what identifies the scheme; a file too short to check
    // cannot be claimed by this key.
    if header_bytes < 64 {
        return Err(ContainerError::Unrecognized);
    }
    let file_key = salt_file_key(key, &bytes[..SALT_SIZE]);
    let header = crypt(
        &section_key(&file_key, header_size as u64),
        &bytes[SALT_SIZE..SALT_SIZE + header_bytes.min(header_size)],
    );
    if header[..64] != key.effective() {
        return Err(ContainerError::Unrecognized);
    }
    let needed = SALT_SIZE + header_size;
    if header_bytes < header_size {
        return Err(ContainerError::Truncated {
            needed,
            available: bytes.len(),
        });
    }
    let size = |offset: usize| {
        u32::from_le_bytes(header[offset..offset + 4].try_into().expect("4 bytes")) as usize
    };
    let (payload_size, logo_size, description_size, serial_units) = (
        size(SIZES_OFFSET),
        size(SIZES_OFFSET + 4),
        size(SIZES_OFFSET + 8),
        size(SIZES_OFFSET + 12),
    );
    let Some(serial_size) = serial_units.checked_mul(2) else {
        return Err(ContainerError::Truncated {
            needed: usize::MAX,
            available: bytes.len(),
        });
    };
    let needed = SALT_SIZE
        .checked_add(header_size)
        .and_then(|total| total.checked_add(description_size))
        .and_then(|total| total.checked_add(logo_size))
        .and_then(|total| total.checked_add(payload_size))
        .and_then(|total| total.checked_add(serial_size));
    let Some(needed) = needed else {
        return Err(ContainerError::Truncated {
            needed: usize::MAX,
            available: bytes.len(),
        });
    };
    if bytes.len() < needed {
        return Err(ContainerError::Truncated {
            needed,
            available: bytes.len(),
        });
    }
    if bytes.len() > needed {
        return Err(ContainerError::TrailingBytes(bytes.len() - needed));
    }
    let mut offset = SALT_SIZE + header_size;
    let mut section = |n: u64, len: usize| -> Vec<u8> {
        let out = crypt(&section_key(&file_key, n), &bytes[offset..offset + len]);
        offset += len;
        out
    };
    let description = section(0, description_size);
    let logo = section(1, logo_size);
    let payload = section(2, payload_size);
    let serial = section(3, serial_size);
    Ok(SaveContainer {
        scheme: Scheme::Keyed(key),
        description,
        logo,
        payload,
        identifier: header[IDENTIFIER_OFFSET..].to_vec(),
        serial,
    })
}

/// `container` encrypted under `key` with `salt` as the file's first 320 bytes.
/// `serial.len()` must be even and `identifier.len()` must be `header_size - 80`.
pub(crate) fn encrypt(
    container: &SaveContainer,
    salt: &[u8; SALT_SIZE],
    key: MasterKey,
) -> Result<Vec<u8>, ContainerError> {
    let header_size = key.header_size();
    if !container.serial.len().is_multiple_of(2) {
        return Err(ContainerError::OddSerial(container.serial.len()));
    }
    let expected = header_size - IDENTIFIER_OFFSET;
    if container.identifier.len() != expected {
        return Err(ContainerError::BadIdentifier {
            expected,
            got: container.identifier.len(),
        });
    }
    let mut header = vec![0u8; header_size];
    header[..64].copy_from_slice(&key.effective());
    for (section, (offset, size)) in [
        (SIZES_OFFSET, container.payload.len()),
        (SIZES_OFFSET + 4, container.logo.len()),
        (SIZES_OFFSET + 8, container.description.len()),
        (SIZES_OFFSET + 12, container.serial.len() / 2),
    ]
    .into_iter()
    .enumerate()
    {
        let size = u32::try_from(size).map_err(|_| ContainerError::SectionTooLarge {
            section: section as u8,
            size,
        })?;
        header[offset..offset + 4].copy_from_slice(&size.to_le_bytes());
    }
    header[IDENTIFIER_OFFSET..].copy_from_slice(&container.identifier);
    let file_key = salt_file_key(key, salt);
    let mut out = Vec::with_capacity(
        SALT_SIZE
            + header_size
            + container.description.len()
            + container.logo.len()
            + container.payload.len()
            + container.serial.len(),
    );
    out.extend_from_slice(salt);
    out.extend_from_slice(&crypt(&section_key(&file_key, header_size as u64), &header));
    out.extend_from_slice(&crypt(&section_key(&file_key, 0), &container.description));
    out.extend_from_slice(&crypt(&section_key(&file_key, 1), &container.logo));
    out.extend_from_slice(&crypt(&section_key(&file_key, 2), &container.payload));
    out.extend_from_slice(&crypt(&section_key(&file_key, 3), &container.serial));
    Ok(out)
}
