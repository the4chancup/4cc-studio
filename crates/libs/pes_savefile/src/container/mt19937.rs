//! The Mersenne Twister (MT19937) the keyed container's keystream is generated with:
//! the standard `init_by_array` seeding, then the rotation/XOR whitening cascade a
//! section is XORed against.

/// The MT19937 state size.
const N: usize = 624;
/// The twist's middle-word offset.
const M: usize = 397;
/// The twist matrix.
const MATRIX_A: u32 = 0x9908_b0df;
/// The twist's high/low bit split.
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7fff_ffff;

/// An MT19937 generator; the only seeding the container uses is [`Mt19937::init_by_array`].
pub(crate) struct Mt19937 {
    mt: [u32; N],
    mti: usize,
}

impl Mt19937 {
    /// `init_genrand`: the scalar seed `init_by_array` primes the state with.
    fn init_genrand(seed: u32) -> Self {
        let mut mt = [0u32; N];
        mt[0] = seed;
        for i in 1..N {
            mt[i] = 1812433253u32
                .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        Self { mt, mti: N }
    }

    /// `init_by_array`: seed from an array of key words (the 64-byte section key as
    /// 16 little-endian u32).
    pub(crate) fn init_by_array(key: &[u32]) -> Self {
        let mut twister = Self::init_genrand(19650218);
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..N.max(key.len()) {
            twister.mt[i] = (twister.mt[i]
                ^ (twister.mt[i - 1] ^ (twister.mt[i - 1] >> 30)).wrapping_mul(1664525))
            .wrapping_add(key[j])
            .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                twister.mt[0] = twister.mt[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..N - 1 {
            twister.mt[i] = (twister.mt[i]
                ^ (twister.mt[i - 1] ^ (twister.mt[i - 1] >> 30)).wrapping_mul(1566083941))
            .wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                twister.mt[0] = twister.mt[N - 1];
                i = 1;
            }
        }
        twister.mt[0] = UPPER_MASK;
        twister
    }

    /// `genrand_int32`: the next tempered 32-bit draw.
    pub(crate) fn next_u32(&mut self) -> u32 {
        if self.mti >= N {
            for kk in 0..N - M {
                let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
                self.mt[kk] = self.mt[kk + M] ^ (y >> 1) ^ if y & 1 != 0 { MATRIX_A } else { 0 };
            }
            for kk in N - M..N - 1 {
                let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
                self.mt[kk] =
                    self.mt[kk + M - N] ^ (y >> 1) ^ if y & 1 != 0 { MATRIX_A } else { 0 };
            }
            let y = (self.mt[N - 1] & UPPER_MASK) | (self.mt[0] & LOWER_MASK);
            self.mt[N - 1] = self.mt[M - 1] ^ (y >> 1) ^ if y & 1 != 0 { MATRIX_A } else { 0 };
            self.mti = 0;
        }
        let mut y = self.mt[self.mti];
        self.mti += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }
}

/// `data` XORed with the keystream `key` generates: the 64-byte key as 16 little-endian
/// u32 to `init_by_array`, four priming draws `c0..c3`, then per output word
/// `c4 ^ c3 ^ c2 ^ c1 ^ c0` with the rotation cascade; a tail under four bytes uses
/// the low bytes of one more word. The same routine encrypts and decrypts.
pub(crate) fn crypt(key: &[u8; 64], data: &[u8]) -> Vec<u8> {
    let mut words = [0u32; 16];
    for (index, word) in words.iter_mut().enumerate() {
        *word = u32::from_le_bytes(key[index * 4..index * 4 + 4].try_into().expect("4 bytes"));
    }
    let mut twister = Mt19937::init_by_array(&words);
    let (mut c0, mut c1, mut c2, mut c3) = (
        twister.next_u32(),
        twister.next_u32(),
        twister.next_u32(),
        twister.next_u32(),
    );
    let mut out = Vec::with_capacity(data.len());
    for chunk in data.chunks(4) {
        let c4 = twister.next_u32();
        let word = (c4 ^ c3 ^ c2 ^ c1 ^ c0).to_le_bytes();
        for (index, byte) in chunk.iter().enumerate() {
            out.push(byte ^ word[index]);
        }
        c0 = c1.rotate_right(15);
        c1 = c2.rotate_left(11);
        c2 = c3.rotate_left(7);
        c3 = c4.rotate_right(13);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The published `mt19937ar.out` vectors for `init_by_array`.
    #[test]
    fn init_by_array_known_answers() {
        let mut twister = Mt19937::init_by_array(&[0x123, 0x234, 0x345, 0x456]);
        let draws: Vec<u32> = (0..5).map(|_| twister.next_u32()).collect();
        assert_eq!(
            draws,
            [1067595299, 955945823, 477289528, 4107218783, 4228976476]
        );
        let key: Vec<u32> = (0..16).collect();
        let mut twister = Mt19937::init_by_array(&key);
        let draws: Vec<u32> = (0..8).map(|_| twister.next_u32()).collect();
        assert_eq!(
            draws,
            [
                420751713, 1317948889, 718696439, 3740536405, 324974999, 2940687479, 1494208090,
                2664957016
            ]
        );
    }

    #[test]
    fn keystream_known_answer() {
        let key: [u8; 64] = std::array::from_fn(|i| i as u8);
        assert_eq!(
            crypt(&key, &[0; 20]),
            [
                0x49, 0x17, 0x36, 0x3b, 0xdb, 0xb9, 0xc2, 0x56, 0x70, 0xa5, 0x17, 0x87, 0xbb, 0x1c,
                0xbb, 0xe1, 0xf8, 0x2c, 0xe8, 0x00
            ]
        );
        // A tail under four bytes uses the low bytes of the next word.
        assert_eq!(crypt(&key, &[0; 18]), crypt(&key, &[0; 20])[..18]);
    }
}
