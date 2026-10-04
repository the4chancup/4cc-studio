//! The `colors.txt` reader (`player_folders.md` "Root files", "Colors"): one grammar for the
//! export's root `colors.txt`, the team's colors, and a kit folder's `colors.txt`, the kit's
//! two menu colors. Bytes in, the colors and the refused lines out.

/// What a `colors.txt` holds: its valid colors, as many as the file's capacity allows, and
/// the lines it refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorsTxt {
    /// The valid colors in line order, at most the capacity asked for, each R, G, B.
    pub colors: Vec<[u8; 3]>,
    /// The lines refused, in line order.
    pub refused: Vec<RefusedColorLine>,
}

/// A `colors.txt` line that gives no color: it is skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefusedColorLine {
    /// The line's number in the file, counting every line, blank ones included, from 1.
    pub line: usize,
    /// Why the line gives no color.
    pub reason: ColorLineRefusal,
}

/// Why a `colors.txt` line gives no color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorLineRefusal {
    /// The line does not hold exactly one color: two colors on one line (the old Team Note
    /// kit entry `211 74 79 - 162 62 77`), a trailing icon number, a component over 255, a
    /// hex color without its `#`, bytes that are not UTF-8.
    NotOneColor,
    /// A valid color past the file's capacity.
    PastCapacity,
}

/// The colors of the `colors.txt` `bytes`, keeping the first `capacity` valid ones: two for a
/// kit's file, whose two colors are the kit's menu colors, four for the root file, the
/// team's colors (a `TeamColor.bin` record holds four).
///
/// The file is UTF-8 with an optional BOM, its lines ending in LF or CRLF. A blank line is
/// skipped. Any other line holds one color, optionally after a label ending in `:` (the color
/// is what follows the line's last `:`), a leading `- ` tolerated: `#RRGGBB`, or three decimal
/// components 0-255 separated by spaces, commas, or both. A line that is not UTF-8 is refused
/// on its own; the other lines are read.
pub fn read_colors_txt(bytes: &[u8], capacity: usize) -> ColorsTxt {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let mut read = ColorsTxt {
        colors: Vec::new(),
        refused: Vec::new(),
    };
    for (index, line) in bytes.split(|&byte| byte == b'\n').enumerate() {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        let color = match std::str::from_utf8(line) {
            Ok(text) if text.trim().is_empty() => continue,
            Ok(text) => line_color(text),
            Err(_) => None,
        };
        let reason = match color {
            None => ColorLineRefusal::NotOneColor,
            Some(color) if read.colors.len() < capacity => {
                read.colors.push(color);
                continue;
            }
            Some(_) => ColorLineRefusal::PastCapacity,
        };
        read.refused.push(RefusedColorLine {
            line: index + 1,
            reason,
        });
    }
    read
}

/// The one color the non-blank line `line` gives, or `None`.
fn line_color(line: &str) -> Option<[u8; 3]> {
    let line = line.trim_start();
    let line = line.strip_prefix("- ").unwrap_or(line);
    let color = match line.rfind(':') {
        Some(colon) => &line[colon + 1..],
        None => line,
    };
    let color = color.trim();
    match color.strip_prefix('#') {
        Some(hex) => hex_color(hex),
        None => decimal_color(color),
    }
}

/// `RRGGBB`, exactly six hex digits of either case, as a color.
fn hex_color(hex: &str) -> Option<[u8; 3]> {
    if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let [_, red, green, blue] = u32::from_str_radix(hex, 16).ok()?.to_be_bytes();
    Some([red, green, blue])
}

/// Exactly three decimal components 0-255, separated by spaces, commas, or both, as a color.
fn decimal_color(text: &str) -> Option<[u8; 3]> {
    let components: Option<Vec<u8>> = text
        .split([' ', ','])
        .filter(|component| !component.is_empty())
        .map(decimal_component)
        .collect();
    match components?.as_slice() {
        &[red, green, blue] => Some([red, green, blue]),
        _ => None,
    }
}

