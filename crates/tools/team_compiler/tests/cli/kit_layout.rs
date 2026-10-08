//! `compile` over kits carrying a layout marker (`team_compiler/pipeline.md` "4. Per-export
//! non-model steps", Kits, "Layout conversion"): a kit drawn for the pre-Fox layout compiled
//! for PES 21 has its sock islands re-laid out, checked against where the games' own uniform
//! models put each sock stripe (`tests/fixtures/kit_layout/`), and its srm with it; a marker
//! naming the target's engine, or on a placeholder kit, changes nothing. A number atlas in the
//! other engine's arrangement is re-arranged by its shape alone, for PES 21 and for PES 17 (the
//! glyph atlas sentences of the same section).

use std::fs;
use std::ops::Range;
use std::path::Path;

use dds_convert::{BlockCodec, SourceFormat, decode, encode_dds};

use crate::common::Sandbox;
use crate::compile::{cpk_entries, kit_texture, pes_settings, pes21_settings, tracer_kit};
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
pub(crate) fn stripes_dds() -> Vec<u8> {
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

/// The four Fox rectangles' columns on the 1024-texel texture: left band 1 and 2, right band
/// 1 and 2.
const FOX_BANDS: [Range<u32>; 4] = [4..64, 64..188, 960..1020, 836..960];

/// The four pre-Fox rectangles' columns on the 1024-texel texture (left 8 to 168 and 168 to
/// 448 of 2048, the right their mirror image): left band 1 and 2, right band 1 and 2.
pub(crate) const PRE_FOX_BANDS: [Range<u32>; 4] = [4..84, 84..224, 940..1020, 800..940];

/// Asserts that the top levels `relaid` and `as_drawn` of a 1024-texel texture hold the same
/// texels outside the rectangles `bands` span over `ISLAND_ROWS`, and differ somewhere inside
/// each. The top levels only: the lower ones have blocks straddling a rectangle's edge
/// encoded afresh, which may shift a texel beside the rectangle by a step.
pub(crate) fn assert_moved_inside_bands_only(relaid: &[u8], as_drawn: &[u8], bands: &[Range<u32>]) {
    assert_eq!(relaid.len(), 1024 * 1024 * 4);
    assert_eq!(as_drawn.len(), relaid.len());
    let texel = |pixels: &[u8], x: u32, y: u32| {
        let at = ((y * 1024 + x) * 4) as usize;
        pixels[at..at + 4].to_vec()
    };
    for y in 0..1024u32 {
        for x in 0..1024u32 {
            let in_band = ISLAND_ROWS.contains(&y) && bands.iter().any(|band| band.contains(&x));
            if !in_band {
                assert_eq!(texel(relaid, x, y), texel(as_drawn, x, y), "({x}, {y})");
            }
        }
    }
    for band in bands {
        let moved = ISLAND_ROWS
            .flat_map(|y| band.clone().map(move |x| (x, y)))
            .any(|(x, y)| texel(relaid, x, y) != texel(as_drawn, x, y));
        assert!(moved, "the band over columns {band:?} is as drawn");
    }
}

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
pub(crate) fn write_stripes_export(sandbox: &Sandbox, marker: Option<&str>) {
    sandbox.write("exports/co Midcup Layout/Kits/p1/kit.dds", &stripes_dds());
    if let Some(marker) = marker {
        sandbox.write(&format!("exports/co Midcup Layout/Kits/p1/{marker}"), b"");
    }
}

const IDENTIFIED: &str = "Info export_identified [Keep] (team=/co/, id=714)";
const TEAM_COLORS_MISSING: &str = "Info team_colors_missing [Keep] ()";

/// What `compile` reports for `write_stripes_export`'s export when nothing is re-laid.
pub(crate) const STRIPES_FINDINGS: [&str; 4] = [
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

    let (relaid, width) = compiled_top_level(&marked, "u0714p1");
    let (as_drawn, _) = compiled_top_level(&unmarked, "u0714p1");
    assert_eq!(width, 1024);
    assert_moved_inside_bands_only(&relaid, &as_drawn, &FOX_BANDS);

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
pub(crate) fn compiled_entries(sandbox: &Sandbox) -> std::collections::BTreeMap<String, Vec<u8>> {
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

// TC-KIT-28
#[test]
fn a_kit_srm_marked_pre_fox_compiled_for_pes_21_is_re_laid_with_its_kit() {
    let write = |sandbox: &Sandbox, marker: Option<&str>| {
        write_stripes_export(sandbox, marker);
        sandbox.write(
            "exports/co Midcup Layout/Kits/p1/kit_srm.dds",
            &stripes_dds(),
        );
    };
    let marked = Sandbox::new("kit_layout_srm_pre_fox");
    write(&marked, Some("pre-fox"));
    let unmarked = Sandbox::new("kit_layout_srm_unmarked");
    write(&unmarked, None);

    let marked_run = marked.run(&pes21_settings(&marked), &["compile", "--no-deploy"]);
    let unmarked_run = unmarked.run(&pes21_settings(&unmarked), &["compile", "--no-deploy"]);

    // Once for the kit, its srm re-laid with its main texture.
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
    let (relaid, width) = compiled_top_level(&marked, "u0714p1_srm");
    let (as_drawn, _) = compiled_top_level(&unmarked, "u0714p1_srm");
    assert_eq!(width, 1024);
    assert_moved_inside_bands_only(&relaid, &as_drawn, &FOX_BANDS);
    // Unmarked, the srm is emitted as given.
    let given = decode(&stripes_dds(), SourceFormat::Dds).unwrap();
    assert!(as_drawn == given.mips[0], "the unmarked srm as given");
}

/// One opaque color per digit, every channel 0, 128 or 255, which the block codecs keep
/// within `CODEC_TOLERANCE`.
const DIGIT_COLORS: [[u8; 4]; 10] = [
    [255, 0, 0, 255],
    [0, 255, 0, 255],
    [0, 0, 255, 255],
    [255, 255, 0, 255],
    [255, 0, 255, 255],
    [0, 255, 255, 255],
    [255, 255, 255, 255],
    [128, 0, 0, 255],
    [0, 128, 0, 255],
    [0, 0, 128, 255],
];

/// How far a channel of a compiled texel may be from the color drawn: a block codec's 5:6:5
/// endpoints move 128 by up to 4.
const CODEC_TOLERANCE: u8 = 8;

/// The middle of each digit's slot along a 2048-long PES 18-21 row: where the stock rows keep
/// their digits.
const SLOT_MIDDLES_2048: [u32; 10] = [80, 265, 420, 620, 820, 1020, 1220, 1420, 1620, 1820];

/// A `width` x `height` BC1 DDS with its generated mip chain, its texel at (x, y)
/// `color(x, y)`.
pub(crate) fn bc1_dds(width: u32, height: u32, color: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            pixels.extend_from_slice(&color(x, y));
        }
    }
    let decoded = dds_convert::Decoded {
        width,
        height,
        mips: vec![pixels],
        blocks: None,
        authored_mips: false,
    };
    encode_dds(&decoded, BlockCodec::Bc1).unwrap()
}

// TC-KIT-26
#[test]
fn a_column_number_atlas_compiled_for_pes_21_becomes_a_row_and_a_row_stays_as_given() {
    let column = bc1_dds(128, 2048, |_, y| DIGIT_COLORS[(y * 10 / 2048) as usize]);
    let row = bc1_dds(2048, 256, |x, _| DIGIT_COLORS[(x * 10 / 2048) as usize]);
    let sandbox = Sandbox::new("number_atlas_pes21");
    for (kit, back) in [("p1", &column), ("p2", &row)] {
        let folder = format!("exports/co Midcup Numbers/Kits/{kit}");
        sandbox.write(&format!("{folder}/kit.dds"), &tracer_kit());
        sandbox.write(&format!("{folder}/kit_back.dds"), back);
    }

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Numbers"),
        [
            IDENTIFIED,
            TEAM_COLORS_MISSING,
            "Info kit_config_generated [Keep] at Kits/p1 ()",
            "Info kit_config_generated [Keep] at Kits/p2 ()",
            "Info kit_colors_derived [Keep] at Kits/p1 ()",
            "Info kit_colors_derived [Keep] at Kits/p2 ()",
        ]
    );
    assert_eq!(run.exit_code(), 0);

    let (p1_back, width) = compiled_top_level(&sandbox, "u0714p1_back");
    assert_eq!((width, p1_back.len()), (2048, 2048 * 256 * 4));
    for (digit, x) in SLOT_MIDDLES_2048.into_iter().enumerate() {
        let at = ((128 * 2048 + x) * 4) as usize;
        let texel = &p1_back[at..at + 4];
        let near = (0..4).all(|c| texel[c].abs_diff(DIGIT_COLORS[digit][c]) <= CODEC_TOLERANCE);
        assert!(near, "digit {digit} at x {x}: {texel:?}");
    }
    let (p2_back, width) = compiled_top_level(&sandbox, "u0714p2_back");
    assert_eq!(width, 2048);
    assert_eq!(p2_back, decode(&row, SourceFormat::Dds).unwrap().mips[0]);
}

