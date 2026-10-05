//! `compile` over textures in every accepted image format (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", step 5): a player folder's, a shared folder's, Common's
//! and a kit's, each converted for the target version's codec with its mip chain, observed
//! through the emitted FTEX's header and blocks; and the texture findings the deep pass
//! reports from their headers, at `check` and at `compile` (`team_compiler/messages.md`
//! "Textures").

use std::fs;
use std::path::Path;

use dds_convert::{
    BlockCodec, Blocks, Decoded, SourceFormat, Target, TextureRole, decode, encode_dds, encode_png,
};
use fmdl::ops::paths::texture_paths;
use fmdl::{FmdlFile, Model};
use ftex::PixelFormat;
use pes_version::PesVersion;

use crate::common::Sandbox;
use crate::compile::{
    compiled_kits, compiled_players, cpk_entries, kit_texture, pass_through_settings, pes_settings,
    pes21_settings, tracer_kit, tracer_player_file,
};
use crate::findings_of;
use crate::models::face_package;

/// The player's texture home, the per-player common subfolder.
const PLAYER_TEXTURES: &str = "Asset/model/character/common/714/05 - A/sourceimages/#windx11";

/// The bytes of `tests/fixtures/textures/<name>` (that folder's `README.md`).
pub(crate) fn texture_fixture(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/textures")
            .join(name),
    )
    .unwrap()
}

/// The tracer's hair model naming the texture `file_name` (`skin.dds`) instead of `shirt.dds`,
/// through `fmdl`'s model API.
fn hair_model_naming(file_name: &str) -> Vec<u8> {
    let file = FmdlFile::read(&tracer_player_file("fcl_hair.fmdl")).unwrap();
    let mut model = Model::from_file(&file).unwrap();
    for material in &mut model.materials {
        for (_, texture) in &mut material.textures {
            if texture.file_name == "shirt.dds" {
                texture.file_name = file_name.to_owned();
            }
        }
    }
    model.to_file().unwrap().write()
}

/// Writes slot `slot` of `exports/<export>`, the folder `<slot> - A`, as the hair model naming
/// `skin` beside the texture file `texture_name` holding `bytes`.
fn write_skin_player(
    sandbox: &Sandbox,
    export: &str,
    slot: &str,
    texture_name: &str,
    bytes: &[u8],
) {
    let player = format!("exports/{export}/Players/{slot} - A");
    sandbox.write(
        &format!("{player}/fcl_hair.fmdl"),
        &hair_model_naming("skin.dds"),
    );
    sandbox.write(&format!("{player}/{texture_name}"), bytes);
}

/// Compiles the sandbox for PES `version`, asserting the export `name`, slot 05 written by
/// `write_skin_player`, reports only the hair model's unnormalized weights and its identity,
/// and returns the entry at `path`.
fn compile_entry(sandbox: &Sandbox, name: &str, version: u8, path: &str) -> Vec<u8> {
    let run = sandbox.run(&pes_settings(sandbox, version), &["compile"]);
    assert_eq!(
        findings_of(&run.messages(), name),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ],
        "PES {version}"
    );
    assert_eq!(run.exit_code(), 0, "PES {version}");
    let mut entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    entries
        .remove(path)
        .unwrap_or_else(|| panic!("PES {version}: no {path} among {:?}", entries.keys()))
}

/// The emitted FTEX `bytes` converted back to a DDS and decoded: asserts the codec, the
/// 1024x1024 size and the full 11-level chain on the *payload*, which `ftex::info`'s
/// header-only read cannot see (TC-TEX-01, TC-TEX-10).
fn assert_decodes_as(bytes: &[u8], codec: BlockCodec, label: &str) {
    let decoded = decode(&ftex::ftex_to_dds(bytes).unwrap(), SourceFormat::Dds).unwrap();
    let blocks = decoded.blocks.as_ref().unwrap();
    assert_eq!(blocks.codec, codec, "{label}");
    assert_eq!((decoded.width, decoded.height), (1024, 1024), "{label}");
    assert_eq!(decoded.mips.len(), 11, "{label}: 1024x1024 down to 1x1");
    assert_eq!(blocks.mips.len(), 11, "{label}: a block buffer per level");
}