/// One decimal component 0-255, digits only.
fn decimal_component(text: &str) -> Option<u8> {
    // `u8::from_str` also takes a leading `+`, which no color is written with.
    if !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `refusals` as refused lines, each a line number and a reason.
    fn refused(refusals: &[(usize, ColorLineRefusal)]) -> Vec<RefusedColorLine> {
        refusals
            .iter()
            .map(|&(line, reason)| RefusedColorLine { line, reason })
            .collect()
    }

    const RED: [u8; 3] = [0xc1, 0x12, 0x00];
    const GREY: [u8; 3] = [0x41, 0x41, 0x41];

    #[test]
    fn a_kit_s_two_colors_in_either_form_are_read_and_nothing_is_refused() {
        assert_eq!(
            read_colors_txt(b"#c11200\n#414141\n", 2),
            ColorsTxt {
                colors: vec![RED, GREY],
                refused: vec![],
            }
        );
        assert_eq!(
            read_colors_txt(b"211 74 79\r\n162 62 77\r\n", 2),
            ColorsTxt {
                colors: vec![[211, 74, 79], [162, 62, 77]],
                refused: vec![],
            }
        );
    }

    #[test]
    fn each_valid_form_of_a_line_gives_its_color() {
        let cases: [(&[u8], [u8; 3]); 8] = [
            (b"211, 74, 79", [211, 74, 79]),
            (b"211,74,79", [211, 74, 79]),
            (b"#C11200", RED),
            (b"  #c11200  ", RED),
            (b"- 1st: #c11200", RED),
            (b"Shirt: 211 74 79", [211, 74, 79]),
            // The color follows the line's last `:`.
            (b"Kit: home: #c11200", RED),
            (b"\xEF\xBB\xBF#c11200\n", RED),
        ];
        for (bytes, color) in cases {
            assert_eq!(
                read_colors_txt(bytes, 2),
                ColorsTxt {
                    colors: vec![color],
                    refused: vec![],
                },
                "{}",
                String::from_utf8_lossy(bytes)
            );
        }
    }

    #[test]
    fn a_line_not_holding_exactly_one_color_is_refused() {
        let lines = [
            "211 74 79 - 162 62 77",
            "#c11200 - #414141 - 11",
            "#c1120",
            "#c112000",
            "256 0 0",
            "1 2",
            "1 2 3 4",
            "c11200",
            "1st:",
            "red",
            "-1 2 3",
        ];
        for line in lines {
            assert_eq!(
                read_colors_txt(line.as_bytes(), 2),
                ColorsTxt {
                    colors: vec![],
                    refused: refused(&[(1, ColorLineRefusal::NotOneColor)]),
                },
                "{line}"
            );
        }
    }

    #[test]
    fn blank_lines_are_skipped_and_still_counted() {
        assert_eq!(
            read_colors_txt(b"#c11200\n\n   \nbad\n#414141\n", 2),
            ColorsTxt {
                colors: vec![RED, GREY],
                refused: refused(&[(4, ColorLineRefusal::NotOneColor)]),
            }
        );
    }

    #[test]
    fn a_valid_color_past_the_capacity_is_refused_and_an_invalid_line_there_is_not_one_color() {
        assert_eq!(
            read_colors_txt(b"#c11200\n#414141\n211 74 79\n", 2),
            ColorsTxt {
                colors: vec![RED, GREY],
                refused: refused(&[(3, ColorLineRefusal::PastCapacity)]),
            }
        );
        assert_eq!(
            read_colors_txt(b"#c11200\n#414141\n211 74 79\n1 2 3\n4 5 6\n", 4),
            ColorsTxt {
                colors: vec![RED, GREY, [211, 74, 79], [1, 2, 3]],
                refused: refused(&[(5, ColorLineRefusal::PastCapacity)]),
            }
        );
        assert_eq!(
            read_colors_txt(b"#c11200\n#414141\nbad\n", 2),
            ColorsTxt {
                colors: vec![RED, GREY],
                refused: refused(&[(3, ColorLineRefusal::NotOneColor)]),
            }
        );
    }

    #[test]
    fn a_line_that_is_not_utf8_is_refused_alone() {
        assert_eq!(
            read_colors_txt(b"#c11200\n\xff\xfe\n#414141\n", 2),
            ColorsTxt {
                colors: vec![RED, GREY],
                refused: refused(&[(2, ColorLineRefusal::NotOneColor)]),
            }
        );
    }
}
