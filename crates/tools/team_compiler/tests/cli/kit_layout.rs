//! `compile` over kits carrying a layout marker (`team_compiler/pipeline.md` "4. Per-export
//! non-model steps", Kits, "Layout conversion"): a kit drawn for the pre-Fox layout compiled
//! for PES 21 has its sock islands re-laid out, checked against where the games' own uniform
//! models put each sock stripe (`tests/fixtures/kit_layout/`); a marker naming the target's
//! engine, or on a placeholder kit, changes nothing.

use std::fs;
use std::ops::Range;
use std::path::Path;

use dds_convert::{BlockCodec, SourceFormat, decode, encode_dds};

use crate::common::Sandbox;
use crate::compile::{cpk_entries, kit_texture, pes21_settings};
use crate::findings_of;

/// The bytes of `tests/fixtures/kit_layout/<name>`.
fn kit_layout_fixture(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/kit_layout")
            .join(name),
    )
    .unwrap()
}

/// `pre_fox_stripes.png` as a BC1 DDS with its full mip chain: lossless, since every color of
/// the fixture is exact in BC1 and every edge sits on a block edge.
fn stripes_dds() -> Vec<u8> {
    let png = decode(
        &kit_layout_fixture("pre_fox_stripes.png"),
        SourceFormat::Png,
    )
    .unwrap();
    encode_dds(&png, BlockCodec::Bc1).unwrap()
}

/// The top level of the kit texture `name` the sandbox's last `compile` wrote, decoded to
/// RGBA8, with its width.
fn compiled_top_level(sandbox: &Sandbox, name: &str) -> (Vec<u8>, u32) {
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let ftex = &entries[&kit_texture(name)];
    let mut decoded = decode(&ftex::ftex_to_dds(ftex).unwrap(), SourceFormat::Dds).unwrap();
    (decoded.mips.swap_remove(0), decoded.width)
}

/// The rows of both sock islands on the fixture's 1024-texel texture (632 to 1160 of 2048).
const ISLAND_ROWS: Range<u32> = 316..580;
/// Each sock's Fox island on the 1024-texel texture, its two bands together: the left
/// (8 to 376 of 2048) and the right (1672 to 2040).
const FOX_LEFT: Range<u32> = 4..188;
const FOX_RIGHT: Range<u32> = 836..1020;

/// How far, in the sum of the three channels' differences, a texel's color may be from a
/// stripe's to count as that stripe's: the stripes' colors differ from their neighbors' by
/// far more, so a texel blending two stripes is left out.
const STRIPE_COLOR_DISTANCE: u32 = 12;

/// One line of `sock_stripes.txt`: the island, the stripe's color and the Fox u the games'
/// models put its centre at, in 2048-px units.
struct Stripe {
    island: String,
    pre_fox: String,
    color: [u8; 3],
    fox_centre: f64,
}

fn stripes() -> Vec<Stripe> {
    let text = String::from_utf8(kit_layout_fixture("sock_stripes.txt")).unwrap();
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let [island, start, end, r, g, b, centre] = fields[..] else {
                panic!("not a stripe line: {line}");
            };
            Stripe {
                island: island.to_owned(),
                pre_fox: format!("{start}-{end}"),
                color: [r.parse().unwrap(), g.parse().unwrap(), b.parse().unwrap()],
                fox_centre: centre.parse().unwrap(),
            }
        })
        .collect()
}

/// The mean u, in 2048-px units, of the texels within `STRIPE_COLOR_DISTANCE` of `stripe`'s
/// color, over the island's rows and its sock's Fox columns of the top level `pixels`, 1024
/// texels wide; `None` when no texel is.
fn stripe_centre(pixels: &[u8], stripe: &Stripe) -> Option<f64> {
    let columns = match stripe.island.as_str() {
        "left" => FOX_LEFT,
        "right" => FOX_RIGHT,
        other => panic!("no island {other}"),
    };
    let (mut sum, mut count) = (0.0, 0u32);
    for y in ISLAND_ROWS {
        for x in columns.clone() {
            let at = ((y * 1024 + x) * 4) as usize;
            let distance: u32 = (0..3)
                .map(|c| u32::from(pixels[at + c].abs_diff(stripe.color[c])))
                .sum();
            if distance <= STRIPE_COLOR_DISTANCE {
                // A texel's centre, in units of a 2048-px texture.
                sum += (f64::from(x) + 0.5) * 2.0;
                count += 1;
            }
        }
    }
    (count > 0).then(|| sum / f64::from(count))
}

/// Writes `exports/co Midcup Layout` holding `p1/kit.dds`, the stripes, plus the empty file
/// `p1/<marker>` when `marker` names one.
fn write_stripes_export(sandbox: &Sandbox, marker: Option<&str>) {
    sandbox.write("exports/co Midcup Layout/Kits/p1/kit.dds", &stripes_dds());
    if let Some(marker) = marker {
        sandbox.write(&format!("exports/co Midcup Layout/Kits/p1/{marker}"), b"");
    }
}

const IDENTIFIED: &str = "Info export_identified [Keep] (team=/co/, id=714)";
const TEAM_COLORS_MISSING: &str = "Info team_colors_missing [Keep] ()";

/// What `compile` reports for `write_stripes_export`'s export when nothing is re-laid.
const STRIPES_FINDINGS: [&str; 4] = [
    IDENTIFIED,
    TEAM_COLORS_MISSING,
    "Info kit_config_generated [Keep] at Kits/p1 ()",
    "Info kit_colors_derived [Keep] at Kits/p1 ()",
];