/// Where each digit's slot starts along a 2048-long PES 18-21 row (`pipeline.md` "4.
/// Per-export non-model steps", Kits, the glyph atlases).
const SLOT_STARTS_2048: [u32; 10] = [0, 190, 340, 540, 740, 940, 1140, 1340, 1540, 1740];

/// The top level of the pre-Fox kit texture `name` the sandbox's last `compile` wrote, a DDS
/// decoded to RGBA8, with its width and height.
pub(crate) fn compiled_pre_fox_top_level(sandbox: &Sandbox, name: &str) -> (Vec<u8>, u32, u32) {
    let entries = compiled_entries(sandbox);
    let path = format!("common/character0/model/character/uniform/texture/{name}.dds");
    let mut decoded = decode(&entries[&path], SourceFormat::Dds).unwrap();
    (decoded.mips.swap_remove(0), decoded.width, decoded.height)
}

// TC-KIT-26
#[test]
fn a_row_number_atlas_compiled_for_pes_17_becomes_a_column_and_a_column_stays_as_given() {
    let column = bc1_dds(128, 2048, |_, y| DIGIT_COLORS[(y * 10 / 2048) as usize]);
    // Each digit's color from its slot's start to the next slot's: the 0's slot ends at 160,
    // and the 160 to 190 gap, which no move reads, keeps the 0's color.
    let row = bc1_dds(2048, 256, |x, _| {
        let digit = SLOT_STARTS_2048.iter().rposition(|start| *start <= x);
        DIGIT_COLORS[digit.unwrap()]
    });
    let sandbox = Sandbox::new("number_atlas_pes17");
    for (kit, back) in [("p1", &column), ("p2", &row)] {
        let folder = format!("exports/co Midcup Numbers/Kits/{kit}");
        sandbox.write(&format!("{folder}/kit.dds"), &tracer_kit());
        sandbox.write(&format!("{folder}/kit_back.dds"), back);
    }

    let run = sandbox.run(&pes_settings(&sandbox, 17), &["compile", "--no-deploy"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Numbers"),
        [
            IDENTIFIED,
            TEAM_COLORS_MISSING,
            "Info kit_config_generated [Keep] at Kits/p1 ()",
            "Info kit_config_generated [Keep] at Kits/p2 ()",
            "Info kit_colors_derived [Keep] at Kits/p1 ()",
            "Info kit_colors_derived [Keep] at Kits/p2 ()",
        ]
    );
    assert_eq!(run.exit_code(), 0);

    let (p1_back, width, height) = compiled_pre_fox_top_level(&sandbox, "u0714p1_back");
    assert_eq!((width, height), (128, 2048));
    assert_eq!(p1_back, decode(&column, SourceFormat::Dds).unwrap().mips[0]);
    let (p2_back, width, height) = compiled_pre_fox_top_level(&sandbox, "u0714p2_back");
    assert_eq!((width, height), (128, 2048));
    // The middle of each digit's tenth of the column.
    for (digit, color) in DIGIT_COLORS.iter().enumerate() {
        let y = (digit as u32 * 2 + 1) * 2048 / 20;
        let at = ((y * 128 + 64) * 4) as usize;
        let texel = &p2_back[at..at + 4];
        let near = (0..4).all(|c| texel[c].abs_diff(color[c]) <= CODEC_TOLERANCE);
        assert!(near, "digit {digit} at y {y}: {texel:?}");
    }
}
