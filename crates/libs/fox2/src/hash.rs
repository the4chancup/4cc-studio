//! The 48-bit string hash the format keys every class, property and
//! string value by: CityHash64 over the UTF-8 bytes plus a NUL
//! terminator, seeded with the first byte and the length, masked to 48
//! bits.

const K0: u64 = 0xC3A5C85C97CB3127;
const K1: u64 = 0xB492B66FBE98F273;
const K2: u64 = 0x9AE16A3B2F90404F;
const K3: u64 = 0xC949D7C7509E6557;

/// Eight little-endian bytes at `at` (callers guarantee the range).
fn fetch64(data: &[u8], at: usize) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&data[at..at + 8]);
    u64::from_le_bytes(bytes)
}

/// Four little-endian bytes at `at`.
fn fetch32(data: &[u8], at: usize) -> u64 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&data[at..at + 4]);
    u64::from(u32::from_le_bytes(bytes))
}

fn shift_mix(value: u64) -> u64 {
    value ^ (value >> 47)
}

fn hash128_to64(u: u64, v: u64) -> u64 {
    const K_MUL: u64 = 0x9DDFEA08EB382D69;
    let mut a = (u ^ v).wrapping_mul(K_MUL);
    a ^= a >> 47;
    let mut b = (v ^ a).wrapping_mul(K_MUL);
    b ^= b >> 47;
    b.wrapping_mul(K_MUL)
}

fn hash_len16(u: u64, v: u64) -> u64 {
    hash128_to64(u, v)
}

fn hash_len0to16(data: &[u8]) -> u64 {
    let length = data.len();
    if length > 8 {
        let a = fetch64(data, 0);
        let b = fetch64(data, length - 8);
        return hash_len16(a, b.wrapping_add(length as u64).rotate_right(length as u32)) ^ b;
    }
    if length >= 4 {
        let a = fetch32(data, 0);
        return hash_len16(
            (length as u64).wrapping_add(a << 3),
            fetch32(data, length - 4),
        );
    }
    if length > 0 {
        let a = u64::from(data[0]);
        let b = u64::from(data[length >> 1]);
        let c = u64::from(data[length - 1]);
        let y = a.wrapping_add(b << 8);
        let z = (length as u64).wrapping_add(c << 2);
        return shift_mix(y.wrapping_mul(K2) ^ z.wrapping_mul(K3)).wrapping_mul(K2);
    }
    0
}

fn hash_len17to32(data: &[u8]) -> u64 {
    let length = data.len();
    let a = fetch64(data, 0).wrapping_mul(K1);
    let b = fetch64(data, 8);
    let c = fetch64(data, length - 8).wrapping_mul(K2);
    let d = fetch64(data, length - 16).wrapping_mul(K0);
    hash_len16(
        a.wrapping_sub(b)
            .rotate_right(43)
            .wrapping_add(c.rotate_right(30))
            .wrapping_add(d),
        a.wrapping_add((b ^ K3).rotate_right(20))
            .wrapping_sub(c)
            .wrapping_add(length as u64),
    )
}

fn weak_hash_len32_with_seeds(w: u64, x: u64, y: u64, z: u64, a: u64, b: u64) -> (u64, u64) {
    let mut a = a.wrapping_add(w);
    let mut b = b.wrapping_add(a).wrapping_add(z).rotate_right(21);
    let c = a;
    a = a.wrapping_add(x);
    a = a.wrapping_add(y);
    b = b.wrapping_add(a.rotate_right(44));
    (a.wrapping_add(z), b.wrapping_add(c))
}

fn weak_hash_len32(data: &[u8], at: usize, a: u64, b: u64) -> (u64, u64) {
    weak_hash_len32_with_seeds(
        fetch64(data, at),
        fetch64(data, at + 8),
        fetch64(data, at + 16),
        fetch64(data, at + 24),
        a,
        b,
    )
}

fn hash_len33to64(data: &[u8]) -> u64 {
    let length = data.len();
    let z0 = fetch64(data, 24);
    let mut a = fetch64(data, 0).wrapping_add(
        (length as u64)
            .wrapping_add(fetch64(data, length - 16))
            .wrapping_mul(K0),
    );
    let mut b = a.wrapping_add(z0).rotate_right(52);
    let mut c = a.rotate_right(37);
    a = a.wrapping_add(fetch64(data, 8));
    c = c.wrapping_add(a.rotate_right(7));
    a = a.wrapping_add(fetch64(data, 16));
    let vf = a.wrapping_add(z0);
    let vs = b.wrapping_add(a.rotate_right(31)).wrapping_add(c);
    a = fetch64(data, 16).wrapping_add(fetch64(data, length - 32));
    let z1 = fetch64(data, length - 8);
    b = a.wrapping_add(z1).rotate_right(52);
    c = a.rotate_right(37);
    a = a.wrapping_add(fetch64(data, length - 24));
    c = c.wrapping_add(a.rotate_right(7));
    a = a.wrapping_add(fetch64(data, length - 16));
    let wf = a.wrapping_add(z1);
    let ws = b.wrapping_add(a.rotate_right(31)).wrapping_add(c);
    let r = shift_mix(
        vf.wrapping_add(ws)
            .wrapping_mul(K2)
            .wrapping_add(wf.wrapping_add(vs).wrapping_mul(K0)),
    );
    shift_mix(r.wrapping_mul(K0).wrapping_add(vs)).wrapping_mul(K2)
}

