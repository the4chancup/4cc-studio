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
use crate::compile::{cpk_entries, kit_texture, pes_settings, pes21_settings, tracer_player_file};
use crate::findings_of;

/// The player's texture home, the per-player common subfolder.
const PLAYER_TEXTURES: &str = "Asset/model/character/common/714/05 - A/sourceimages/#windx11";

/// The bytes of `tests/fixtures/textures/<name>` (that folder's `README.md`).
fn texture_fixture(name: &str) -> Vec<u8> {
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

/// Compiles the sandbox for PES `version`, asserting the export `name` reports only its
/// identity, and returns the entry at `path`.
fn compile_entry(sandbox: &Sandbox, name: &str, version: u8, path: &str) -> Vec<u8> {
    let run = sandbox.run(&pes_settings(sandbox, version), &["compile"]);
    assert_eq!(
        findings_of(&run.messages(), name),
        ["Info export_identified [Keep] (team=/co/, id=714)"],
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

#[test]
fn png_and_tga_kit_textures_are_emitted_as_ftex_under_the_kit_s_names() {
    let sandbox = Sandbox::new("tex_kit_formats");
    sandbox.write(
        "exports/co - Kit/Kits/p1/kit.png",
        &texture_fixture("kit.png"),
    );
    sandbox.write(
        "exports/co - Kit/Kits/p1/kit_back.tga",
        &texture_fixture("kit_back.tga"),
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
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=legs.fmdl.common)",
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