// TC-KIT-18
#[test]
fn a_kit_marked_pre_fox_compiled_for_pes_21_has_its_socks_where_the_fox_models_read_them() {
    let marked = Sandbox::new("kit_layout_pre_fox");
    write_stripes_export(&marked, Some("pre-fox"));
    let unmarked = Sandbox::new("kit_layout_pre_fox_unmarked");
    write_stripes_export(&unmarked, None);

    let marked_run = marked.run(&pes21_settings(&marked), &["compile", "--no-deploy"]);
    let unmarked_run = unmarked.run(&pes21_settings(&unmarked), &["compile", "--no-deploy"]);

    let mut converted = STRIPES_FINDINGS.to_vec();
    converted.insert(
        3,
        "Info kit_layout_converted [Keep] at Kits/p1 (from=pre-fox, to=fox)",
    );
    assert_eq!(
        findings_of(&marked_run.messages(), "co Midcup Layout"),
        converted
    );
    assert_eq!(marked_run.exit_code(), 0);
    assert_eq!(
        findings_of(&unmarked_run.messages(), "co Midcup Layout"),
        STRIPES_FINDINGS
    );

    // The top levels; the lower ones have blocks straddling a rectangle's edge encoded
    // afresh, which may shift a texel beside the rectangle by a step.
    let (relaid, width) = compiled_top_level(&marked, "u0714p1");
    let (as_drawn, _) = compiled_top_level(&unmarked, "u0714p1");
    assert_eq!(width, 1024);
    // The four Fox rectangles' columns on the 1024-texel texture: left band 1 and 2, right
    // band 1 and 2.
    let fox_bands = [4..64, 64..188, 960..1020, 836..960];
    for y in 0..1024u32 {
        for x in 0..1024u32 {
            let in_band =
                ISLAND_ROWS.contains(&y) && fox_bands.iter().any(|band| band.contains(&x));
            let at = ((y * 1024 + x) * 4) as usize;
            if !in_band {
                assert_eq!(relaid[at..at + 4], as_drawn[at..at + 4], "({x}, {y})");
            }
        }
    }

    for stripe in stripes() {
        let Some(centre) = stripe_centre(&relaid, &stripe) else {
            panic!(
                "{} stripe {} {:?}: no texel of its color",
                stripe.island, stripe.pre_fox, stripe.color
            );
        };
        assert!(
            (centre - stripe.fox_centre).abs() <= 6.0,
            "{} stripe {}: centre {centre:.1}, the models' {}",
            stripe.island,
            stripe.pre_fox,
            stripe.fox_centre
        );
    }
}

/// Every entry of the CPK the sandbox's last `compile` wrote.
fn compiled_entries(sandbox: &Sandbox) -> std::collections::BTreeMap<String, Vec<u8>> {
    cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"))
}

// TC-KIT-20
#[test]
fn a_placeholder_kit_marked_pre_fox_compiles_as_without_the_marker() {
    let write = |sandbox: &Sandbox, marked: bool| {
        fs::create_dir_all(sandbox.root.join("exports/co Midcup Layout/Kits/p3")).unwrap();
        if marked {
            sandbox.write("exports/co Midcup Layout/Kits/p3/pre-fox", b"");
        }
        sandbox.write("exports/co Midcup Layout/Kits/all/fox", b"");
    };
    let marked = Sandbox::new("kit_layout_placeholder");
    write(&marked, true);
    let unmarked = Sandbox::new("kit_layout_placeholder_unmarked");
    write(&unmarked, false);

    let marked_run = marked.run(&pes21_settings(&marked), &["compile", "--no-deploy"]);
    let unmarked_run = unmarked.run(&pes21_settings(&unmarked), &["compile", "--no-deploy"]);

    // No kit_layout_converted: the placeholder is never re-laid.
    let findings = [
        "Warning kit_all_file_ignored [DropFile] at Kits/all/fox ()",
        IDENTIFIED,
        TEAM_COLORS_MISSING,
        "Info kit_config_generated [Keep] at Kits/p3 ()",
        "Info kit_placeholder [Keep] at Kits/p3 ()",
        "Warning kit_colors_missing [Keep] at Kits/p3 ()",
    ];
    assert_eq!(
        findings_of(&marked_run.messages(), "co Midcup Layout"),
        findings
    );
    assert_eq!(
        findings_of(&unmarked_run.messages(), "co Midcup Layout"),
        findings
    );
    let entries = compiled_entries(&marked);
    assert!(entries.contains_key(&kit_texture("u0714p3")));
    assert_eq!(entries, compiled_entries(&unmarked));
}

#[test]
fn a_kit_marked_fox_compiled_for_pes_21_is_as_without_the_marker() {
    let marked = Sandbox::new("kit_layout_fox");
    write_stripes_export(&marked, Some("fox"));
    let unmarked = Sandbox::new("kit_layout_fox_unmarked");
    write_stripes_export(&unmarked, None);

    let marked_run = marked.run(&pes21_settings(&marked), &["compile", "--no-deploy"]);
    let unmarked_run = unmarked.run(&pes21_settings(&unmarked), &["compile", "--no-deploy"]);

    for run in [&marked_run, &unmarked_run] {
        assert_eq!(
            findings_of(&run.messages(), "co Midcup Layout"),
            STRIPES_FINDINGS
        );
    }
    assert_eq!(compiled_entries(&marked), compiled_entries(&unmarked));
}