// TC-TEX-01
#[test]
fn a_png_skin_is_bc7_with_a_full_mip_chain_on_pes_21_and_bc3_on_pes_18() {
    let sandbox = Sandbox::new("tex_png_skin");
    write_skin_player(
        &sandbox,
        "co - Skin",
        "05",
        "skin.png",
        &texture_fixture("skin.png"),
    );
    let skin = format!("{PLAYER_TEXTURES}/skin.ftex");

    for (version, format, codec) in [
        (21, PixelFormat::Bc7, BlockCodec::Bc7),
        (18, PixelFormat::Bc3, BlockCodec::Bc3),
    ] {
        let emitted = compile_entry(&sandbox, "co - Skin", version, &skin);
        let info = ftex::info(&emitted).unwrap();
        assert_eq!(info.format, format, "PES {version}");
        assert_eq!(info.mipmaps, 11, "PES {version}: 1024x1024 down to 1x1");
        assert_eq!((info.width, info.height), (1024, 1024), "PES {version}");
        assert_decodes_as(&emitted, codec, &format!("PES {version}"));
    }
}

// TC-TEX-02
#[test]
fn a_bc7_dds_skin_keeps_its_blocks_on_pes_21_and_is_bc3_on_pes_18() {
    let sandbox = Sandbox::new("tex_bc7_skin");
    let source = texture_fixture("bc7.dds");
    write_skin_player(&sandbox, "co - Skin", "05", "skin.dds", &source);
    let skin = format!("{PLAYER_TEXTURES}/skin.ftex");

    let emitted = compile_entry(&sandbox, "co - Skin", 21, &skin);
    let round_trip = ftex::ftex_to_dds(&emitted).unwrap();
    let source_blocks = decode(&source, SourceFormat::Dds).unwrap().blocks.unwrap();
    let emitted_blocks = decode(&round_trip, SourceFormat::Dds)
        .unwrap()
        .blocks
        .unwrap();
    assert_eq!(source_blocks.mips.len(), 6, "the fixture's full chain");
    assert_eq!(
        emitted_blocks, source_blocks,
        "the same BC7 blocks, mip by mip"
    );

    let info = ftex::info(&compile_entry(&sandbox, "co - Skin", 18, &skin)).unwrap();
    assert_eq!(info.format, PixelFormat::Bc3);
    assert_eq!(info.mipmaps, 6);
}

#[test]
fn a_raster_nrm_texture_is_a_bc3_normal_map_on_pes_21() {
    let sandbox = Sandbox::new("tex_png_nrm");
    // A color raster of the same size would be BC7 on PES 21; the `_nrm` stem makes it a
    // normal map, which is BC3 (DXT5nm) on every version.
    write_skin_player(
        &sandbox,
        "co - Nrm",
        "05",
        "skin_nrm.png",
        &texture_fixture("kit.png"),
    );

    let emitted = compile_entry(
        &sandbox,
        "co - Nrm",
        21,
        &format!("{PLAYER_TEXTURES}/skin_nrm.ftex"),
    );

    let info = ftex::info(&emitted).unwrap();
    assert_eq!(info.format, PixelFormat::Bc3);
    assert_eq!((info.width, info.height, info.mipmaps), (256, 128, 9));
}

/// The tracer's hair model naming `skin.dds` and `skin_nrm.dds`: its `shirt.dds` and
/// `dummy_nrm.dds` renamed.
fn hair_model_naming_skin_and_nrm() -> Vec<u8> {
    let file = FmdlFile::read(&tracer_player_file("fcl_hair.fmdl")).unwrap();
    let mut model = Model::from_file(&file).unwrap();
    for material in &mut model.materials {
        for (_, texture) in &mut material.textures {
            if texture.file_name == "shirt.dds" {
                texture.file_name = "skin.dds".to_owned();
            } else if texture.file_name == "dummy_nrm.dds" {
                texture.file_name = "skin_nrm.dds".to_owned();
            }
        }
    }
    model.to_file().unwrap().write()
}

