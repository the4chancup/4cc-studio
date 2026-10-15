//! `check-aatf`: an AATF ruleset run over the save's cup teams, the violations as finding
//! lines, and the rules file's selection (`--rules`, the `aatf_rules` setting, the built-in
//! official ruleset) or refusal when it does not load.

use std::fs;
use std::path::Path;

use pes_savefile::EditFile;
use pes_version::PesVersion;

use crate::TEST_SALT;
use crate::common::Sandbox;

/// The `rulesets/<ruleset>.rhai` rules file, assembled the way `aatf`'s `file::assemble`
/// builds a data block that has no CUSTOM CHECKS section: the block trimmed, the current
/// logic trimmed, then the banner line and the empty `custom_checks` stub. `autumn26` is
/// the block `Rules::official` embeds.
fn rules_file(ruleset: &str) -> String {
    let aatf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libs/aatf");
    let data = fs::read_to_string(aatf.join(format!("rulesets/{ruleset}.rhai"))).unwrap();
    let logic = fs::read_to_string(aatf.join("logic/logic.rhai")).unwrap();
    format!(
        "{}\n\n{}\n\n{}\nfn custom_checks(team) {{ [] }}\n",
        data.trim_end(),
        logic.trim_end(),
        "// ==== CUSTOM CHECKS: optional, hand-written; the Ruleset editor keeps this section as it is ===="
    )
}

/// The VGL27 violation team 701 of the PES 21 fixture save carries and the official ruleset
/// cannot produce (it has no `fielded_at_a`).
const FIELDED_NOT_A: &str =
    "Error aatf_violation (team=701, player=70101, rule=FieldedNotA, preset=1, position=GK)";

/// `check-aatf <save>` with the given extra arguments.
fn check(sandbox: &Sandbox, save: &str, extra: &[&str]) -> crate::common::Run {
    let save = sandbox.arg(save);
    let mut args = vec!["check-aatf", save.as_str()];
    args.extend_from_slice(extra);
    sandbox.run(&args)
}

// SE-AATF-01
#[test]
fn a_teams_violations_print_in_order_and_an_error_exits_1() {
    let sandbox = Sandbox::new("aatf_errors");
    sandbox.fixture(PesVersion::Pes15, "EDIT");

    let run = check(&sandbox, "EDIT", &["--teams", "718"]);
    assert_eq!(run.exit_code(), 1, "{:?}", run.messages());
    assert_eq!(
        run.messages(),
        [
            "Error aatf_violation (team=718, player=71802, rule=Height, height=185)",
            "Error aatf_violation (team=718, player=71803, rule=Height, height=185)",
            "Error aatf_violation (team=718, player=71820, rule=Height, height=185)",
            "Error aatf_violation (team=718, player=71822, rule=Height, height=185)",
            "Error aatf_violation (team=718, player=71823, rule=Height, height=185)",
            "Error aatf_violation (team=718, rule=TierCount, tier=gold, count=2, expected=1)",
            "Error aatf_violation (team=718, rule=TierCount, tier=bronze, count=0, expected=2)",
            "Error aatf_violation (team=718, rule=TierCount, tier=regular, count=19, expected=18)",
            "Error aatf_violation (team=718, rule=BracketQuota, system=red, bracket=tall, count=5, quota=10)",
        ]
    );
}

// SE-AATF-01
#[test]
fn a_warning_only_team_exits_0_and_a_clean_team_prints_nothing() {
    let sandbox = Sandbox::new("aatf_warning_legal");
    sandbox.fixture(PesVersion::Pes15, "EDIT");
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/aatf");
    for (file, expected) in [
        (
            "official_warning.toml",
            vec![
                "Warning aatf_violation (team=714, player=71401, rule=Special, label=Goalkeepers below giant, text=GK heights cannot be 194cm or more)",
            ],
        ),
        ("official_legal.toml", Vec::new()),
    ] {
        let run = sandbox.run(&[
            "import-toml",
            fixtures.join(file).to_str().unwrap(),
            "--base",
            &sandbox.arg("EDIT"),
            "--out",
            &sandbox.arg("OUT"),
        ]);
        assert_eq!(run.exit_code(), 0, "{file}: {:?}", run.messages());
        let run = check(&sandbox, "OUT", &["--teams", "714"]);
        assert_eq!(run.exit_code(), 0, "{file}");
        assert_eq!(run.messages(), expected, "{file}");
    }
}

// SE-AATF-02
#[test]
fn teams_limits_the_check_to_the_named_teams_in_id_order() {
    let sandbox = Sandbox::new("aatf_teams");
    sandbox.fixture(PesVersion::Pes15, "EDIT");

    let run = check(&sandbox, "EDIT", &["--teams", "718"]);
    assert_eq!(run.exit_code(), 1);
    let messages = run.messages();
    assert!(
        messages.iter().all(|line| line.contains("team=718")),
        "{messages:?}"
    );

    let run = check(&sandbox, "EDIT", &["--teams", "704,718"]);
    assert_eq!(run.exit_code(), 1);
    let messages = run.messages();
    assert_eq!(
        messages
            .iter()
            .filter(|line| line.contains("team=704"))
            .count(),
        31,
        "{messages:?}"
    );
    let last_704 = messages
        .iter()
        .rposition(|line| line.contains("team=704"))
        .unwrap();
    let first_718 = messages
        .iter()
        .position(|line| line.contains("team=718"))
        .unwrap();
    assert!(last_704 < first_718, "704's lines come first: {messages:?}");
}

