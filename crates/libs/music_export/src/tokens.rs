//! The token grammar of a condition or instruction field: whitespace-separated
//! words, `[bracketed text]` grouping spaced words into one token, a leading
//! `\` escaping a word and `\]` a bracket at a word's end.

/// Splits one `;`-field into tokens.
///
/// A word starting with `\` loses the backslash and is taken as is (so `\[x`
/// is the token `[x`). A word starting with `[` opens a group that runs to the
/// first word ending in `]` not preceded by `\`; the group's words are joined by
/// single spaces, a trailing `\]` inside it becoming `]`. A group that never
/// closes is the error.
pub(crate) fn tokenize(field: &str) -> Result<Vec<String>, UnclosedBracket> {
    let mut tokens = Vec::new();
    let mut words = field.split_whitespace();
    while let Some(word) = words.next() {
        if let Some(escaped) = word.strip_prefix('\\') {
            tokens.push(escaped.to_string());
        } else if let Some(opened) = word.strip_prefix('[') {
            tokens.push(group(opened, &mut words)?);
        } else {
            tokens.push(word.to_string());
        }
    }
    Ok(tokens)
}

/// A `[` group with no closing `]` on the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UnclosedBracket;

/// Reads a bracket group whose first word, `[` already stripped, is `first`.
fn group<'a>(
    first: &str,
    words: &mut impl Iterator<Item = &'a str>,
) -> Result<String, UnclosedBracket> {
    let mut text = String::new();
    let mut word = first;
    loop {
        if let Some(inner) = word.strip_suffix("\\]") {
            text.push_str(inner);
            text.push(']');
        } else if let Some(inner) = word.strip_suffix(']') {
            text.push_str(inner);
            return Ok(text);
        } else {
            text.push_str(word);
        }
        text.push(' ');
        word = words.next().ok_or(UnclosedBracket)?;
    }
}

/// Writes one token so that [`tokenize`] reads it back: bracketed when it holds
/// whitespace or is empty, escaped when it starts with `[` or `\`.
pub(crate) fn write_token(token: &str) -> String {
    if token.is_empty() {
        return "[]".to_string();
    }
    if !token.contains(char::is_whitespace) {
        return if token.starts_with('[') || token.starts_with('\\') {
            format!("\\{token}")
        } else {
            token.to_string()
        };
    }
    // Inside a group only a word's trailing `]` closes it, so that is the one
    // character to escape; the last word needs none, since the reader strips a
    // single `]` from the closing word and the group's own `]` follows it.
    let words: Vec<&str> = token.split(' ').collect();
    let mut written = String::from("[");
    for (i, word) in words.iter().enumerate() {
        if i > 0 {
            written.push(' ');
        }
        match word.strip_suffix(']') {
            Some(inner) if i + 1 < words.len() => {
                written.push_str(inner);
                written.push_str("\\]");
            }
            _ => written.push_str(word),
        }
    }
    written.push(']');
    written
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(field: &str) -> Vec<String> {
        tokenize(field).expect("closed brackets")
    }

    #[test]
    fn plain_words_split_on_any_whitespace() {
        assert_eq!(tokens("goals >= 2"), ["goals", ">=", "2"]);
        assert_eq!(tokens("  match   ro16\tfinal "), ["match", "ro16", "final"]);
        assert!(tokens("").is_empty());
    }

    #[test]
    fn brackets_group_spaced_words_into_one_token() {
        assert_eq!(
            tokens("special [Promotion Anthem]"),
            ["special", "Promotion Anthem"]
        );
        assert_eq!(
            tokens("mostgoals [Get Out] x"),
            ["mostgoals", "Get Out", "x"]
        );
        assert_eq!(tokens("special [One]"), ["special", "One"]);
        assert_eq!(tokens("special []"), ["special", ""]);
    }

    #[test]
    fn escapes_keep_brackets_literal() {
        assert_eq!(tokens("x \\[not a group"), ["x", "[not", "a", "group"]);
        assert_eq!(tokens("[a\\] b]"), ["a] b"]);
        assert_eq!(tokens("[a b\\] c]"), ["a b] c"]);
        assert_eq!(tokens("\\\\x"), ["\\x"]);
    }

    #[test]
    fn an_unclosed_group_is_refused() {
        assert_eq!(tokenize("special [never closed"), Err(UnclosedBracket));
        assert_eq!(tokenize("["), Err(UnclosedBracket));
    }

    #[test]
    fn written_tokens_read_back() {
        let cases = [
            ("ro16", "ro16"),
            ("Promotion Anthem", "[Promotion Anthem]"),
            ("", "[]"),
            ("[x", "\\[x"),
            ("\\x", "\\\\x"),
            ("a] b", "[a\\] b]"),
            ("a b]", "[a b]]"),
            ("a x\\]", "[a x\\]]"),
            ("x]", "x]"),
        ];
        for (token, written) in cases {
            assert_eq!(write_token(token), written, "{token:?}");
            assert_eq!(tokens(written), [token], "{written:?}");
        }
    }
}
