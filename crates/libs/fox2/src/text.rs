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
        // Trailing zeros are fractional only — an integer like `123456720` keeps its digits.
        if fixed.contains('.') {
            fixed
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        } else {
            fixed
        }
    }
}

/// A `f32` as the XML form writes it: 7 significant digits when that reads back to the same
/// value, else 9 (which always round-trips an `f32`), `E+NN`/`E-NN` exponents, `-0` for
/// negative zero, `nan`/`inf`/`-inf` for non-finite values.
pub(crate) fn float_text(value: f32) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-inf"
        } else {
            "inf"
        }
        .to_string();
    }
    if value == 0.0 {
        return if value.is_sign_negative() { "-0" } else { "0" }.to_string();
    }
    let text = general(f64::from(value), 7);
    if parse_float(&text).map(f32::to_bits) == Some(value.to_bits()) {
        return text;
    }
    general(f64::from(value), 9)
}

/// `value`'s shortest exact-precision digits and its decimal exponent. `{:.*e}` rounds the
/// last digit ties-to-even, which the shortest-digits `{e}`/`{}` forms do not.
fn shortest_digits(value: f64) -> (String, i32) {
    for precision in 1..=17 {
        let scientific = format!("{:.*e}", precision - 1, value);
        if scientific.parse::<f64>().map(f64::to_bits) == Ok(value.to_bits()) {
            let (mantissa, exponent) = scientific.split_once('e').expect("exponent marker");
            let digits: String = mantissa.chars().filter(|c| c.is_ascii_digit()).collect();
            return (digits, exponent.parse().expect("decimal exponent"));
        }
    }
    unreachable!("17 digits always round-trip an f64")
}

/// A `f64` the way the XML form writes it: the shortest text that reads back to the same
/// value, lowercase `e`, exponent form when the decimal exponent is `< -4` or `>= 16`,
/// a `.0` appended to an integral fixed value, `-0.0` for negative zero. Untested against a
/// real file: no fixture carries a double; `double_golden.tsv` pins the expected text.
pub(crate) fn double_text(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-inf"
        } else {
            "inf"
        }
        .to_string();
    }
    let (digits, exponent) = shortest_digits(value);
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if !(-4..16).contains(&exponent) {
        let mut text = format!("{sign}{}", &digits[..1]);
        if digits.len() > 1 {
            text.push('.');
            text.push_str(&digits[1..]);
        }
        text.push_str(&format!("e{exponent:+03}"));
        return text;
    }
    let mut text = String::from(sign);
    if exponent >= 0 {
        let point = digits.len().min(exponent as usize + 1);
        text.push_str(&digits[..point]);
        text.push_str(&"0".repeat(exponent as usize + 1 - point));
        if digits.len() <= point {
            text.push_str(".0");
        } else {
            text.push('.');
            text.push_str(&digits[point..]);
        }
    } else {
        text.push_str("0.");
        text.push_str(&"0".repeat((-exponent - 1) as usize));
        text.push_str(&digits);
    }
    text
}

/// Reads what `float_text` writes (and plain decimal text), parsing as `f64` and narrowing:
/// the double rounding is deliberate (the XML form's floats are read through binary64, so a
/// 17-digit text compiles to the same bits as the goldens' producer), and `f64 as f32` is the
/// rounding conversion (there is no `From`).
pub(crate) fn parse_float(text: &str) -> Option<f32> {
    text.parse::<f64>().ok().map(|value| value as f32)
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
    fn double_golden() {
        let mut checked = 0;
        for line in include_str!("../tests/fixtures/double_golden.tsv").lines() {
            let (bits, text) = line.split_once('\t').expect("bits<TAB>text");
            let bits = u64::from_str_radix(bits, 16).expect("hex bits");
            assert_eq!(double_text(f64::from_bits(bits)), text, "bits {bits:016X}");
            if text != "nan" {
                assert_eq!(
                    text.parse::<f64>().map(f64::to_bits),
                    Ok(bits),
                    "text {text:?}"
                );
            }
            checked += 1;
        }
        assert_eq!(checked, 29);
    }

    #[test]
    fn integer_digits_are_not_trimmed() {
        // The 7-digit text does not round-trip; the 9-digit fixed text is `123456720`.
        assert_eq!(float_text(123456720.0), "123456720");
        assert_eq!(
            parse_float("123456720").map(f32::to_bits),
            Some(123456720.0f32.to_bits())
        );
        assert_eq!(float_text(100000000.0), "1E+08");
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
