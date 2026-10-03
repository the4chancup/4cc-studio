//! `compile` over textures in every accepted image format (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", step 5): a player folder's, a shared folder's, Common's
//! and a kit's, each converted for the target version's codec with its mip chain, observed
//! through the emitted FTEX's header and blocks.

use std::fs;
use std::path::Path;

use dds_convert::{SourceFormat, decode};
use fmdl::{FmdlFile, Model};
use ftex::PixelFormat;

use crate::common::Sandbox;
use crate::compile::{
    compiled_kits, cpk_entries, kit_texture, pes_settings, pes21_settings, tracer_kit,
    tracer_player_file,
};
use crate::findings_of;

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

/// The tracer's hair model naming `skin` instead of `shirt`, through `fmdl`'s model API.
fn hair_model_naming_skin() -> Vec<u8> {
    let file = FmdlFile::read(&tracer_player_file("fcl_hair.fmdl")).unwrap();
    let mut model = Model::from_file(&file).unwrap();
    for material in &mut model.materials {
        for (_, texture) in &mut material.textures {
            if texture.file_name == "shirt.dds" {
                texture.file_name = "skin.dds".to_owned();
            }
        }
    }
    model.to_file().unwrap().write()
}

/// Writes slot 05 of `exports/<export>` as the hair model naming `skin` beside the texture
/// file `texture_name` holding `bytes`.
fn write_skin_player(sandbox: &Sandbox, export: &str, texture_name: &str, bytes: &[u8]) {
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(
        &format!("{player}/fcl_hair.fmdl"),
        &hair_model_naming_skin(),
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
            "Info export_identified [Keep] (team=/co/, id=714)"
        ],
        "PES {version}"
    );
    assert_eq!(run.exit_code(), 0, "PES {version}");
    let mut entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    entries
        .remove(path)
        .unwrap_or_else(|| panic!("PES {version}: no {path} among {:?}", entries.keys()))
}

// TC-TEX-01
#[test]
fn a_png_skin_is_bc7_with_a_full_mip_chain_on_pes_21_and_bc3_on_pes_18() {
    let sandbox = Sandbox::new("tex_png_skin");
    write_skin_player(
        &sandbox,
        "co - Skin",
        "skin.png",
        &texture_fixture("skin.png"),
    );
    let skin = format!("{PLAYER_TEXTURES}/skin.ftex");

    for (version, format) in [(21, PixelFormat::Bc7), (18, PixelFormat::Bc3)] {
        let info = ftex::info(&compile_entry(&sandbox, "co - Skin", version, &skin)).unwrap();
        assert_eq!(info.format, format, "PES {version}");
        assert_eq!(info.mipmaps, 11, "PES {version}: 1024x1024 down to 1x1");
        assert_eq!((info.width, info.height), (1024, 1024), "PES {version}");
    }
}

// TC-TEX-02
#[test]
fn a_bc7_dds_skin_keeps_its_blocks_on_pes_21_and_is_bc3_on_pes_18() {
    let sandbox = Sandbox::new("tex_bc7_skin");
    let source = texture_fixture("bc7.dds");
    write_skin_player(&sandbox, "co - Skin", "skin.dds", &source);
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
            "Info kit_config_generated [Keep] at Kits/p1 ()",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
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

/// One texture finding on a player folder's texture: the file written under `name` with the
/// fixture `fixture`'s bytes, and the code expected.
struct FolderDrop {
    name: &'static str,
    fixture: &'static str,
    code: &'static str,
}

const FOLDER_DROPS: [FolderDrop; 4] = [
    FolderDrop {
        name: "skin.dds",
        fixture: "bc6h.dds",
        code: "texture_codec_unsupported",
    },
    FolderDrop {
        name: "tiny.png",
        fixture: "tiny.png",
        code: "texture_too_small",
    },
    FolderDrop {
        name: "odd.png",
        fixture: "odd.png",
        code: "texture_not_pow2",
    },
    // PNG bytes under a `.dds` name: renamed, not resaved.
    FolderDrop {
        name: "skin.dds",
        fixture: "portrait.png",
        code: "texture_type_mismatch",
    },
];

// TC-TEX-04
#[test]
fn a_texture_finding_drops_the_player_folder_naming_the_file() {
    for case in FOLDER_DROPS {
        let sandbox = Sandbox::new(&format!("tex_drop_{}", case.code));
        write_skin_player(
            &sandbox,
            "co - Drop",
            case.name,
            &texture_fixture(case.fixture),
        );
        // A kit beside the folder, so the CPK is written and the folder's absence observable.
        sandbox.write("exports/co - Drop/Kits/p1/kit.dds", &tracer_kit());

        let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

        assert_eq!(
            findings_of(&run.messages(), "co - Drop"),
            [
                "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)".to_owned(),
                "Info export_identified [Keep] (team=/co/, id=714)".to_owned(),
                "Info kit_config_generated [Keep] at Kits/p1 ()".to_owned(),
                format!(
                    "Error {} [DropFolder] at Players/05 - A (file={})",
                    case.code, case.name
                )
            ],
            "{}",
            case.code
        );
        assert_eq!(run.exit_code(), 1, "{}", case.code);
        // The kit is in the CPK; nothing of the folder is, neither its face nor its texture.
        let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
        let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
        assert!(
            paths.contains(&kit_texture("u0714p1").as_str()),
            "{paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|path| !path.starts_with("Asset/model/character/face/")
                    && !path.starts_with(PLAYER_TEXTURES)),
            "{}: {paths:?}",
            case.code
        );
    }
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
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info kit_config_generated [Keep] at Kits/p1 ()",
            "Info kit_config_generated [Keep] at Kits/g1 ()",
            "Error texture_too_small [DropFolder] at Kits/p1 (file=kit_back.png)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
    assert_eq!(compiled_kits(&sandbox), ["u0714g1"]);
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
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Error texture_too_small [DropFile] at Common (file=tiny.png)"
        ]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
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
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=legs.fmdl.common)"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
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