fn hash_len_above64(data: &[u8]) -> u64 {
    let length = data.len();
    let mut x = fetch64(data, length - 40);
    let mut y = fetch64(data, length - 16).wrapping_add(fetch64(data, length - 56));
    let mut z = hash_len16(
        fetch64(data, length - 48).wrapping_add(length as u64),
        fetch64(data, length - 24),
    );
    let (mut v_lo, mut v_hi) = weak_hash_len32(data, length - 64, length as u64, z);
    let (mut w_lo, mut w_hi) = weak_hash_len32(data, length - 32, y.wrapping_add(K1), x);
    x = x.wrapping_mul(K1).wrapping_add(fetch64(data, 0));

    let mut offset = 0usize;
    let mut remaining = (length - 1) & !63;
    while remaining > 0 {
        x = x
            .wrapping_add(y)
            .wrapping_add(v_lo)
            .wrapping_add(fetch64(data, offset + 8))
            .rotate_right(37)
            .wrapping_mul(K1);
        y = y
            .wrapping_add(v_hi)
            .wrapping_add(fetch64(data, offset + 48))
            .rotate_right(42)
            .wrapping_mul(K1);
        x ^= w_hi;
        y = y
            .wrapping_add(v_lo)
            .wrapping_add(fetch64(data, offset + 40));
        z = z.wrapping_add(w_lo).rotate_right(33).wrapping_mul(K1);
        (v_lo, v_hi) = weak_hash_len32(data, offset, v_hi.wrapping_mul(K1), x.wrapping_add(w_lo));
        (w_lo, w_hi) = weak_hash_len32(
            data,
            offset + 32,
            z.wrapping_add(w_hi),
            y.wrapping_add(fetch64(data, offset + 16)),
        );
        std::mem::swap(&mut z, &mut x);
        offset += 64;
        remaining -= 64;
    }

    hash_len16(
        hash_len16(v_lo, w_lo)
            .wrapping_add(shift_mix(y).wrapping_mul(K1))
            .wrapping_add(z),
        hash_len16(v_hi, w_hi).wrapping_add(x),
    )
}

fn city_hash64(data: &[u8]) -> u64 {
    match data.len() {
        0..=16 => hash_len0to16(data),
        17..=32 => hash_len17to32(data),
        33..=64 => hash_len33to64(data),
        _ => hash_len_above64(data),
    }
}

fn city_hash64_with_seeds(data: &[u8], seed0: u64, seed1: u64) -> u64 {
    hash_len16(city_hash64(data).wrapping_sub(seed0), seed1)
}

/// The 48-bit hash the format keys every class, property and string value by.
pub fn hash_string(text: &str) -> u64 {
    let mut data = Vec::with_capacity(text.len() + 1);
    data.extend_from_slice(text.as_bytes());
    data.push(0);
    let seed1 = ((u32::from(data[0])) << 16).wrapping_add(data.len() as u32 - 1);
    city_hash64_with_seeds(&data, 0x9AE16A3B2F90404F, u64::from(seed1)) & 0xFFFF_FFFF_FFFF
}

/// A string dictionary: hash → text, used to resolve `Hash` strings a file's own table
/// does not cover.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Dictionary(std::collections::HashMap<u64, String>);

impl Dictionary {
    /// One literal per line, hashed; the empty line included. The first occurrence of a hash
    /// wins; a leading UTF-8 BOM is stripped.
    pub fn from_lines(text: &str) -> Dictionary {
        let mut map = std::collections::HashMap::new();
        for line in text.strip_prefix('\u{FEFF}').unwrap_or(text).lines() {
            map.entry(hash_string(line))
                .or_insert_with(|| line.to_string());
        }
        Dictionary(map)
    }

    /// The text for `hash`, when the dictionary knows it.
    pub fn get(&self, hash: u64) -> Option<&str> {
        self.0.get(&hash).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden() {
        let mut checked = 0;
        for line in include_str!("../tests/fixtures/hash_golden.tsv").lines() {
            let (hash, text) = line.split_once('\t').expect("hash<TAB>text");
            let hash = u64::from_str_radix(hash, 16).expect("hex hash");
            assert_eq!(hash_string(text), hash, "text {text:?}");
            checked += 1;
        }
        assert_eq!(checked, 46);
    }

    #[test]
    fn fixture_tables() {
        for bytes in [
            include_bytes!("../tests/fixtures/audi_low_parts.fox2").as_slice(),
            include_bytes!("../tests/fixtures/boots_edit_k0051.fox2").as_slice(),
            include_bytes!("../tests/fixtures/edit_spike.fox2").as_slice(),
            include_bytes!("../tests/fixtures/steward_sit_st074.fox2").as_slice(),
        ] {
            let mut at = i32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
            let mut checked = 0;
            loop {
                let hash = u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
                if hash == 0 {
                    break;
                }
                let len = u32::from_le_bytes(bytes[at + 8..at + 12].try_into().unwrap());
                let text =
                    std::str::from_utf8(&bytes[at + 12..at + 12 + usize::try_from(len).unwrap()])
                        .expect("utf8 string");
                assert_eq!(hash_string(text), hash, "text {text:?}");
                checked += 1;
                at += 12 + usize::try_from(len).unwrap();
            }
            assert!(checked >= 11, "only {checked} strings in the table");
        }
    }
}
