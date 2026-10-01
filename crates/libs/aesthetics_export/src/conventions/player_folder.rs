//! The model-name suffix table the structure pass needs ("Model names: a free
//! part plus a suffix"). Pre-Fox type names join it with the deep pass.

/// The recognized model-name suffixes: what a model file's stem tail says the
/// model is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ModelSuffix {
    /// `face_high`.
    FaceHigh,
    /// `hair_high`.
    HairHigh,
    /// `oral`.
    Oral,
    /// `fcl_hair`.
    FclHair,
    /// `boots`.
    Boots,
    /// `glove_l` / `gloveL`.
    GloveL,
    /// `glove_r` / `gloveR`.
    GloveR,
    /// `handL`.
    HandL,
    /// `handR`.
    HandR,
}

/// The suffixes' underscore-free lowercase spellings.
const MODEL_SUFFIXES: [(&str, ModelSuffix); 9] = [
    ("facehigh", ModelSuffix::FaceHigh),
    ("hairhigh", ModelSuffix::HairHigh),
    ("oral", ModelSuffix::Oral),
    ("fclhair", ModelSuffix::FclHair),
    ("boots", ModelSuffix::Boots),
    ("glovel", ModelSuffix::GloveL),
    ("glover", ModelSuffix::GloveR),
    ("handl", ModelSuffix::HandL),
    ("handr", ModelSuffix::HandR),
];

/// A stem's trailing suffix: walking the stem from the end and skipping `_`,
/// its last `s.len()` non-underscore characters equal `s`
/// ASCII-case-insensitively and what precedes them is empty or ends with `_`.
/// `face_high`, `FaceHigh`, `x_hair_high`, `gloveL` match; `myface_high`,
/// `coral`, `torso` do not.
pub(crate) fn model_suffix(stem: &str) -> Option<ModelSuffix> {
    for (spelling, suffix) in MODEL_SUFFIXES {
        let mut reversed = stem.char_indices().rev().filter(|(_, c)| *c != '_');
        let mut start = stem.len();
        let mut matched = true;
        for want in spelling.chars().rev() {
            match reversed.next() {
                Some((index, c)) if c.eq_ignore_ascii_case(&want) => start = index,
                _ => {
                    matched = false;
                    break;
                }
            }
        }
        if matched && (start == 0 || stem[..start].ends_with('_')) {
            return Some(suffix);
        }
    }
    None
}

/// The explicitly-named face models (`face_high`, `hair_high`, `oral`): they
/// have no skeleton slot and cannot exist under `ingame_face`.
pub(crate) fn is_explicit_face(suffix: ModelSuffix) -> bool {
    matches!(
        suffix,
        ModelSuffix::FaceHigh | ModelSuffix::HairHigh | ModelSuffix::Oral
    )
}

/// The boots suffix.
pub(crate) fn is_boots(suffix: ModelSuffix) -> bool {
    suffix == ModelSuffix::Boots
}

/// The gloves suffixes (including the hand-skeleton models, which have
/// nowhere else to go on Fox).
pub(crate) fn is_gloves(suffix: ModelSuffix) -> bool {
    matches!(
        suffix,
        ModelSuffix::GloveL | ModelSuffix::GloveR | ModelSuffix::HandL | ModelSuffix::HandR
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_suffix_reads_the_stem_tail() {
        let cases: [(&str, Option<ModelSuffix>); 20] = [
            ("face_high", Some(ModelSuffix::FaceHigh)),
            ("FaceHigh", Some(ModelSuffix::FaceHigh)),
            ("x_hair_high", Some(ModelSuffix::HairHigh)),
            ("oral", Some(ModelSuffix::Oral)),
            ("fcl_hair", Some(ModelSuffix::FclHair)),
            ("kit_boots", Some(ModelSuffix::Boots)),
            ("gloveL", Some(ModelSuffix::GloveL)),
            ("glove_l", Some(ModelSuffix::GloveL)),
            ("glove_r", Some(ModelSuffix::GloveR)),
            ("handL", Some(ModelSuffix::HandL)),
            ("handR", Some(ModelSuffix::HandR)),
            ("kit_boots_", Some(ModelSuffix::Boots)),
            ("myface_high", None),
            ("coral", None),
            ("torso", None),
            ("ボコ", None),
            ("face", None),
            ("_oral", Some(ModelSuffix::Oral)),
            ("hair-high", None),
            ("x_-boots", None),
        ];
        for (stem, expected) in cases {
            assert_eq!(model_suffix(stem), expected, "{stem:?}");
        }
    }
}
