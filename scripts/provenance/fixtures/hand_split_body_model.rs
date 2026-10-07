//! Provenance of `crates/tools/team_compiler/tests/fixtures/hand_split/body.model` and
//! `body.mtl` (step 4.14e3, TC-MOD-43): the pre-Fox twin of `body.fmdl`, the same strip
//! (`hand_split_body.rs`: 33 vertices, 40 faces, one mesh, one material, both hands weighted on
//! `skh_index_mcp_l`/`_r`) written by `ir_to_model`, with the `.mtl` the export gives it.
//!
//! Not part of the workspace. Run it as a bin of the same scratch crate, with `pes_model` as
//! one more path dependency: `hand_split_body_model <out model> <out mtl>`. It writes the two
//! files once it has read the model back, checked the pair with `pes_model::check`, imported it
//! again and split it to the counts of the FMDL (each glove 9 vertices and 8 faces, the body 21
//! and 24), and exported each part back to a `.model` keeping the material's name.

#[path = "hand_split_body.rs"]
#[allow(dead_code)]
mod fmdl_fixture;

use fmdl_fixture::{counts, strip};
use model_convert::Imported;
use model_convert::formats::pes_model::{ir_to_model, model_to_ir};
use model_convert::ops::hand_split::split_by_skeleton_group;
use pes_model::format::PreFoxModel;
use pes_model::format::mtl::MaterialSet;
use pes_model::model::Model;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [out_model, out_mtl] = args.as_slice() else {
        panic!("hand_split_body_model <out model> <out mtl>")
    };
    let exported = ir_to_model(&strip()).expect("the strip exports");
    for finding in &exported.findings {
        println!("export: {} {:?} {}", finding.code, finding.subject, finding.detail);
    }
    let written = exported
        .model
        .to_file()
        .expect("the model converts to its file")
        .write()
        .expect("the model writes");
    let mtl = exported.mtl.write();

    let read_back = Model::from_file(&PreFoxModel::read(&written).expect("a .model"))
        .expect("a model");
    let set = MaterialSet::read(&mtl).expect("a .mtl");
    let checked = pes_model::check::check_bundle(&read_back, &set);
    for finding in &checked {
        println!(
            "check: {} {:?} {:?} {}",
            finding.code, finding.severity, finding.subject, finding.count
        );
    }
    assert!(
        checked
            .iter()
            .all(|finding| finding.severity != pes_model::check::Severity::Error),
        "pes_model::check raises no Error"
    );
    assert_eq!(read_back.materials, ["body_mat"]);

    let Imported { model, findings } = model_to_ir(&read_back, &set).expect("the model imports");
    for finding in &findings {
        println!("import: {} {:?} {}", finding.code, finding.subject, finding.detail);
    }
    assert_eq!(counts(&model), (33, 40));
    let split = split_by_skeleton_group(&model);
    let glove_l = split.glove_l.as_ref().expect("a left glove");
    let glove_r = split.glove_r.as_ref().expect("a right glove");
    assert_eq!(counts(glove_l), (9, 8));
    assert_eq!(counts(glove_r), (9, 8));
    assert_eq!(counts(&split.body), (21, 24));
    // Each part exports back to a `.model` naming the strip's one material, so the source
    // `.mtl` defines every material a part uses (`hand_split.md` "Pipeline integration").
    for (name, part) in [("body", &split.body), ("glove_l", glove_l), ("glove_r", glove_r)] {
        let part = ir_to_model(part).unwrap_or_else(|error| panic!("{name} exports: {error}"));
        assert_eq!(part.model.materials, ["body_mat"], "{name}");
        part.model
            .to_file()
            .expect("the part converts to its file")
            .write()
            .expect("the part writes");
    }

    for (out, bytes) in [(out_model, &written), (out_mtl, &mtl)] {
        let temporary = format!("{out}.tmp");
        std::fs::write(&temporary, bytes).expect("the fixture writes");
        std::fs::rename(&temporary, out).expect("the fixture moves into place");
        println!("{out}: {} bytes", bytes.len());
    }
}