// SE-AATF-02
#[test]
fn a_teams_id_the_save_lacks_is_refused() {
    let sandbox = Sandbox::new("aatf_no_team");
    sandbox.fixture(PesVersion::Pes15, "EDIT");
    check(&sandbox, "EDIT", &["--teams", "9999"]).assert_refused(2, &["9999"]);
    // A known id beside the missing one: the refusal stands and emits nothing.
    check(&sandbox, "EDIT", &["--teams", "718,9999"]).assert_refused(2, &["9999"]);
}

// SE-AATF-03
#[test]
fn a_rules_file_that_fails_to_load_is_refused_with_its_position() {
    let sandbox = Sandbox::new("aatf_broken_rules");
    sandbox.fixture(PesVersion::Pes21, "EDIT");
    fs::write(
        sandbox.path("broken.rhai"),
        "const RULESET = #{ name: \"x\",\n",
    )
    .unwrap();

    let run = check(
        &sandbox,
        "EDIT",
        &["--teams", "701", "--rules", &sandbox.arg("broken.rhai")],
    );
    run.assert_refused(2, &["broken.rhai"]);
    let error = match &run.result {
        Err(error) => error.to_string(),
        Ok(code) => panic!("not refused: exit code {code}"),
    };
    // `LoadError::Parse` displays `{line}:{column}: <message>` after the file's name.
    let mut position = error
        .split("broken.rhai: ")
        .nth(1)
        .unwrap_or_else(|| panic!("no position after the file's name: {error}"))
        .split(':');
    let line = position.next().unwrap().trim().parse::<usize>();
    let column = position.next().unwrap().trim().parse::<usize>();
    assert!(line.is_ok() && column.is_ok(), "a line and column: {error}");
    assert!(run.events.is_empty(), "a refused command emits nothing");
}

// SE-AATF-03
#[test]
fn rules_and_the_aatf_rules_setting_pick_the_rules_file() {
    let sandbox = Sandbox::new("aatf_vgl27");
    sandbox.fixture(PesVersion::Pes21, "EDIT");
    let rules = sandbox.path("vgl27.rhai");
    fs::write(&rules, rules_file("vgl27")).unwrap();

    let arg = rules.to_str().unwrap().to_owned();
    let run = check(
        &sandbox,
        "EDIT",
        &["--teams", "701", "--rules", arg.as_str()],
    );
    assert_eq!(run.exit_code(), 1, "{:?}", run.messages());
    assert!(
        run.messages().iter().any(|line| line == FIELDED_NOT_A),
        "{:?}",
        run.messages()
    );

    // A TOML literal string: the Windows path's backslashes need no escaping.
    let settings = format!("[save-editor]\naatf_rules = '{}'\n", rules.display());
    let run = sandbox.run_with_settings(
        &settings,
        &["check-aatf", &sandbox.arg("EDIT"), "--teams", "701"],
    );
    assert_eq!(run.exit_code(), 1, "{:?}", run.messages());
    assert!(
        run.messages().iter().any(|line| line == FIELDED_NOT_A),
        "{:?}",
        run.messages()
    );

    let run = check(&sandbox, "EDIT", &["--teams", "701"]);
    assert!(
        !run.messages()
            .iter()
            .any(|line| line.contains("rule=FieldedNotA")),
        "the official ruleset has no fielded_at_a: {:?}",
        run.messages()
    );
}

// SE-AATF-05
#[test]
fn without_teams_every_cup_team_is_checked_in_id_order() {
    let sandbox = Sandbox::new("aatf_default_teams");
    // The fixture holds cup teams alone: team 703's id moved outside the cup range makes
    // the team outside 701 to 920, and it breaks the same rules (`--teams 925` flags it)
    // but the default selection leaves it out.
    let mut file = EditFile::from_bytes(&pes_savefile::fixture_save(PesVersion::Pes15)).unwrap();
    file.team_mut(703).unwrap().id = 925;
    // Swapping 704's and 718's ids (through the free id 921) makes record order disagree
    // with id order: the record at 704's place now carries 718, the one at 718's place 704.
    // Both still break the same rules, as the assertions below check.
    file.team_mut(704).unwrap().id = 921;
    file.team_mut(718).unwrap().id = 704;
    file.team_mut(921).unwrap().id = 718;
    fs::write(sandbox.path("EDIT"), file.to_bytes(&TEST_SALT).unwrap()).unwrap();

    let run = check(&sandbox, "EDIT", &["--teams", "925"]);
    assert!(
        run.messages().iter().any(|line| line.contains("team=925")),
        "925 breaks the same rules: {:?}",
        run.messages()
    );

    let run = check(&sandbox, "EDIT", &[]);
    assert_eq!(run.exit_code(), 1);
    let ids: Vec<u32> = run
        .messages()
        .iter()
        .filter_map(|line| {
            line.split("team=")
                .nth(1)?
                .split(|c: char| !c.is_ascii_digit())
                .next()?
                .parse()
                .ok()
        })
        .collect();
    assert!(
        ids.iter().all(|id| (701..=920).contains(id)),
        "an out-of-range team printed: {ids:?}"
    );
    assert!(
        ids.contains(&704) && ids.contains(&718),
        "SE-AATF-02's two teams print: {ids:?}"
    );
    let mut teams = ids.clone();
    teams.dedup();
    assert!(
        teams.windows(2).all(|pair| pair[0] < pair[1]),
        "team id order: {teams:?}"
    );
}