// TC-TEX-10
#[test]
fn an_opaque_skin_is_bc1_and_its_normal_map_bc3_with_x_in_alpha_y_in_green_on_pes_18() {
    let sandbox = Sandbox::new("tex_pes18_codecs");
    let export = "exports/co - Codecs";
    let player = format!("{export}/Players/05 - A");
    sandbox.write(
        &format!("{player}/fcl_hair.fmdl"),
        &hair_model_naming_skin_and_nrm(),
    );
    // 1024x1024, fully opaque: a color texture PES 18 encodes as BC1.
    let skin_png = encode_png(&[40, 100, 160, 255].repeat(1024 * 1024), 1024, 1024).unwrap();
    // A flat normal map, every source pixel (X, Y) = (10, 200).
    let nrm_png = encode_png(&[10, 200, 90, 255].repeat(16 * 16), 16, 16).unwrap();
    sandbox.write(&format!("{player}/skin.png"), &skin_png);
    sandbox.write(&format!("{player}/skin_nrm.png"), &nrm_png);

    let run = sandbox.run(&pes_settings(&sandbox, 18), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Codecs"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));

    let skin = &entries[&format!("{PLAYER_TEXTURES}/skin.ftex")];
    let info = ftex::info(skin).unwrap();
    assert_eq!(info.format, PixelFormat::Bc1);
    assert_eq!((info.width, info.height, info.mipmaps), (1024, 1024, 11));
    assert_decodes_as(skin, BlockCodec::Bc1, "skin.ftex");

    let nrm = &entries[&format!("{PLAYER_TEXTURES}/skin_nrm.ftex")];
    assert_eq!(ftex::info(nrm).unwrap().format, PixelFormat::Bc3);
    // The bytes the normal-map conversion of the source gives.
    let decoded_source = decode(&nrm_png, SourceFormat::Png).unwrap();
    let expected = dds_convert::convert(
        &decoded_source,
        Target {
            version: PesVersion::Pes18,
            role: TextureRole::Normal,
        },
    )
    .unwrap();
    assert_eq!(*nrm, expected);
    // The Fox DXT5nm layout: X (the source's red) in alpha, Y (its green) in green, with
    // red 255 and blue 0; the flat block decodes each channel as written.
    let decoded = decode(&ftex::ftex_to_dds(nrm).unwrap(), SourceFormat::Dds).unwrap();
    assert_eq!((decoded.width, decoded.height), (16, 16));
    for pixel in decoded.mips[0].as_chunks::<4>().0 {
        assert_eq!((pixel[0], pixel[2], pixel[3]), (255, 0, 10));
        assert!(pixel[1].abs_diff(200) <= 2, "green: {}", pixel[1]);
    }
}

// TC-TEX-06
#[test]
fn png_and_tga_kit_textures_are_ftex_under_the_kit_s_names_and_a_webp_portrait_is_a_dds() {
    let sandbox = Sandbox::new("tex_kit_formats");
    sandbox.write(
        "exports/co - Kit/Kits/p1/kit.png",
        &texture_fixture("kit.png"),
    );
    sandbox.write(
        "exports/co - Kit/Kits/p1/kit_back.tga",
        &texture_fixture("kit_back.tga"),
    );
    sandbox.write(
        "exports/co - Kit/Portraits/player_05.webp",
        &texture_fixture("portrait.webp"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Kit"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/p1 ()",
            "Info kit_colors_derived [Keep] at Kits/p1 ()",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    for (name, width, height, mipmaps) in [("u0714p1", 256, 128, 9), ("u0714p1_back", 128, 64, 8)] {
        let info = ftex::info(&entries[&kit_texture(name)]).unwrap();
        assert_eq!(info.format, PixelFormat::Bc7, "{name}");
        assert_eq!(
            (info.width, info.height, info.mipmaps),
            (width, height, mipmaps),
            "{name}"
        );
    }
    let portrait = &entries["common/render/symbol/player/71405.dds"];
    assert!(portrait.starts_with(b"DDS "));
    let layout = ftex::dds::read_layout(portrait).unwrap();
    assert_eq!(layout.pixel, ftex::dds::DdsPixel::Format(PixelFormat::Bc3));
    assert_eq!((layout.width, layout.height, layout.mipmaps), (128, 128, 8));
}

/// The paths of the CPK the sandbox's last `compile` wrote.
fn compiled_paths(sandbox: &Sandbox) -> Vec<String> {
    cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"))
        .into_keys()
        .collect()
}

/// The CPK path of the face package of `/co/`'s slot `slot`.
fn co_face(slot: u32) -> String {
    format!("Asset/model/character/face/real/714{slot:02}/#Win/face.fpk")
}

/// A single-level BC1 DDS of `width`x`height`, its blocks zero: built from the blocks, so
/// nothing is encoded.
pub(crate) fn bc1_dds(width: u32, height: u32) -> Vec<u8> {
    let blocks = width.div_ceil(4) * height.div_ceil(4) * 8;
    let decoded = Decoded {
        width,
        height,
        mips: vec![vec![0; (width * height * 4) as usize]],
        blocks: Some(Blocks {
            codec: BlockCodec::Bc1,
            mips: vec![vec![0; blocks as usize]],
        }),
        authored_mips: true,
    };
    encode_dds(&decoded, BlockCodec::Bc1).unwrap()
}

/// `dds_convert`'s fixture `rgba8.dds`: 32x16 uncompressed RGBA8 under a DX10 header, six
/// levels.
fn uncompressed_dds() -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../libs/dds_convert/tests/fixtures/rgba8.dds"),
    )
    .unwrap()
}

/// Writes `exports/co - Sizes`: slot 03 a 3x3 skin, slot 04 a 300x300 one, slot 05 PNG bytes
/// under `skin.dds` (renamed, not resaved), each beside the hair model naming it, and a kit,
/// so the CPK is written whatever the folders become.
fn write_texture_findings_export(sandbox: &Sandbox) {
    let export = "co - Sizes";
    write_skin_player(
        sandbox,
        export,
        "03",
        "skin.png",
        &texture_fixture("tiny.png"),
    );
    write_skin_player(
        sandbox,
        export,
        "04",
        "skin.png",
        &texture_fixture("odd.png"),
    );
    write_skin_player(
        sandbox,
        export,
        "05",
        "skin.dds",
        &texture_fixture("portrait.png"),
    );
    sandbox.write(&format!("exports/{export}/Kits/p1/kit.dds"), &tracer_kit());
}

/// The hair model's line on slot `slot`'s folder, which both commands report.
fn hair_weights(slot: &str) -> String {
    format!(
        "Info fmdl_weights_not_normalized [Keep] at Players/{slot} - A (file=fcl_hair.fmdl, count=1662)"
    )
}

// TC-TEX-03
#[test]
fn texture_findings_are_checked_and_each_drops_its_folder() {
    let sandbox = Sandbox::new("tex_checked");
    write_texture_findings_export(&sandbox);
    let findings = [
        hair_weights("03"),
        "Error texture_too_small [DropFolder] at Players/03 - A (file=skin.png)".to_owned(),
        hair_weights("04"),
        "Error texture_not_pow2 [DropFolder] at Players/04 - A (file=skin.png)".to_owned(),
        hair_weights("05"),
        "Error texture_type_mismatch [DropFolder] at Players/05 - A (file=skin.dds)".to_owned(),
        "Info export_identified [Keep] (team=/co/, id=714)".to_owned(),
    ];

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);
    assert_eq!(findings_of(&check.messages(), "co - Sizes"), findings);
    assert_eq!(check.exit_code(), 1);

    let compile = sandbox.run(&pes21_settings(&sandbox), &["compile"]);
    let mut compiled = findings.to_vec();
    compiled.push("Info team_colors_missing [Keep] ()".to_owned());
    compiled.push("Info kit_config_generated [Keep] at Kits/p1 ()".to_owned());
    compiled.push("Info kit_colors_derived [Keep] at Kits/p1 ()".to_owned());
    assert_eq!(findings_of(&compile.messages(), "co - Sizes"), compiled);
    assert_eq!(compile.exit_code(), 1);
    // The kit is in the CPK; nothing of the three folders is, neither a face nor a texture.
    let paths = compiled_paths(&sandbox);
    assert!(paths.contains(&kit_texture("u0714p1")), "{paths:?}");
    assert!(
        paths
            .iter()
            .all(|path| !path.starts_with("Asset/model/character/face/")
                && !path.starts_with("Asset/model/character/common/714/")),
        "{paths:?}"
    );
}

