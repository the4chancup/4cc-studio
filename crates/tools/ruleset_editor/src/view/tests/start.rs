//! The start page: its entries in order (RE-GUI-01), each template and
//! Blank opening clean on General and saving back equal (RE-GUI-02),
//! and the refused-open listing (RE-FILE-03).

use super::*;

/// The buttons in `labels` appear in `expected`'s order (a subsequence,
/// not necessarily contiguous).
fn assert_order(labels: &[String], expected: &[&str]) {
    let mut at = 0;
    for entry in expected {
        let found = labels[at..]
            .iter()
            .position(|label| label == entry)
            .unwrap_or_else(|| panic!("{entry:?} not drawn after index {at}: {labels:?}"));
        at += found + 1;
    }
}

/// The rail's section labels drawn for `view` (a `⚠` shows inside the
/// label), Custom checks only when the document has it.
fn rail_labels(nodes: &[accesskit::Node]) -> Vec<String> {
    Section::RAIL
        .iter()
        .filter_map(|section| {
            let label = section.label();
            nodes
                .iter()
                .find(|node| node.label().is_some_and(|text| text.starts_with(label)))
                .map(|node| node.label().expect("a label").to_string())
        })
        .collect()
}

// RE-GUI-01
#[test]
fn the_start_page_offers_its_entries_in_order() {
    let (ctx, _events, mut window, mut view) = fresh();
    let frame = window.draw(&mut view, &ctx);
    let labels = labels(&frame.nodes);
    assert_order(
        &labels,
        &[
            "New from template",
            "Autumn 26 (the official rules)",
            "Spring 26",
            "VGL27",
            "VGL26",
            "VTL11",
            "VTL10",
            "DBC7 (Divegrass Before Christmas 7)",
            "Blank: one tier, no brackets, the universal rules on",
            "Sensible",
            "Weird",
            "Crazy",
            "Open file…",
            "Recent files",
        ],
    );
    assert!(drawn(&frame.nodes, "Random"));
}

// RE-GUI-01
#[test]
fn the_recent_files_list_the_later_open_first_and_skip_a_deleted_file() {
    let dir = crate::test_support::scratch("recent_files");
    let first = write_file(&dir, "first.rhai", &official_text());
    let second = write_file(&dir, "second.rhai", &official_text());
    let saved = dir.join("saved.rhai");
    let (ctx, _events) = crate::test_support::context();

    // Two files opened in turn, then the tool restarted: the later
    // heads the list.
    let mut view = opened_view(&ctx, &first);
    open_path(&mut view, &ctx, &second);
    let mut restarted = RulesetView::new();
    let mut window = Headless::new();
    let frame = window.draw(&mut restarted, &ctx);
    assert_order(&labels(&frame.nodes), &["second.rhai", "first.rhai"]);

    // One opened and one saved, restarted: the saved heads it.
    let mut view = opened_view(&ctx, &first);
    assert!(save_to(&mut view, &ctx, &saved));
    let mut restarted = RulesetView::new();
    let frame = window.draw(&mut restarted, &ctx);
    assert_order(&labels(&frame.nodes), &["saved.rhai", "first.rhai"]);

    // A listed file deleted is no longer listed.
    std::fs::remove_file(&saved).unwrap();
    let frame = window.draw(&mut restarted, &ctx);
    assert!(
        !drawn(&frame.nodes, "saved.rhai"),
        "the deleted file is filtered"
    );
    assert!(drawn(&frame.nodes, "first.rhai"));
}

