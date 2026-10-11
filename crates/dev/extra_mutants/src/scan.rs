//! Finds the mutants `cargo-mutants` never makes (worklog 5.0b): `if` guards dropped by
//! replacing the condition with `false`, and elements removed from the literal lists a
//! `matches!` or a separator method decides by.
//!
//! A mutant is a text patch, so positions come from `proc_macro2::Span`'s line and column
//! (columns count chars, not bytes) and are converted to byte offsets against the source.

use proc_macro2::{LineColumn, Span};
use syn::parse::{ParseStream, Parser};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Expr, ExprArray, ExprIf, ExprLit, ExprMacro, ExprMethodCall, ImplItemFn, Item,
    ItemFn, Lit, Pat, PatOr, Token, TraitItemFn,
};

/// The method names whose receiver or arguments decide by a literal list (worklog 5.0b).
const LIST_METHODS: &[&str] = &[
    "split",
    "rsplit",
    "split_terminator",
    "splitn",
    "split_once",
    "rsplit_once",
    "contains",
    "starts_with",
    "ends_with",
    "trim_matches",
    "trim_start_matches",
    "trim_end_matches",
    "strip_prefix",
    "strip_suffix",
    "find",
    "rfind",
    "matches",
];

/// One mutant: replacing `text[start..end]` with `replacement` must still parse. `line` is
/// 1-based and `kind` is `guard` or `literal`, as one row of the lister's output is built.
pub(crate) struct Mutant {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) kind: &'static str,
    pub(crate) line: usize,
    pub(crate) replacement: &'static str,
    pub(crate) description: String,
}

/// Every mutant in one source file's text, in source order.
pub(crate) fn scan_source(text: &str) -> Result<Vec<Mutant>, syn::Error> {
    // syn::parse_file strips a BOM and a shebang line before parsing; doing the same here
    // keeps the spans and this text the same length, so byte offsets line up.
    let content = strip_prelude(text);
    let file = syn::parse_file(content)?;
    let mut lister = Lister {
        text: content,
        line_starts: line_starts(content),
        fn_name: String::new(),
        mutants: Vec::new(),
    };
    lister.visit_file(&file);
    let skipped = text.len() - content.len();
    for mutant in &mut lister.mutants {
        mutant.start += skipped;
        mutant.end += skipped;
    }
    Ok(lister.mutants)
}

fn strip_prelude(text: &str) -> &str {
    let content = text.strip_prefix('\u{feff}').unwrap_or(text);
    if let Some(rest) = content.strip_prefix("#!")
        && !rest.trim_start().starts_with('[')
    {
        let end = content.find('\n').map_or(content.len(), |i| i + 1);
        return &content[end..];
    }
    content
}

/// The byte offset each 1-based line starts at, for line/column to byte conversion.
fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// The byte offset of a 1-based `line`, 0-based `column` position, `column` counting chars
/// as `proc_macro2::Span::start`/`end` report it.
fn byte_offset(text: &str, line_starts: &[usize], loc: LineColumn) -> usize {
    let base = line_starts[loc.line - 1];
    match text[base..].char_indices().nth(loc.column) {
        Some((offset, _)) => base + offset,
        None => text.len(),
    }
}

struct Lister<'a> {
    text: &'a str,
    line_starts: Vec<usize>,
    /// The function being visited, for the description. Empty at top level.
    fn_name: String,
    mutants: Vec<Mutant>,
}