#[test]
fn pass_through_keeps_an_odd_sized_texture_s_folder_but_not_a_renamed_one() {
    let sandbox = Sandbox::new("tex_pass_through");
    write_texture_findings_export(&sandbox);

    let run = sandbox.run(&pass_through_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Sizes"),
        [
            hair_weights("03"),
            "Error texture_too_small [Keep] at Players/03 - A (file=skin.png)".to_owned(),
            hair_weights("04"),
            "Error texture_not_pow2 [Keep] at Players/04 - A (file=skin.png)".to_owned(),
            hair_weights("05"),
            "Error texture_type_mismatch [DropFolder] at Players/05 - A (file=skin.dds)".to_owned(),
            "Info export_identified [Keep] (team=/co/, id=714)".to_owned(),
            "Info team_colors_missing [Keep] ()".to_owned(),
            "Info kit_config_generated [Keep] at Kits/p1 ()".to_owned(),
            "Info kit_colors_derived [Keep] at Kits/p1 ()".to_owned(),
        ]
    );
    assert_eq!(run.exit_code(), 1);
    // Both kept folders are packed, the 3x3 texture and the 300x300 one converted as they are.
    let paths = compiled_paths(&sandbox);
    assert!(paths.contains(&co_face(3)), "{paths:?}");
    assert!(
        paths.contains(
            &"Asset/model/character/common/714/03 - A/sourceimages/#windx11/skin.ftex".to_owned()
        ),
        "{paths:?}"
    );
    assert!(paths.contains(&co_face(4)), "{paths:?}");
    assert!(
        paths.contains(
            &"Asset/model/character/common/714/04 - A/sourceimages/#windx11/skin.ftex".to_owned()
        ),
        "{paths:?}"
    );
    assert!(!paths.contains(&co_face(5)), "{paths:?}");
}