/// `index`'s template opened through its button: the document's name,
/// General drawn, the rail's sections in order with no problem mark,
/// and `save_to` writing a file `Rules::load` reads back equal.
fn template_opens_clean(index: usize, test: &str) {
    let templates = crate::view::start::templates();
    let template = &templates[index];
    let dir = crate::test_support::scratch(test);
    let (ctx, _events, mut window, mut view) = fresh();
    let frame = window.draw(&mut view, &ctx);
    click_prefix(
        &mut window,
        &frame.nodes,
        &template.ruleset.name,
        &mut view,
        &ctx,
    );

    let document = view.document.as_ref().expect("a document opened");
    assert_eq!(document.ruleset.name, template.ruleset.name);
    assert_eq!(view.section, Section::General);
    let frame = window.draw(&mut view, &ctx);
    let expected: Vec<String> = Section::RAIL
        .iter()
        .filter(|section| **section != Section::CustomChecks || template.custom_checks.is_some())
        .map(|section| section.label().to_string())
        .collect();
    assert_eq!(rail_labels(&frame.nodes), expected, "the rail's sections");
    assert!(
        !labels(&frame.nodes).iter().any(|label| label.contains('⚠')),
        "no rail entry carries a problem mark"
    );
    assert!(
        drawn(
            &frame.nodes,
            "✔ Valid: the Save editor and the Team creator can use it"
        ),
        "the bottom bar's valid line"
    );

    let path = dir.join("saved.rhai");
    assert!(save_to(&mut view, &ctx, &path));
    let text = std::fs::read_to_string(&path).unwrap();
    let rules = aatf::Rules::load(&text).expect("the saved file loads");
    assert_eq!(rules.ruleset(), &template.ruleset);
}

// RE-GUI-02
#[test]
fn each_template_opens_clean_and_saves_back_equal() {
    for index in 0..crate::view::start::templates().len() {
        template_opens_clean(index, &format!("template_{index}"));
    }
}

// RE-GUI-02
#[test]
fn blank_opens_as_new_ruleset_and_saves_back_equal() {
    let dir = crate::test_support::scratch("template_blank");
    let (ctx, _events, mut window, mut view) = fresh();
    let frame = window.draw(&mut view, &ctx);
    click_labeled(
        &mut window,
        &frame.nodes,
        "Blank: one tier, no brackets, the universal rules on",
        &mut view,
        &ctx,
    );

    let document = view.document.as_ref().expect("a document opened");
    assert_eq!(document.ruleset.name, "New ruleset");
    assert!(document.path.is_none(), "unsaved");
    assert_eq!(view.section, Section::General);
    let frame = window.draw(&mut view, &ctx);
    let expected: Vec<String> = Section::RAIL
        .iter()
        .filter(|section| **section != Section::CustomChecks)
        .map(|section| section.label().to_string())
        .collect();
    assert_eq!(rail_labels(&frame.nodes), expected);
    assert!(
        !labels(&frame.nodes).iter().any(|label| label.contains('⚠')),
        "no rail entry carries a problem mark"
    );

    let path = dir.join("saved.rhai");
    assert!(save_to(&mut view, &ctx, &path));
    let text = std::fs::read_to_string(&path).unwrap();
    let rules = aatf::Rules::load(&text).expect("the saved file loads");
    assert_eq!(rules.ruleset(), &crate::document::blank());
}

/// `text` opened as `rules.rhai` under a fresh scratch folder: it is not
/// opened, and the start page's labels as drawn (the file's name, then
/// one line per finding).
fn refused_open_labels(test: &str, text: &str) -> Vec<String> {
    let dir = crate::test_support::scratch(test);
    let file = write_file(&dir, "rules.rhai", text);
    let (ctx, _events, mut window, mut view) = fresh();
    open_path(&mut view, &ctx, &file);
    assert!(view.document.is_none(), "not opened in the forms");
    let frame = window.draw(&mut view, &ctx);
    labels(&frame.nodes)
}

/// `labels` (the refused-open listing) holds a finding matching
/// `predicate`.
fn assert_finding(labels: &[String], predicate: impl Fn(&str) -> bool) {
    assert!(
        labels.iter().any(|label| predicate(label)),
        "the finding names it: {labels:?}"
    );
}

// RE-FILE-03
#[test]
fn a_parse_error_is_refused_with_its_line_and_column() {
    let labels = refused_open_labels("open_parse", &crate::test_support::parse_error_file());
    assert_finding(&labels, |label| label == "Could not open rules.rhai:");
    assert_finding(&labels, |label| label.starts_with("line=12, column="));
}

// RE-FILE-03
#[test]
fn a_schema_error_is_refused_with_its_key_path() {
    let labels = refused_open_labels("open_schema", &crate::test_support::schema_error_file());
    assert_finding(&labels, |label| label.contains("tiers[2].free.tricks"));
}

// RE-FILE-03
#[test]
fn a_duplicate_tier_id_is_refused_with_its_key_path() {
    let labels = refused_open_labels("open_duplicate", &crate::test_support::duplicate_id_file());
    assert_finding(&labels, |label| {
        label.contains("tiers[1]") && label.contains("gold")
    });
}
