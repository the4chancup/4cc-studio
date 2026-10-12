# 4cc Studio — Model conversion plan: Model format conversion

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Model Format Conversion

### SKL and glTF: native integration, not a companion

What exists of this is in `spec/model_conversion/conversion.md` "SKL and glTF: native integration, not a companion".

A glTF-authored model stores its skeleton natively in the glTF `skin` (field mapping in the format
plan, "Skeleton: native glTF skin, not a companion SKL"). When the target is Fox, `ir_to_fmdl`'s
caller reconstructs the `.skl` binary from the IR bones — which `gltf_to_ir` filled from the skin's
joint hierarchy and bind data, applying the inverse coordinate/local-to-model-space mapping.
