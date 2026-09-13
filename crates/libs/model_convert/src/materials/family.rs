//! Reading a material's shader family off a native shader name. The rules are the format
//! plan's "Shader families" inference column: Fox by substring, pre-Fox by exact name.

use super::MaterialFamily;

/// A family read off a native shader name. `approximate` when no rule named the shader and the
/// family is the closest fit (the importer reports it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InferredFamily {
    /// The inferred family.
    pub family: MaterialFamily,
    /// Whether the match was a fallback rather than a rule.
    pub approximate: bool,
}

/// From an FMDL shader name, case-insensitive substring rules in this order: `glass` → Glass;
/// `ggx` → Metal; `constant` or `lambert` → Shadeless; `blin`, `3ddf` or `hair` → Shaded;
/// any other `3dfw` → Shadeless approximate; anything else → Shaded approximate.
pub fn from_fox_shader(shader: &str) -> InferredFamily {
    let name = shader.to_lowercase();
    let exact = |family| InferredFamily {
        family,
        approximate: false,
    };
    if name.contains("glass") {
        exact(MaterialFamily::Glass)
    } else if name.contains("ggx") {
        exact(MaterialFamily::Metal)
    } else if name.contains("constant") || name.contains("lambert") {
        exact(MaterialFamily::Shadeless)
    } else if name.contains("blin") || name.contains("3ddf") || name.contains("hair") {
        exact(MaterialFamily::Shaded)
    } else if name.contains("3dfw") {
        InferredFamily {
            family: MaterialFamily::Shadeless,
            approximate: true,
        }
    } else {
        InferredFamily {
            family: MaterialFamily::Shaded,
            approximate: true,
        }
    }
}

/// From a `.mtl` shader name, exact rules: `Basic_*` (prefix), `Boots`, `Shirt_*`, `Pants_*` →
/// Shaded; `Shadeless`, `Constant`, `Overlay` → Shadeless; anything else (Konami's `Hair`,
/// `Wrinkle`, `Accessory`, `Skin`, `Eye`, ...) → Shaded approximate.
pub fn from_prefox_shader(shader: &str) -> InferredFamily {
    let shaded = shader.starts_with("Basic_")
        || shader == "Boots"
        || shader.starts_with("Shirt_")
        || shader.starts_with("Pants_");
    if shaded {
        InferredFamily {
            family: MaterialFamily::Shaded,
            approximate: false,
        }
    } else if matches!(shader, "Shadeless" | "Constant" | "Overlay") {
        InferredFamily {
            family: MaterialFamily::Shadeless,
            approximate: false,
        }
    } else {
        InferredFamily {
            family: MaterialFamily::Shaded,
            approximate: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fox_shader_rules() {
        let cases = [
            ("pes3dfw_glass2", MaterialFamily::Glass, false),
            ("fox3ddf_ggx", MaterialFamily::Metal, false),
            (
                "fox3dfw_constant_srgb_ndr_solid",
                MaterialFamily::Shadeless,
                false,
            ),
            ("eyeocclusion_lambert", MaterialFamily::Shadeless, false),
            ("fox3ddf_blin", MaterialFamily::Shaded, false),
            ("fox3ddf_blin_fuzzblock", MaterialFamily::Shaded, false),
            (
                "pes_3ddf_basic_color_translucent",
                MaterialFamily::Shaded,
                false,
            ),
            ("fox3ddf_hair", MaterialFamily::Shaded, false),
            ("fox3dfw_something_else", MaterialFamily::Shadeless, true),
            ("fox_3ddc_basic", MaterialFamily::Shaded, true),
            ("unrelated", MaterialFamily::Shaded, true),
        ];
        for (shader, family, approximate) in cases {
            assert_eq!(
                from_fox_shader(shader),
                InferredFamily {
                    family,
                    approximate
                },
                "{shader}"
            );
        }
    }

    #[test]
    fn prefox_shader_rules() {
        let cases = [
            ("Basic_C", MaterialFamily::Shaded, false),
            ("Basic_CNSR", MaterialFamily::Shaded, false),
            ("Boots", MaterialFamily::Shaded, false),
            ("Shirt_NB", MaterialFamily::Shaded, false),
            ("Pants_NB", MaterialFamily::Shaded, false),
            ("Shadeless", MaterialFamily::Shadeless, false),
            ("Constant", MaterialFamily::Shadeless, false),
            ("Overlay", MaterialFamily::Shadeless, false),
            ("Hair", MaterialFamily::Shaded, true),
            ("Wrinkle", MaterialFamily::Shaded, true),
            ("Skin", MaterialFamily::Shaded, true),
        ];
        for (shader, family, approximate) in cases {
            assert_eq!(
                from_prefox_shader(shader),
                InferredFamily {
                    family,
                    approximate
                },
                "{shader}"
            );
        }
    }
}