impl Lister<'_> {
    fn start_of(&self, span: Span) -> usize {
        byte_offset(self.text, &self.line_starts, span.start())
    }

    fn end_of(&self, span: Span) -> usize {
        byte_offset(self.text, &self.line_starts, span.end())
    }

    /// The source text a byte range covers, whitespace collapsed so it can sit in a
    /// tab-separated line.
    fn snippet(&self, start: usize, end: usize) -> String {
        self.text[start..end]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn description(&self, change: &str, start: usize, end: usize) -> String {
        let removed = self.snippet(start, end);
        let place = if self.fn_name.is_empty() {
            "at top level".to_owned()
        } else {
            format!("in {}", self.fn_name)
        };
        format!("{place}: {change} | removed: {removed}")
    }

    fn push(
        &mut self,
        kind: &'static str,
        start: usize,
        end: usize,
        replacement: &'static str,
        change: &str,
        line: usize,
    ) {
        let description = self.description(change, start, end);
        self.mutants.push(Mutant {
            start,
            end,
            kind,
            line,
            replacement,
            description,
        });
    }

    /// One removal range per element of a comma- or `|`-separated list: the element and the
    /// separator after it, or the separator before it for a last element with none after.
    /// `pairs` is `(element span, separator span)` per element, in order.
    fn removal_ranges(&self, pairs: &[(Span, Option<Span>)]) -> Vec<(usize, usize)> {
        pairs
            .iter()
            .enumerate()
            .map(|(i, &(element, separator))| match separator {
                Some(separator) => (self.start_of(element), self.end_of(separator)),
                None => (
                    self.start_of(
                        pairs[i - 1]
                            .1
                            .expect("every element but the last is followed by a separator"),
                    ),
                    self.end_of(element),
                ),
            })
            .collect()
    }

    /// The mutants of a `matches!(expr, pat)` call: one per alternative of a top-level
    /// or-pattern. Anything that does not parse as `expr, pat` is not a `matches!` this
    /// tool understands and is skipped.
    fn matches_mutants(&mut self, mac: &syn::Macro) {
        let Ok(Pat::Or(PatOr { cases, .. })) = matches_args.parse2(mac.tokens.clone()) else {
            return;
        };
        if cases.len() < 2 {
            return;
        }
        let pairs: Vec<(Span, Option<Span>)> = cases
            .pairs()
            .map(|pair| {
                (
                    pair.value().span(),
                    pair.punct().map(|token| token.spans[0]),
                )
            })
            .collect();
        let ranges = self.removal_ranges(&pairs);
        for (start, end) in ranges {
            let line = self.line_of(start);
            self.push(
                "literal",
                start,
                end,
                "",
                "the matches! pattern loses an alternative",
                line,
            );
        }
    }

    /// The mutants of `[a, b, ...]` used by a list-deciding method call: one per element,
    /// when every element is a literal.
    fn array_mutants(&mut self, expr: &Expr, method: &str) {
        let mut expr = expr;
        while let Expr::Reference(reference) = expr {
            expr = &reference.expr;
        }
        let Expr::Array(ExprArray { elems, .. }) = expr else {
            return;
        };
        if elems.len() < 2 || !elems.iter().all(is_literal) {
            return;
        }
        let pairs: Vec<(Span, Option<Span>)> = elems
            .pairs()
            .map(|pair| {
                (
                    pair.value().span(),
                    pair.punct().map(|token| token.spans[0]),
                )
            })
            .collect();
        let ranges = self.removal_ranges(&pairs);
        for (start, end) in ranges {
            let line = self.line_of(start);
            let change = format!("the {method}() list loses an element");
            self.push("literal", start, end, "", &change, line);
        }
    }

    fn line_of(&self, byte: usize) -> usize {
        self.line_starts.partition_point(|&start| start <= byte)
    }
}

impl<'ast> Visit<'ast> for Lister<'_> {
    fn visit_item(&mut self, item: &'ast Item) {
        // #[cfg(test)] and #[test] mark test code, whose mutants nobody wants counted.
        if has_test_attr(item_attrs(item)) {
            return;
        }
        visit::visit_item(self, item);
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        let outer = std::mem::replace(&mut self.fn_name, item.sig.ident.to_string());
        visit::visit_item_fn(self, item);
        self.fn_name = outer;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        if has_test_attr(&item.attrs) {
            return;
        }
        let outer = std::mem::replace(&mut self.fn_name, item.sig.ident.to_string());
        visit::visit_impl_item_fn(self, item);
        self.fn_name = outer;
    }

    fn visit_trait_item_fn(&mut self, item: &'ast TraitItemFn) {
        if has_test_attr(&item.attrs) {
            return;
        }
        let outer = std::mem::replace(&mut self.fn_name, item.sig.ident.to_string());
        visit::visit_trait_item_fn(self, item);
        self.fn_name = outer;
    }

    fn visit_item_const(&mut self, _item: &'ast syn::ItemConst) {
        // A const's declared length makes every element removal unviable.
    }

    fn visit_item_static(&mut self, _item: &'ast syn::ItemStatic) {
        // Same as const: a declared length the compiler checks.
    }

    fn visit_expr_if(&mut self, expr: &'ast ExprIf) {
        if expr.else_branch.is_none()
            && !has_let(expr.cond.as_ref())
            && has_early_exit(&expr.then_branch)
        {
            let start = self.start_of(expr.cond.span());
            let end = self.end_of(expr.cond.span());
            let line = self.line_of(start);
            self.push(
                "guard",
                start,
                end,
                "false",
                "the if guard becomes false",
                line,
            );
        }
        visit::visit_expr_if(self, expr);
    }

    fn visit_expr_macro(&mut self, expr: &'ast ExprMacro) {
        if expr.mac.path.is_ident("matches") {
            self.matches_mutants(&expr.mac);
        }
        visit::visit_expr_macro(self, expr);
    }

    fn visit_expr_method_call(&mut self, expr: &'ast ExprMethodCall) {
        let method = expr.method.to_string();
        if LIST_METHODS.contains(&method.as_str()) {
            self.array_mutants(&expr.receiver, &method);
            for arg in &expr.args {
                self.array_mutants(arg, &method);
            }
        }
        visit::visit_expr_method_call(self, expr);
    }
}

