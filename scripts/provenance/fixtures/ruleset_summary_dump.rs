//! Each embedded ruleset read and written back by `libs/aatf`'s writer, into
//! `<out>/written_<name>.rhai`: the input `ruleset_summary_goldens.py` feeds the draft's
//! summary().
//!
//! A throwaway crate outside the workspace: `Cargo.toml` with `[workspace]` (empty, its own
//! workspace), `edition = "2024"` and `aatf = { path = "<repo>/crates/libs/aatf" }`, this file
//! as `src/main.rs`; then `cargo run --release -- <repo>/crates/libs/aatf/rulesets <out>`.
use std::{env, fs, path::Path};

fn main() {
    let args: Vec<String> = env::args().collect();
    let (rulesets, out) = (Path::new(&args[1]), Path::new(&args[2]));
    for name in [
        "autumn25", "autumn26", "dbc7", "spring26", "vgl25", "vgl26", "vgl27", "vtl10", "vtl11",
    ] {
        let text = fs::read_to_string(rulesets.join(format!("{name}.rhai"))).expect("reads");
        let ruleset = aatf::Ruleset::from_data_block(&text).expect("loads");
        fs::write(out.join(format!("written_{name}.rhai")), ruleset.to_data_block())
            .expect("writes");
    }
}