// SE-AATF-06
#[test]
fn a_check_that_fails_on_one_team_does_not_stop_the_rest() {
    let sandbox = Sandbox::new("aatf_check_failed_continues");
    sandbox.fixture(PesVersion::Pes15, "EDIT");
    let file = EditFile::load(&sandbox.path("EDIT")).unwrap();
    // The fixture's one captain-19 team is 743; 744's is 22, so the custom check below
    // fails on 743 alone (the sample team's captain is 10: the file still loads).
    assert_eq!(file.team(743).unwrap().tactics.set_pieces.captain, 19);
    assert_eq!(file.team(744).unwrap().tactics.set_pieces.captain, 22);
    fs::write(
        sandbox.path("throw19.rhai"),
        rules_file("autumn26").replacen(
            "fn custom_checks(team) { [] }",
            "fn custom_checks(team) { if team.captain == 19 { throw \"boom\" } [] }",
            1,
        ),
    )
    .unwrap();

    let run = check(
        &sandbox,
        "EDIT",
        &[
            "--teams",
            "743,744",
            "--rules",
            &sandbox.arg("throw19.rhai"),
        ],
    );
    assert_eq!(run.exit_code(), 1, "{:?}", run.messages());
    let failed = run
        .messages()
        .iter()
        .position(|line| line.starts_with("Error aatf_check_failed (team=743,"))
        .unwrap_or_else(|| panic!("743's check failed line: {:?}", run.messages()));
    assert!(
        run.messages()[failed].contains("boom"),
        "{:?}",
        run.messages()
    );
    assert!(
        run.messages()
            .iter()
            .any(|line| line.contains("aatf_violation (team=744,")),
        "744's violations still print: {:?}",
        run.messages()
    );
}

// SE-AATF-07
#[test]
fn rules_wins_over_the_aatf_rules_setting() {
    let sandbox = Sandbox::new("aatf_rules_flag_wins");
    sandbox.fixture(PesVersion::Pes21, "EDIT");
    fs::write(sandbox.path("vgl27.rhai"), rules_file("vgl27")).unwrap();
    fs::write(sandbox.path("official.rhai"), rules_file("autumn26")).unwrap();

    // The setting names VGL27; --rules names the official file and is the one checked.
    let settings = format!(
        "[save-editor]\naatf_rules = '{}'\n",
        sandbox.path("vgl27.rhai").display()
    );
    let run = sandbox.run_with_settings(
        &settings,
        &[
            "check-aatf",
            &sandbox.arg("EDIT"),
            "--teams",
            "701",
            "--rules",
            &sandbox.arg("official.rhai"),
        ],
    );
    // A run that checked (0 or 1), not a refused one: a refusal prints no rule line either.
    assert!(run.exit_code() < 2, "{:?}", run.messages());
    assert!(
        !run.messages()
            .iter()
            .any(|line| line.contains("rule=FieldedNotA")),
        "the VGL27 rule the official ruleset does not have: {:?}",
        run.messages()
    );
}

// SE-AATF-04
#[test]
fn a_relative_aatf_rules_resolves_in_the_data_directory() {
    let sandbox = Sandbox::new("aatf_rules_relative");
    sandbox.fixture(PesVersion::Pes21, "EDIT");
    fs::write(sandbox.path("data/vgl27.rhai"), rules_file("vgl27")).unwrap();

    // The sandbox runs the command in-process, so its current directory is the test
    // process's, which holds no vgl27.rhai: the file is found only under the data
    // directory's resolution.
    let run = sandbox.run_with_settings(
        "[save-editor]\naatf_rules = 'vgl27.rhai'\n",
        &["check-aatf", &sandbox.arg("EDIT"), "--teams", "701"],
    );
    assert_eq!(run.exit_code(), 1, "{:?}", run.messages());
    assert!(
        run.messages().iter().any(|line| line == FIELDED_NOT_A),
        "{:?}",
        run.messages()
    );
}