/// `matches!(<expr>, <pattern> [if <guard>] [,])`; a `matches!` that does not parse this
/// way is skipped by the caller.
fn matches_args(input: ParseStream) -> syn::Result<Pat> {
    input.parse::<Expr>()?;
    input.parse::<Token![,]>()?;
    let pat = Pat::parse_multi_with_leading_vert(input)?;
    if input.peek(Token![if]) {
        input.parse::<Token![if]>()?;
        input.parse::<Expr>()?;
    }
    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
    }
    if !input.is_empty() {
        return Err(input.error("unexpected tokens after the pattern"));
    }
    Ok(pat)
}

/// An `if` whose condition contains `let` (`if let`, a let-chain) is not a plain guard.
fn has_let(cond: &Expr) -> bool {
    struct Finder(bool);
    impl<'ast> Visit<'ast> for Finder {
        fn visit_expr_let(&mut self, _expr: &'ast syn::ExprLet) {
            self.0 = true;
        }
    }
    let mut finder = Finder(false);
    finder.visit_expr(cond);
    finder.0
}

/// Whether a block exits early: `return`, `continue` or `break` anywhere but inside a
/// closure, an `async`/`const` block or a nested item, which have their own control flow.
fn has_early_exit(block: &syn::Block) -> bool {
    struct Finder(bool);
    impl<'ast> Visit<'ast> for Finder {
        fn visit_expr_return(&mut self, _expr: &'ast syn::ExprReturn) {
            self.0 = true;
        }
        fn visit_expr_continue(&mut self, _expr: &'ast syn::ExprContinue) {
            self.0 = true;
        }
        fn visit_expr_break(&mut self, _expr: &'ast syn::ExprBreak) {
            self.0 = true;
        }
        fn visit_expr_closure(&mut self, _expr: &'ast syn::ExprClosure) {}
        fn visit_expr_async(&mut self, _expr: &'ast syn::ExprAsync) {}
        fn visit_expr_const(&mut self, _expr: &'ast syn::ExprConst) {}
        fn visit_item(&mut self, _item: &'ast Item) {}
    }
    let mut finder = Finder(false);
    finder.visit_block(block);
    finder.0
}

/// An element of a literal list: a string, char, byte or integer literal.
fn is_literal(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Lit(ExprLit {
            lit: Lit::Str(_) | Lit::Char(_) | Lit::Byte(_) | Lit::Int(_),
            ..
        })
    )
}

fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        Item::Verbatim(_) => &[],
        _ => &[],
    }
}

/// `#[test]` or `#[cfg(test)]` (exactly `test`: `cfg(all(test, …))` keeps the code scanned).
fn has_test_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("test") || {
            if !attr.path().is_ident("cfg") {
                return false;
            }
            let mut is_test = false;
            attr.parse_nested_meta(|meta| {
                is_test = meta.path.is_ident("test") && meta.input.is_empty();
                Ok(())
            })
            .is_ok()
                && is_test
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(text: &str) -> Vec<Mutant> {
        scan_source(text).expect("the test sources parse")
    }

    /// The source with a mutant applied, which must still parse.
    fn patch(text: &str, mutant: &Mutant) -> String {
        format!(
            "{}{}{}",
            &text[..mutant.start],
            mutant.replacement,
            &text[mutant.end..]
        )
    }

    fn assert_patches_parse(text: &str, mutants: &[Mutant]) {
        for mutant in mutants {
            let patched = patch(text, mutant);
            syn::parse_file(&patched).unwrap_or_else(|error| {
                panic!("the patched source does not parse: {error}\n{patched}")
            });
        }
    }

    #[test]
    fn drops_a_guard() {
        let text = "fn f(x: bool) -> u32 {\n    if x {\n        return 1;\n    }\n    0\n}\n";
        let mutants = scan(text);
        assert_eq!(mutants.len(), 1);
        let mutant = &mutants[0];
        assert_eq!(mutant.kind, "guard");
        assert_eq!(&text[mutant.start..mutant.end], "x");
        assert_eq!(mutant.replacement, "false");
        assert_eq!(mutant.line, 2);
        assert!(mutant.description.starts_with("in f:"));
        assert_patches_parse(text, &mutants);
    }

    #[test]
    fn drops_break_and_continue_guards() {
        let text = "fn f(xs: &[u32]) {\n    for x in xs {\n        if *x == 0 {\n            break;\n        }\n        if *x == 1 {\n            continue;\n        }\n    }\n}\n";
        let mutants = scan(text);
        assert_eq!(mutants.len(), 2);
        assert!(mutants.iter().all(|m| m.kind == "guard"));
        assert_eq!(&text[mutants[0].start..mutants[0].end], "*x == 0");
        assert_eq!(&text[mutants[1].start..mutants[1].end], "*x == 1");
        assert_patches_parse(text, &mutants);
    }

    #[test]
    fn removes_matches_alternatives() {
        let text = "fn f(s: &str) -> bool {\n    matches!(s, \"a\" | \"b\" | \"c\")\n}\n";
        let mutants = scan(text);
        assert_eq!(mutants.len(), 3);
        assert!(mutants.iter().all(|m| m.kind == "literal"));
        assert_eq!(&text[mutants[0].start..mutants[0].end], "\"a\" |");
        assert_eq!(&text[mutants[1].start..mutants[1].end], "\"b\" |");
        assert_eq!(&text[mutants[2].start..mutants[2].end], "| \"c\"");
        assert_patches_parse(text, &mutants);
    }

    #[test]
    fn removes_matches_alternatives_with_a_guard() {
        let text = "fn f(s: &str) -> bool {\n    matches!(s, \"a\" | \"b\" if s.len() > 1)\n}\n";
        let mutants = scan(text);
        assert_eq!(mutants.len(), 2);
        assert_eq!(&text[mutants[0].start..mutants[0].end], "\"a\" |");
        assert_eq!(&text[mutants[1].start..mutants[1].end], "| \"b\"");
        assert_patches_parse(text, &mutants);
    }

    #[test]
    fn removes_array_elements() {
        let text = "fn f(s: &str) -> Vec<&str> {\n    s.split(&[',', ';', '\\t'])\n}\n";
        let mutants = scan(text);
        assert_eq!(mutants.len(), 3);
        assert!(mutants.iter().all(|m| m.kind == "literal"));
        assert_eq!(&text[mutants[0].start..mutants[0].end], "',',");
        assert_eq!(&text[mutants[1].start..mutants[1].end], "';',");
        assert_eq!(&text[mutants[2].start..mutants[2].end], ", '\\t'");
        assert_patches_parse(text, &mutants);
    }

    #[test]
    fn removes_array_elements_of_the_receiver() {
        let text = "fn f(c: char) -> bool {\n    [',', ';'].contains(&c)\n}\n";
        let mutants = scan(text);
        assert_eq!(mutants.len(), 2);
        assert_eq!(&text[mutants[0].start..mutants[0].end], "',',");
        assert_eq!(&text[mutants[1].start..mutants[1].end], ", ';'");
        assert_patches_parse(text, &mutants);
    }

    #[test]
    fn a_guard_and_its_matches_both_list() {
        let text = "fn f(s: &str) -> bool {\n    if matches!(s, \"a\" | \"b\") {\n        return true;\n    }\n    false\n}\n";
        let mutants = scan(text);
        assert_eq!(mutants.len(), 3);
        assert_eq!(mutants[0].kind, "guard");
        assert_eq!(
            &text[mutants[0].start..mutants[0].end],
            "matches!(s, \"a\" | \"b\")"
        );
        assert!(mutants[1..].iter().all(|m| m.kind == "literal"));
        assert_patches_parse(text, &mutants);
    }

    #[test]
    fn an_if_with_else_is_not_a_guard() {
        let text = "fn f(x: bool) -> u32 {\n    if x {\n        return 1;\n    } else {\n        0\n    }\n}\n";
        assert!(scan(text).is_empty());
    }

    #[test]
    fn an_else_if_is_its_own_guard() {
        let text = "fn f(x: bool, y: bool) -> u32 {\n    if x {\n        return 1;\n    } else if y {\n        return 2;\n    }\n    0\n}\n";
        let mutants = scan(text);
        assert_eq!(mutants.len(), 1);
        assert_eq!(&text[mutants[0].start..mutants[0].end], "y");
        assert_patches_parse(text, &mutants);
    }

    #[test]
    fn an_if_let_is_not_a_guard() {
        let text = "fn f(x: Option<u32>) -> u32 {\n    if let Some(v) = x {\n        return v;\n    }\n    0\n}\n";
        assert!(scan(text).is_empty());
    }

    #[test]
    fn a_let_chain_is_not_a_guard() {
        let text = "fn f(a: bool, x: Option<u32>) -> u32 {\n    if a && let Some(v) = x {\n        return v;\n    }\n    0\n}\n";
        assert!(scan(text).is_empty());
    }

    #[test]
    fn a_return_in_a_closure_is_not_an_early_exit() {
        let text = "fn f(x: bool) {\n    if x {\n        let g = || {\n            return;\n        };\n    }\n}\n";
        assert!(scan(text).is_empty());
    }

    #[test]
    fn a_return_in_an_async_block_is_not_an_early_exit() {
        let text = "fn f(x: bool) {\n    if x {\n        let g = async {\n            return;\n        };\n    }\n}\n";
        assert!(scan(text).is_empty());
    }

    #[test]
    fn a_return_in_a_nested_item_is_not_an_early_exit() {
        let text = "fn f(x: bool) {\n    if x {\n        fn inner() -> u32 {\n            return 0;\n        }\n    }\n}\n";
        assert!(scan(text).is_empty());
    }

    #[test]
    fn a_cfg_test_module_is_skipped() {
        let text = "fn f(x: bool) {}\n\n#[cfg(test)]\nmod tests {\n    fn g(x: bool) {\n        if x {\n            return;\n        }\n    }\n}\n";
        assert!(scan(text).is_empty());
    }

    #[test]
    fn a_test_fn_is_skipped() {
        let text = "#[test]\nfn t() {\n    let x = false;\n    if x {\n        return;\n    }\n}\n";
        assert!(scan(text).is_empty());
    }

    #[test]
    fn a_single_alternative_matches_gives_nothing() {
        let text = "fn f(s: &str) -> bool {\n    matches!(s, \"a\")\n}\n";
        assert!(scan(text).is_empty());
    }

    #[test]
    fn a_single_element_array_gives_nothing() {
        let text = "fn f(s: &str) -> Vec<&str> {\n    s.split(&[','])\n}\n";
        assert!(scan(text).is_empty());
    }

    #[test]
    fn an_unlisted_method_gives_nothing() {
        let text = "fn f(s: &str) -> bool {\n    s.chars().eq(['a', 'b'])\n}\n";
        assert!(scan(text).is_empty());
    }

    #[test]
    fn byte_offsets_ignore_a_multibyte_char_on_the_line() {
        // caf\u{e9}: the \u{e9} is two bytes, one char; a column counted in bytes would
        // land the mutant one byte early.
        let text = "fn f(x: bool) {\n    if /* caf\u{e9} */ x {\n        return;\n    }\n}\n";
        let mutants = scan(text);
        assert_eq!(&text[mutants[0].start..mutants[0].end], "x");
        assert_eq!(mutants[0].line, 2);
        assert_patches_parse(text, &mutants);
    }

    #[test]
    fn the_second_file_in_a_process_gets_its_own_offsets() {
        // A first parse fills the proc-macro2 source map; the second file's spans must
        // still map into its own text.
        let first = "fn a(long_condition_name: bool) {\n    if long_condition_name {\n        return;\n    }\n}\n";
        scan(first);
        let text = "fn f(x: bool) {\n    if x {\n        return;\n    }\n}\n";
        let mutants = scan(text);
        assert_eq!(mutants.len(), 1);
        assert_eq!(&text[mutants[0].start..mutants[0].end], "x");
        assert_eq!(mutants[0].line, 2);
        assert_patches_parse(text, &mutants);
    }
}
