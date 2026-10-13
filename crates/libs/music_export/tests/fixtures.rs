//! The committed exports (`tests/fixtures/README.md`): each parses with no
//! warning, reads back equal from its written form, and the hand-written one
//! carries every keyword the cup exports lack.

use std::path::Path;

use music_export::{
    Comparison, Condition, EndAction, EventKind, Instruction, MatchType, MusicExport, Parsed,
    PauseAction, Slot, SongEntry, StartOffset, parse, write,
};

const FIXTURES: [&str; 7] = [
    "98hu - Set A.4ccm",
    "o music summer.4ccm",
    "jp.4ccm",
    "vg.4ccm",
    "fgog music.4ccm",
    "v.4ccm",
    "every_keyword.4ccm",
];

fn fixture(name: &str) -> Parsed {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let bytes = std::fs::read(&path).expect("fixture read");
    let stem = path
        .file_stem()
        .expect("stem")
        .to_str()
        .expect("utf-8 stem");
    parse(&bytes, stem).expect("fixture parses")
}

fn entries<'a>(export: &'a MusicExport, slot: &Slot) -> &'a [SongEntry] {
    export.slots.get(slot).map(Vec::as_slice).unwrap_or(&[])
}

fn player(name: &str) -> Slot {
    Slot::Player(name.to_string())
}

#[test]
fn every_fixture_parses_without_warnings_and_reads_back_equal() {
    for name in FIXTURES {
        let parsed = fixture(name);
        assert!(parsed.warnings.is_empty(), "{name}: {:?}", parsed.warnings);
        let written = write(&parsed.export);
        let again = parse(written.as_bytes(), "other stem").expect("the written form parses");
        assert!(again.warnings.is_empty(), "{name}: {:?}", again.warnings);
        assert_eq!(again.export, parsed.export, "{name}");
        assert_eq!(
            write(&again.export),
            written,
            "{name}: the written form is stable"
        );
    }
}

#[test]
fn the_cup_exports_read_as_their_authors_meant() {
    let hu = fixture("98hu - Set A.4ccm").export;
    assert_eq!(hu.team, "98hu");
    assert!(hu.sync && hu.normalize);
    let anthems = entries(&hu, &Slot::Anthem);
    assert_eq!(anthems.len(), 3);
    assert_eq!(
        anthems[0].conditions,
        [Condition::Match(vec![
            MatchType::Semifinal,
            MatchType::ThirdPlace,
            MatchType::Final,
            MatchType::Boss
        ])]
    );
    assert_eq!(anthems[0].instructions, [Instruction::End(EndAction::Stop)]);
    let victories = entries(&hu, &Slot::Victory);
    assert_eq!(victories[1].conditions, [Condition::Special(None)]);
    let mima = entries(&hu, &player("[CF] I can feel it in my bones"));
    assert_eq!(mima.len(), 2, "the position tag is part of the name");
    assert_eq!(mima[0].conditions, [Condition::Goals(Comparison::Ge, 3)]);
    let mai = entries(&hu, &player("[RB] You're not bad"));
    assert_eq!(
        mai[0].instructions,
        [
            Instruction::End(EndAction::Stop),
            Instruction::Pause {
                action: PauseAction::Restart,
                every: 1
            }
        ]
    );
    assert!(
        !hu.slots.contains_key(&Slot::Chant),
        "a comment-only section"
    );

    let o = fixture("o music summer.4ccm").export;
    let takumi = entries(&o, &player("Takumi"));
    assert_eq!(takumi[0].conditions, [Condition::Goals(Comparison::Eq, 1)]);
    assert_eq!(
        takumi[0].instructions,
        [Instruction::Start(StartOffset { millis: 22_617 })]
    );
    assert_eq!(entries(&o, &player("[COOL VIBRATIONS]")).len(), 1);

    let jp = fixture("jp.4ccm").export;
    assert_eq!(
        entries(&jp, &Slot::Anthem)[0].conditions,
        [Condition::Opponent(vec!["lit".into()])]
    );
    assert_eq!(entries(&jp, &player("Billy")).len(), 17);
    assert_eq!(
        entries(
            &jp,
            &player("Other 2hu reps:,Whale/Suffering/Rapist,Yukari/Tenshi")
        )
        .len(),
        1
    );

    let vg = fixture("vg.4ccm").export;
    assert!(vg.sync);
    let helldiver = entries(&vg, &player("John Helldiver"));
    assert_eq!(
        helldiver[0].conditions,
        [Condition::Comeback, Condition::First]
    );
    assert_eq!(helldiver[1].instructions, [Instruction::Advance]);
    let cunny = entries(&vg, &player("Mandate of Cunny"));
    assert_eq!(
        cunny[2].conditions,
        [
            Condition::Match(vec![
                MatchType::Ro16,
                MatchType::ThirdPlace,
                MatchType::Semifinal,
                MatchType::Quarterfinal
            ]),
            Condition::Not(Box::new(Condition::First))
        ]
    );
    let hagcat = entries(&vg, &player("Sexy Hagcat"));
    assert_eq!(
        hagcat[2].instructions,
        [
            Instruction::Start(StartOffset { millis: 105_000 }),
            Instruction::End(EndAction::Loop),
            Instruction::Randomise
        ]
    );
    assert_eq!(entries(&vg, &Slot::Chant).len(), 41);

    let fgog = fixture("fgog music.4ccm").export;
    assert_eq!(
        entries(&fgog, &Slot::Anthem)[4].instructions,
        [Instruction::Start(StartOffset { millis: 39_600 })]
    );
    let redman = entries(&fgog, &player("Smug Redman"));
    assert_eq!(redman[0].conditions, [Condition::Comeback, Condition::Once]);
    assert_eq!(redman[1].conditions, [Condition::Every(3)]);

    let v = fixture("v.4ccm").export;
    assert_eq!(
        entries(&v, &Slot::Goal)[0].conditions,
        [Condition::Lead(Comparison::Gt, 3)]
    );
    assert_eq!(
        entries(&v, &Slot::Anthem)[1].conditions,
        [Condition::Match(vec![
            MatchType::ThirdPlace,
            MatchType::Consolation
        ])]
    );
}

