//! `CpkStem`: a CPK file name without its `.cpk`, validated against the
//! writer's rules (`team_compiler/pipeline.md` "Writer" item 5): portable on
//! every platform the game and DLC run on.

use std::fmt;

/// A CPK file name without its `.cpk`: valid on every platform the game and
/// DLC run on.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CpkStem(String);

/// Why a string is not a `CpkStem`. The checks run in the enum's order, so
/// the first failing rule is the one reported.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CpkStemError {
    /// The stem is empty.
    #[error("empty")]
    Empty,
    /// More than 28 characters.
    #[error("{len} characters, more than 28")]
    TooLong {
        /// The character count.
        len: usize,
    },
    /// A character that is not a letter, digit, `_`, `-` or `.`.
    #[error("'{character}' is not a letter, digit, '_', '-' or '.'")]
    InvalidCharacter {
        /// The offending character.
        character: char,
    },
    /// A trailing `.`.
    #[error("ends with '.'")]
    TrailingDot,
    /// A `.cpk` suffix in any case; the extension is added at write time.
    #[error("ends with '.cpk'; the extension is added")]
    CpkSuffix,
    /// A Windows reserved device name on the part before the first `.`.
    #[error("'{name}' is a reserved Windows device name")]
    ReservedName {
        /// The stem part that matched.
        name: String,
    },
}

/// Windows reserved device names: `CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`,
/// `LPT1`–`LPT9` — judged on the part before the first `.`, since Windows
/// reserves `con.x` as it does `con`.
const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

impl CpkStem {
    /// 1–28 characters, each ASCII alphanumeric, `_`, `-` or `.`; no trailing
    /// `.`; no `.cpk` suffix in any case; not a Windows reserved name.
    pub fn new(text: &str) -> Result<CpkStem, CpkStemError> {
        if text.is_empty() {
            return Err(CpkStemError::Empty);
        }
        // Length in characters; the character rule makes that equal to bytes.
        let len = text.chars().count();
        if len > 28 {
            return Err(CpkStemError::TooLong { len });
        }
        for character in text.chars() {
            if !(character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')) {
                return Err(CpkStemError::InvalidCharacter { character });
            }
        }
        if text.ends_with('.') {
            return Err(CpkStemError::TrailingDot);
        }
        if text
            .get(text.len().saturating_sub(4)..)
            .is_some_and(|tail| tail.eq_ignore_ascii_case(".cpk"))
        {
            return Err(CpkStemError::CpkSuffix);
        }
        let base = text.split_once('.').map(|(base, _)| base).unwrap_or(text);
        if RESERVED.iter().any(|name| base.eq_ignore_ascii_case(name)) {
            return Err(CpkStemError::ReservedName {
                name: base.to_owned(),
            });
        }
        Ok(CpkStem(text.to_owned()))
    }

    /// The stem as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CpkStem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_stems_pass() {
        for text in ["4cc_99_test".to_owned(), "x".repeat(28), "a.b".to_owned()] {
            let stem = CpkStem::new(&text).unwrap();
            assert_eq!(stem.as_str(), text);
            assert_eq!(stem.to_string(), text);
        }
    }

    #[test]
    fn the_first_failing_rule_reports() {
        let long = "x".repeat(29);
        let cases: [(&str, CpkStemError); 11] = [
            ("", CpkStemError::Empty),
            (long.as_str(), CpkStemError::TooLong { len: 29 }),
            (
                "has space",
                CpkStemError::InvalidCharacter { character: ' ' },
            ),
            ("a/b", CpkStemError::InvalidCharacter { character: '/' }),
            ("a\\b", CpkStemError::InvalidCharacter { character: '\\' }),
            ("x.", CpkStemError::TrailingDot),
            ("x.CPK", CpkStemError::CpkSuffix),
            (
                "con",
                CpkStemError::ReservedName {
                    name: "con".to_owned(),
                },
            ),
            (
                "CON.x",
                CpkStemError::ReservedName {
                    name: "CON".to_owned(),
                },
            ),
            (
                "com1",
                CpkStemError::ReservedName {
                    name: "com1".to_owned(),
                },
            ),
            (
                "lpt9",
                CpkStemError::ReservedName {
                    name: "lpt9".to_owned(),
                },
            ),
        ];
        for (stem, expected) in cases {
            assert_eq!(CpkStem::new(stem), Err(expected), "{stem:?}");
        }
    }

    #[test]
    fn the_first_failing_rule_in_enum_order_reports() {
        // Each stem breaks two rules; the earlier one is the finding.
        let long = format!("{} b", "x".repeat(28));
        let cases: [(&str, CpkStemError); 4] = [
            (long.as_str(), CpkStemError::TooLong { len: 30 }),
            ("a b.", CpkStemError::InvalidCharacter { character: ' ' }),
            ("con.", CpkStemError::TrailingDot),
            ("con.cpk", CpkStemError::CpkSuffix),
        ];
        for (stem, expected) in cases {
            assert_eq!(CpkStem::new(stem), Err(expected), "{stem:?}");
        }
    }
}
