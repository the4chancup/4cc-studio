//! IEEE 754 binary16 (half) conversion, hand-written: 1 sign bit, 5
//! exponent bits biased at 15, 10 mantissa bits. Decoding is exact by
//! construction; encoding is round-to-nearest-even.

/// Decodes a half-precision bit pattern to `f32`. Every half value,
/// subnormals and NaN payloads included, has an exact `f32`.
pub(crate) fn f16_to_f32(bits: u16) -> f32 {
    let sign = u32::from(bits & 0x8000) << 16;
    let exponent = u32::from(bits >> 10) & 0x1F;
    let mantissa = u32::from(bits & 0x03FF);
    if exponent == 0x1F {
        // Infinity (mantissa 0) or NaN: quiet bit set, payload kept in the
        // top mantissa bits.
        let payload = if mantissa == 0 {
            0
        } else {
            0x0040_0000 | (mantissa << 13)
        };
        return f32::from_bits(sign | 0x7F80_0000 | payload);
    }
    if exponent == 0 {
        // Subnormal or zero: mantissa * 2^-24, exactly representable.
        let magnitude = (mantissa as f32) * f32::from_bits(0x3380_0000);
        return f32::from_bits(sign | magnitude.to_bits());
    }
    f32::from_bits(sign | ((exponent + 112) << 23) | (mantissa << 13))
}

/// Encodes an `f32` to the nearest half bit pattern (ties to even).
/// NaN becomes a quiet NaN keeping the sign; magnitudes beyond the half
/// range become the signed infinity.
pub(crate) fn f32_to_f16(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let abs = bits & 0x7FFF_FFFF;
    if abs > 0x7F80_0000 {
        // NaN: keep sign and payload, set the quiet bit so the result is a
        // NaN even when the surviving payload bits were empty.
        return sign | 0x7E00 | (((abs & 0x007F_FFFF) >> 13) as u16);
    }
    if abs >= 0x4780_0000 {
        // Infinity or too large for a half.
        return sign | 0x7C00;
    }
    let exponent = ((abs >> 23) & 0xFF) as i32;
    if exponent == 0 {
        // f32 subnormal: far below the smallest half.
        return sign;
    }
    let half_exponent = exponent - 112;
    if half_exponent <= 0 {
        // Half subnormal or zero: the f32 mantissa with its implicit bit,
        // shifted into units of 2^-24 with round-to-nearest-even on the
        // remainder.
        let mantissa = (abs & 0x007F_FFFF) | 0x0080_0000;
        let shift = 14 - half_exponent;
        if shift > 24 {
            return sign;
        }
        let mut result = (mantissa >> shift) as u16;
        let remainder = mantissa & ((1 << shift) - 1);
        let halfway = 1 << (shift - 1);
        if remainder > halfway || (remainder == halfway && result & 1 == 1) {
            result += 1;
        }
        return sign | result;
    }
    let stored = abs & 0x007F_FFFF;
    let mut result = ((half_exponent as u16) << 10) | ((stored >> 13) as u16);
    let remainder = stored & 0x1FFF;
    if remainder > 0x1000 || (remainder == 0x1000 && result & 1 == 1) {
        // Mantissa overflow rolls into the exponent, which is the correct
        // carry (and reaches infinity at the top of the range).
        result += 1;
    }
    sign | result
}
