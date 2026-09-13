//! The round-trip float text the XML form uses: the shortest `%.{p}g`
//! rendering that parses back to the same bits, `E+NN`/`E-NN` exponents,
//! `-0` for negative zero.

/// `value` as C-style `%.{precision}g` with an uppercase, always-signed,
/// at-least-two-digit exponent. Finite `value` only.
fn general(value: f64, precision: usize) -> String {
    let scientific = format!("{:.*e}", precision - 1, value);
    let (mantissa, exponent) = scientific.split_once('e').expect("exponent marker");
    let exponent: i32 = exponent.parse().expect("decimal exponent");
    if exponent < -4 || exponent >= precision as i32 {
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        format!("{mantissa}E{exponent:+03}")
    } else {
        let decimals = (precision - 1) as i32 - exponent;
        let fixed = format!("{:.*}", decimals.max(0) as usize, value);
        fixed
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

/// A `f32` as the XML form writes it: the shortest of 7 and 9 significant digits that reads
/// back to the same value, `E+NN`/`E-NN` exponents, `-0` for negative zero, `nan`/`inf`/`-inf`
/// for non-finite values.
pub fn float_text(value: f32) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value < 0.0 { "-inf" } else { "inf" }.to_string();
    }
    if value == 0.0 {
        return if value.is_sign_negative() { "-0" } else { "0" }.to_string();
    }
    for precision in [7, 9] {
        let text = general(f64::from(value), precision);
        if text.parse::<f32>().map(f32::to_bits) == Ok(value.to_bits()) {
            return text;
        }
    }
    general(f64::from(value), 9)
}

/// A `f64` the way the XML form writes it: the shortest text that reads back to the same
/// value, lowercase `e`, exponent form when the decimal exponent is `< -4` or `>= 16`,
/// a `.0` appended to an integral fixed value, `-0.0` for negative zero. Untested against a
/// real file: no fixture carries a double, so this follows the rule without a golden.
pub fn double_text(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value < 0.0 { "-inf" } else { "inf" }.to_string();
    }
    let scientific = format!("{value:e}");
    let (mantissa, exponent) = scientific.split_once('e').expect("exponent marker");
    let exponent: i32 = exponent.parse().expect("decimal exponent");
    if !(-4..16).contains(&exponent) {
        format!("{mantissa}e{exponent:+03}")
    } else {
        let fixed = format!("{value}");
        if fixed.contains('.') {
            fixed
        } else {
            format!("{fixed}.0")
        }
    }
}

/// Reads what `float_text` writes (and plain decimal text); `-0` gives negative zero.
pub fn parse_float(text: &str) -> Option<f32> {
    text.parse::<f32>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_golden() {
        let mut checked = 0;
        for line in include_str!("../tests/fixtures/float_golden.tsv").lines() {
            let (bits, text) = line.split_once('\t').expect("bits<TAB>text");
            let bits = u32::from_str_radix(bits, 16).expect("hex bits");
            assert_eq!(float_text(f32::from_bits(bits)), text, "bits {bits:08X}");
            assert_eq!(parse_float(text).map(f32::to_bits), Some(bits));
            checked += 1;
        }
        assert_eq!(checked, 36);
    }

    #[test]
    fn non_finite() {
        assert_eq!(float_text(f32::NAN), "nan");
        assert_eq!(float_text(f32::INFINITY), "inf");
        assert_eq!(float_text(f32::NEG_INFINITY), "-inf");
        assert_eq!(parse_float("-inf"), Some(f32::NEG_INFINITY));
    }

    #[test]
    fn double_values() {
        assert_eq!(double_text(0.5), "0.5");
        assert_eq!(double_text(1.0), "1.0");
        assert_eq!(double_text(100.0), "100.0");
        assert_eq!(double_text(1e-5), "1e-05");
        assert_eq!(double_text(1.5e-5), "1.5e-05");
        assert_eq!(double_text(1e16), "1e+16");
        assert_eq!(double_text(0.0), "0.0");
        assert_eq!(double_text(-0.0), "-0.0");
        assert_eq!(double_text(0.1), "0.1");
    }
}