// TC-TEX-04
#[test]
fn a_texture_finding_drops_the_player_folder_naming_the_file() {
    let sandbox = Sandbox::new("tex_drop_texture_codec_unsupported");
    write_skin_player(
        &sandbox,
        "co - Drop",
        "05",
        "skin.dds",
        &texture_fixture("bc6h.dds"),
    );
    // A kit beside the folder, so the CPK is written and the folder's absence observable.
    sandbox.write("exports/co - Drop/Kits/p1/kit.dds", &tracer_kit());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Drop"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/p1 ()",
            "Error texture_codec_unsupported [DropFolder] at Players/05 - A (file=skin.dds)",
            "Info kit_colors_derived [Keep] at Kits/p1 ()",
        ]
    );
    assert_eq!(run.exit_code(), 1);
    // The kit is in the CPK; nothing of the folder is, neither its face nor its texture.
    let paths = compiled_paths(&sandbox);
    assert!(paths.contains(&kit_texture("u0714p1")), "{paths:?}");
    assert!(
        paths
            .iter()
            .all(|path| !path.starts_with("Asset/model/character/face/")
                && !path.starts_with(PLAYER_TEXTURES)),
        "{paths:?}"
    );
}

#[test]
fn a_kit_texture_finding_drops_the_kit() {
    let sandbox = Sandbox::new("tex_kit_drop");
    sandbox.write("exports/co - Kit/Kits/p1/kit.dds", &tracer_kit());
    sandbox.write(
        "exports/co - Kit/Kits/p1/kit_back.png",
        &texture_fixture("tiny.png"),
    );
    sandbox.write("exports/co - Kit/Kits/g1/kit.dds", &tracer_kit());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Kit"),
        [
            "Error texture_too_small [DropFolder] at Kits/p1 (file=kit_back.png)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/g1 ()",
            "Info kit_colors_derived [Keep] at Kits/g1 ()",
        ]
    );
    assert_eq!(run.exit_code(), 1);
    assert_eq!(compiled_kits(&sandbox), ["u0714g1"]);
}

