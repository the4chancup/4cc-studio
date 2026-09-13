//! Checks for what is known wrong: cross-field constraints, the collar's
//! 1-based field, raw sleeve values and version fit of the Name Y field.

use pes_version::PesVersion;

use crate::model::{KitConfig, LongSleeves, ShortSleeves};

/// How bad a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Deliberate or harmless; worth mentioning.
    Info,
    /// Stored but ignored by the game, or clamped on encode.
    Warning,
    /// Breaks the feature or violates a hard constraint.
    Error,
}

/// One validation rule that fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Finding {
    /// The stable finding code.
    pub code: &'static str,
    /// Its severity.
    pub severity: Severity,
}

/// Validates a config. Only what is known wrong is flagged: collar and winter
/// collar are 1-based fields so zero is invalid; shirt models other than
/// 144/160/176 are unheard of; cut-out short sleeves, undershirt-only long
/// sleeves and tight are only seen on models 144 and 160; raw sleeve values are
/// noted as Info; Name Y above 16 is stored but ignored on PES <= 20 and any
/// value wider than the version's field is clamped on encode.
pub fn validate(config: &KitConfig, version: PesVersion) -> Vec<Finding> {
    let mut findings = Vec::new();

    if config.shirt.collar == 0 || config.shirt.winter_collar == 0 {
        findings.push(Finding {
            code: "kit_collar_zero",
            severity: Severity::Error,
        });
    }
    if !matches!(config.shirt.model, 144 | 160 | 176) {
        findings.push(Finding {
            code: "kit_shirt_model_unknown",
            severity: Severity::Info,
        });
    }

    if matches!(config.shirt.short_sleeves, ShortSleeves::CutOut)
        && !matches!(config.shirt.model, 144 | 160)
    {
        findings.push(Finding {
            code: "kit_cut_out_requires_model_144_or_160",
            severity: Severity::Warning,
        });
    }
    if matches!(config.shirt.long_sleeves, LongSleeves::UndershirtOnly)
        && !matches!(config.shirt.model, 144 | 160)
    {
        findings.push(Finding {
            code: "kit_undershirt_only_requires_model_144_or_160",
            severity: Severity::Warning,
        });
    }
    if config.shirt.tight && !matches!(config.shirt.model, 144 | 160) {
        findings.push(Finding {
            code: "kit_tight_requires_model_144_or_160",
            severity: Severity::Warning,
        });
    }

    if matches!(config.shirt.short_sleeves, ShortSleeves::Raw(_))
        || matches!(config.shirt.long_sleeves, LongSleeves::Raw(_))
    {
        findings.push(Finding {
            code: "kit_unknown_sleeve_value",
            severity: Severity::Info,
        });
    }

    let name_y_max = if version <= PesVersion::Pes20 { 31 } else { 63 };
    if config.name.y > name_y_max || (version <= PesVersion::Pes20 && config.name.y > 16) {
        findings.push(Finding {
            code: "kit_name_y_clamped",
            severity: Severity::Warning,
        });
    }

    findings
}
