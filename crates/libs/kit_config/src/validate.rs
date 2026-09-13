//! Checks for what is known wrong: cross-field constraints, the collar's
//! 1-based field, raw sleeve values, and every packed field's fit in the
//! target version (one table, `field_limits`, shared with `encode`).

use pes_version::PesVersion;

use crate::model::{KitConfig, LongSleeves, ShortSleeves, field_limits};

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

/// The value a field carried and the maximum the target version emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfRange {
    /// The field's TOML key.
    pub field: &'static str,
    /// The offending value.
    pub value: u8,
    /// The version's emission maximum.
    pub max: u8,
}

/// One validation rule that fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Finding {
    /// The stable finding code.
    pub code: &'static str,
    /// Its severity.
    pub severity: Severity,
    /// The field, value and limit for range findings; `None` otherwise.
    pub context: Option<OutOfRange>,
}

/// Validates a config. Only what is known wrong is flagged: collar and winter
/// collar are 1-based fields so zero is invalid; shirt models other than
/// 144/160/176 are unheard of; cut-out short sleeves, undershirt-only long
/// sleeves and tight are only seen on models 144 and 160; raw sleeve values are
/// noted as Info; every packed field wider than the target version's limit is
/// `kit_value_out_of_range` (clamped to that limit on encode); and a shirt
/// pattern of 12 or more is `kit_pattern_unsupported_pes15` when the target
/// is PES 15, whose pattern set stops at 5.
pub fn validate(config: &KitConfig, version: PesVersion) -> Vec<Finding> {
    let mut findings = Vec::new();

    if config.shirt.collar == 0 || config.shirt.winter_collar == 0 {
        findings.push(Finding {
            code: "kit_collar_zero",
            severity: Severity::Error,
            context: None,
        });
    }
    if !matches!(config.shirt.model, 144 | 160 | 176) {
        findings.push(Finding {
            code: "kit_shirt_model_unknown",
            severity: Severity::Info,
            context: None,
        });
    }

    if matches!(config.shirt.short_sleeves, ShortSleeves::CutOut)
        && !matches!(config.shirt.model, 144 | 160)
    {
        findings.push(Finding {
            code: "kit_cut_out_requires_model_144_or_160",
            severity: Severity::Warning,
            context: None,
        });
    }
    if matches!(config.shirt.long_sleeves, LongSleeves::UndershirtOnly)
        && !matches!(config.shirt.model, 144 | 160)
    {
        findings.push(Finding {
            code: "kit_undershirt_only_requires_model_144_or_160",
            severity: Severity::Warning,
            context: None,
        });
    }
    if config.shirt.tight && !matches!(config.shirt.model, 144 | 160) {
        findings.push(Finding {
            code: "kit_tight_requires_model_144_or_160",
            severity: Severity::Warning,
            context: None,
        });
    }

    if matches!(config.shirt.short_sleeves, ShortSleeves::Raw(_))
        || matches!(config.shirt.long_sleeves, LongSleeves::Raw(_))
    {
        findings.push(Finding {
            code: "kit_unknown_sleeve_value",
            severity: Severity::Info,
            context: None,
        });
    }

    for limit in field_limits(version) {
        let value = (limit.get)(config);
        if value > limit.max {
            findings.push(Finding {
                code: "kit_value_out_of_range",
                severity: Severity::Warning,
                context: Some(OutOfRange {
                    field: limit.field,
                    value,
                    max: limit.max,
                }),
            });
        }
    }

    // PES 15 reads only bits 5-7 of the pattern byte, a 3-bit index with no
    // entry above 5; emission maps 12-13 down to 10-11 but the result is
    // still not the pattern the author picked.
    if version == PesVersion::Pes15 && config.shirt.pattern >= 12 {
        findings.push(Finding {
            code: "kit_pattern_unsupported_pes15",
            severity: Severity::Warning,
            context: Some(OutOfRange {
                field: "shirt.pattern",
                value: config.shirt.pattern,
                max: 11,
            }),
        });
    }

    findings
}