// TC-CHK-04
#[test]
fn a_kit_texture_too_big_drops_its_kit_and_an_uncompressed_kit_is_bc7() {
    let sandbox = Sandbox::new("tex_kit_too_big");
    sandbox.write("exports/co - Kit/Kits/p1/kit.dds", &bc1_dds(4096, 4096));
    sandbox.write("exports/co - Kit/Kits/p2/kit.dds", &uncompressed_dds());
    let too_big = "Error kit_texture_too_big [DropFolder] at Kits/p1 (file=kit.dds)";
    let identified = "Info export_identified [Keep] (team=/co/, id=714)";

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);
    assert_eq!(
        findings_of(&check.messages(), "co - Kit"),
        [too_big, identified]
    );
    assert_eq!(check.exit_code(), 1);

    let compile = sandbox.run(&pes21_settings(&sandbox), &["compile"]);
    assert_eq!(
        findings_of(&compile.messages(), "co - Kit"),
        [
            too_big,
            identified,
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/p2 ()",
            "Info kit_colors_derived [Keep] at Kits/p2 ()"
        ]
    );
    assert_eq!(compile.exit_code(), 1);
    assert_eq!(compiled_kits(&sandbox), ["u0714p2"]);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let info = ftex::info(&entries[&kit_texture("u0714p2")]).unwrap();
    assert_eq!(info.format, PixelFormat::Bc7);
}