#[test]
fn the_hand_written_fixture_covers_the_rest_of_the_grammar() {
    let export = fixture("every_keyword.4ccm").export;
    assert_eq!(export.team, "nijiura jp");
    assert!(!export.sync && !export.normalize);
    let anthems = entries(&export, &Slot::Anthem);
    assert_eq!(
        anthems[0].conditions,
        [Condition::Match(MatchType::KNOCKOUTS.to_vec())]
    );
    assert_eq!(
        anthems[0].instructions,
        [Instruction::Start(StartOffset { millis: 3_723_500 })]
    );
    assert_eq!(anthems[1].file, "nijiura jp - Anthem.mp3");
    let victories = entries(&export, &Slot::Victory);
    assert_eq!(
        victories[0].conditions,
        [Condition::Special(Some("Promotion Anthem".into()))]
    );
    assert_eq!(
        victories[1].conditions,
        [Condition::Home, Condition::TeamGoals(Comparison::Ge, 3)]
    );
    let goal = &entries(&export, &Slot::Goal)[0];
    assert_eq!(
        goal.conditions,
        [Condition::Opponent(vec![
            "jp".into(),
            "v".into(),
            "o".into()
        ])]
    );
    assert_eq!(
        goal.instructions,
        [
            Instruction::Pause {
                action: PauseAction::Restart,
                every: 2
            },
            Instruction::Speed(1.25)
        ]
    );
    assert_eq!(
        entries(&export, &Slot::Chant)[0].instructions,
        [Instruction::Unrandom, Instruction::Louder]
    );
    let janny = entries(&export, &player("Cat Look Like Janny"));
    assert_eq!(janny[0].instructions, [Instruction::Warcry]);
    assert_eq!(
        janny[1].conditions,
        [
            Condition::Lead(Comparison::Gt, 1),
            Condition::MostGoals(None),
            Condition::Time(Comparison::Le, 45)
        ]
    );
    assert_eq!(
        janny[2].conditions,
        [
            Condition::Not(Box::new(Condition::MostGoals(Some("Get Out".into())))),
            Condition::Every(2)
        ]
    );
    let get_out = entries(&export, &player("Get Out"));
    assert_eq!(get_out.len(), 1, "the event lines are clips, not entries");
    let kinds: Vec<(&str, EventKind)> = export
        .events
        .iter()
        .map(|clip| (clip.player.as_str(), clip.kind))
        .collect();
    assert_eq!(
        kinds,
        [("Get Out", EventKind::Red), ("Get Out", EventKind::Sub)]
    );
    assert_eq!(
        export.events[1].entry.instructions,
        [Instruction::Start(StartOffset { millis: 2_000 })]
    );
    let escaped = &entries(&export, &player("Other 2hu reps:,Whale/Suffering"))[0];
    assert_eq!(
        escaped.file, "\\[escaped].mp3",
        "the file field is not tokenized"
    );
    assert_eq!(
        escaped.conditions,
        [Condition::Opponent(vec!["[v]".into()])]
    );
}
