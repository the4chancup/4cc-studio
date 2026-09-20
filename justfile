# 4cc Studio developer recipes. Rules: CONTRIBUTING.md "Testing and verification".
# A recipe is a list of commands, never logic; anything with a branch or a loop is a script
# under scripts/ that the recipe calls.

# Git for Windows' `sh` is not on PATH from a plain PowerShell, so recipes run under PowerShell
# there (the fallback CONTRIBUTING.md names). Recipe lines stay bare commands either way.
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

python := if os_family() == "windows" { "python" } else { "python3" }

# The four verification gates, in order. CI runs this same recipe.
gates: fmt-check clippy test wasm-check

# Gate 1: formatting
fmt-check:
    cargo fmt --all --check

# Gate 2: lints, warnings denied
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Gate 3: tests
test:
    cargo test --workspace

# Gate 4: studio_core and every lib crate check for wasm32 (list derived from the workspace)
wasm-check:
    {{python}} scripts/wasm_check.py

# Mutation run over one crate (converge); survivors are test gaps, equivalents or dead code
mutants crate:
    cargo mutants -p {{crate}} --jobs 2

# Mutation run over the lines changed since `base` (review of a landed diff)
mutants-diff base="HEAD":
    {{python}} scripts/mutants_diff.py {{base}}

# The python_bindings wheel: maturin build, then the smoke test with the wheel on sys.path
# (`just bindings <interpreter>` runs the smoke test under that Python, e.g. Blender's)
bindings interpreter=python:
    {{python}} scripts/bindings_check.py --python {{interpreter}}

# Guardrail 4 (fmdl/pes_model dependency denylist) and the license allowlist (deny.toml)
deps-check:
    {{python}} scripts/deps_check.py
    cargo deny check licenses