#[test]
fn a_common_texture_the_deep_pass_drops_takes_the_player_linking_it() {
    let sandbox = Sandbox::new("tex_common_link_dropped");
    let export = "exports/co - Link";
    sandbox.write(
        &format!("{export}/Players/05 - A/fcl_hair.fmdl"),
        &hair_model_naming("skin.dds"),
    );
    sandbox.write(&format!("{export}/Players/05 - A/skin.png.common"), b"");
    sandbox.write(
        &format!("{export}/Common/skin.png"),
        &texture_fixture("tiny.png"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["check"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Link"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Error texture_too_small [DropFile] at Common/skin.png (file=skin.png)",
            "Error link_target_dropped [DropFolder] at Players/05 - A (link=skin.png.common, target=Common/skin.png, finding=texture_too_small)",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
}

// TC-TEX-12
#[test]
fn a_common_texture_whose_task_fails_lets_the_players_linking_it_commit() {
    let sandbox = Sandbox::new("tex_common_task_failed");
    // The deep pass drops the 3x3 hair.png: the player linking it follows.
    let dropped = "exports/co - Dropped";
    sandbox.write(
        &format!("{dropped}/Players/05 - A/fcl_hair.fmdl"),
        &hair_model_naming("hair.dds"),
    );
    sandbox.write(&format!("{dropped}/Players/05 - A/hair.png.common"), b"");
    sandbox.write(
        &format!("{dropped}/Common/hair.png"),
        &texture_fixture("tiny.png"),
    );
    // Bytes no decoder reads give no deep-pass finding: the conversion fails the export's
    // Common task instead, which commits on its own.
    let failed = "exports/dbg - Failed";
    sandbox.write(
        &format!("{failed}/Players/05 - A/fcl_hair.fmdl"),
        &hair_model_naming("hair.dds"),
    );
    sandbox.write(&format!("{failed}/Players/05 - A/hair.dds.common"), b"");
    sandbox.write(&format!("{failed}/Common/hair.dds"), b"not a texture");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let messages = run.messages();
    assert_eq!(
        findings_of(&messages, "co - Dropped"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Error texture_too_small [DropFile] at Common/hair.png (file=hair.png)",
            "Error link_target_dropped [DropFolder] at Players/05 - A (link=hair.png.common, target=Common/hair.png, finding=texture_too_small)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ]
    );
    let failed_findings = findings_of(&messages, "dbg - Failed");
    assert_eq!(
        failed_findings[..3],
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/dbg/, id=790)",
            "Info team_colors_missing [Keep] ()",
        ]
    );
    let [failure] = &failed_findings[3..] else {
        panic!("{failed_findings:#?}");
    };
    assert!(
        failure.starts_with(
            "Error folder_pack_failed [DropFolder] at Common (error=hair.dds: cannot convert"
        ),
        "{failure}"
    );
    assert_eq!(run.exit_code(), 1);
    // Slot 05 of /co/ is left out; slot 05 of /dbg/ commits anyway.
    assert_eq!(compiled_players(&sandbox), [79005]);
    let paths = compiled_paths(&sandbox);
    assert!(
        paths.iter().all(|path| !path.contains("face/real/71405")),
        "{paths:?}"
    );
}

#[test]
fn a_common_texture_finding_leaves_that_file_out_and_the_rest_is_emitted() {
    let sandbox = Sandbox::new("tex_common_drop");
    sandbox.copy_tracer_face("exports/co - Common/Players/03 - A");
    sandbox.write(
        "exports/co - Common/Common/tiny.png",
        &texture_fixture("tiny.png"),
    );
    sandbox.write(
        "exports/co - Common/Common/hair.png",
        &texture_fixture("kit.png"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Common"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "Error texture_too_small [DropFile] at Common/tiny.png (file=tiny.png)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let common: Vec<&str> = entries
        .keys()
        .filter_map(|path| path.strip_prefix("Asset/model/character/common/714/sourceimages/"))
        .collect();
    assert_eq!(common, ["#windx11/hair.ftex"]);
}

#[test]
fn a_common_texture_and_a_shared_folder_s_texture_in_png_are_emitted_as_ftex() {
    let sandbox = Sandbox::new("tex_common_shared_png");
    let export = "exports/co - Png";
    sandbox.write(&format!("{export}/Players/05 - A/legs.fmdl.common"), b"");
    sandbox.write(
        &format!("{export}/Common/legs.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(
        &format!("{export}/Common/hair.png"),
        &texture_fixture("kit.png"),
    );
    sandbox.write(&format!("{export}/Players/05 - A/Crocs.boots"), b"");
    sandbox.write(
        &format!("{export}/Boots/Crocs/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(
        &format!("{export}/Boots/Crocs/sole.png"),
        &texture_fixture("kit.png"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Png"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Boots/Crocs (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Common/legs.fmdl (file=legs.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=legs.fmdl.common)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    for path in [
        "Asset/model/character/common/714/sourceimages/#windx11/hair.ftex",
        "Asset/model/character/boots/k0644/#windx11/sole.ftex",
    ] {
        let emitted = entries
            .get(path)
            .unwrap_or_else(|| panic!("no {path} among {:?}", entries.keys()));
        let info = ftex::info(emitted).unwrap();
        assert_eq!(info.format, PixelFormat::Bc7, "{path}");
        assert_eq!(
            (info.width, info.height, info.mipmaps),
            (256, 128, 9),
            "{path}"
        );
    }
}

// TC-CMN-04
#[test]
fn a_kit_number_without_its_texture_variant_gets_the_lowest_one_and_the_model_names_kit_n() {
    let sandbox = Sandbox::new("tex_kit_variants");
    let export = "exports/co - Variants";
    let player = format!("{export}/Players/05 - A");
    sandbox.write(
        &format!("{player}/fcl_hair.fmdl"),
        &hair_model_naming("pants_kitN.dds"),
    );
    // Two different images: the tracer's shirt and its kit.
    sandbox.write(
        &format!("{player}/pants_kit1.dds"),
        &tracer_player_file("shirt.dds"),
    );
    sandbox.write(&format!("{player}/pants_kit3.dds"), &tracer_kit());
    for slot in ["p1", "p2", "p3"] {
        sandbox.write(&format!("{export}/Kits/{slot}/kit.dds"), &tracer_kit());
    }

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Variants"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/p1 ()",
            "Info kit_config_generated [Keep] at Kits/p2 ()",
            "Info kit_config_generated [Keep] at Kits/p3 ()",
            "Warning kit_variant_missing [Keep] at Players/05 - A (texture=pants_kitN, kit=2, copied=pants_kit1)",
            "Info kit_colors_derived [Keep] at Kits/p1 ()",
            "Info kit_colors_derived [Keep] at Kits/p2 ()",
            "Info kit_colors_derived [Keep] at Kits/p3 ()",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let variant = |kit: u8| {
        let path = format!("{PLAYER_TEXTURES}/pants_kit{kit}.ftex");
        entries
            .get(&path)
            .unwrap_or_else(|| panic!("no {path} among {:?}", entries.keys()))
    };
    assert_eq!(variant(2), variant(1));
    assert_ne!(variant(3), variant(1));
    let model = FmdlFile::read(face_package(&entries).get("fcl_hair.fmdl").unwrap()).unwrap();
    let paths: Vec<(String, String)> = texture_paths(&model)
        .unwrap()
        .into_iter()
        .map(|path| (path.file_name, path.directory))
        .collect();
    assert!(
        paths.contains(&(
            "pants_kitN.dds".to_owned(),
            "/Assets/pes16/model/character/common/714/05 - A/sourceimages/".to_owned()
        )),
        "{paths:?}"
    );
}
