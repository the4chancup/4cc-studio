//! Live AATF's marks on the editor itself (`plans/save_editor.md`
//! "Configurable AATF rules", the Live AATF bullet's last two
//! sub-bullets; the draft's `rowMark`, `violationField` and `fieldMark`):
//! while the toggle is on and the dock's check holds one team's result,
//! the list's rows carry a `✖` or `✔` and the selected player's field a
//! violation is about carries a `✖`. `held_violations` is the one read
//! of the held check every mark goes through.

use team_widgets::violations::violation_line;

use crate::view::ActiveRules;
use crate::view::aatf_check::TeamsCheck;

/// The failing glyph (`✖`, U+2716) and the passing one (`✔`, U+2714):
/// egui's bundled emoji fonts hold them, and the theme keeps those fonts
/// in the fallback chain (DECISIONS.md "Live AATF's marks: where they
/// sit and their glyphs").
const CROSS: char = '\u{2716}';
const TICK: char = '\u{2714}';

/// The gap between a cross and what it sits beside (a field's box, a
/// label's text): one figure so every mark's spacing agrees.
pub(super) const MARK_GAP: f32 = 2.0;

/// The violations the marks draw: `check`'s held result while `live`
/// (the `live_aatf` toggle) is on and the active rules file loaded, when
/// the result is one team's successful check — the team id and its
/// violations, `None` otherwise. While a check is in flight the held
/// result stands (the plan: the marks keep the previous result); a
/// failed check or a rules file that does not load marks nothing.
pub(super) fn held_violations<'a>(
    live: bool,
    check: &'a TeamsCheck,
    rules: Option<&ActiveRules>,
) -> Option<(u32, &'a [aatf::Violation])> {
    if !live || !matches!(rules, Some(active) if active.rules.is_ok()) {
        return None;
    }
    let (key, checks) = check.result.as_ref()?;
    let [team_id] = key.ids.as_slice() else {
        return None;
    };
    let [Ok(violations)] = checks.as_slice() else {
        return None;
    };
    Some((*team_id, violations))
}

/// The field a violation's cross sits beside, the plan bullet's set:
/// `marked_field` maps a kind to one, `None` for the kinds about no
/// single field and the team's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum MarkedField {
    /// The Height box (`Height`).
    Height,
    /// The Age box (`Age`).
    Age,
    /// The Weight box (`Weight`).
    Weight,
    /// The Form box (`Form`).
    Form,
    /// The Injury resistance box (`InjuryResistance`).
    InjuryResistance,
    /// The Weak foot Usage box (`WeakFootUsage`).
    WeakFootUsage,
    /// The Weak foot Accuracy box (`WeakFootAccuracy`).
    WeakFootAccuracy,
    /// The ability stat's box (`Stat`).
    Stat(aatf::Stat),
    /// The Name field (`NameColor`).
    Name,
    /// The Registered position combo (`RegisteredNotA`, `PositionRange`).
    RegisteredPosition,
    /// The Playable positions grid (`BRating`, `GkSecondA`, `APositions`).
    PlayablePositions,
    /// The player's spot on the named preset's Tactics pitch
    /// (`FieldedNotA`; the pitch's cross is another step's, so the field
    /// exists but nothing draws it yet).
    Pitch,
    /// The Playing style combo (`PlayingStyleRange`).
    PlayingStyle,
    /// The Skills checkboxes (`SkillCardCap`, a required skill card).
    Skills,
    /// The COM playing styles (`ComStyleCap`, a required COM style).
    ComStyles,
    /// The Skills header's Cards left badge (`Cards`, `MinCards`).
    CardsLeft,
}

/// The field `kind`'s violation is about — the Live AATF bullet's
/// field-a-violation-is-about sub-bullet's mapping, one arm per kind so
/// a new kind does not compile past it. `None` for the kinds about no
/// single field (`NoTier`, `StatTotal`, `Special`, `Custom`,
/// `RosterOrder`, the suggestions), which mark the player's row alone,
/// and the team's own kinds, which mark the team's.
pub(super) fn marked_field(kind: &aatf::ViolationKind) -> Option<MarkedField> {
    use aatf::ViolationKind as K;
    match kind {
        K::Height { .. } => Some(MarkedField::Height),
        K::Age { .. } => Some(MarkedField::Age),
        K::Weight { .. } => Some(MarkedField::Weight),
        K::Form { .. } => Some(MarkedField::Form),
        K::InjuryResistance { .. } => Some(MarkedField::InjuryResistance),
        K::WeakFootUsage { .. } => Some(MarkedField::WeakFootUsage),
        K::WeakFootAccuracy { .. } => Some(MarkedField::WeakFootAccuracy),
        K::Stat { stat, .. } => Some(MarkedField::Stat(*stat)),
        K::NameColor { .. } => Some(MarkedField::Name),
        K::RegisteredNotA | K::PositionRange => Some(MarkedField::RegisteredPosition),
        K::BRating | K::GkSecondA | K::APositions { .. } => Some(MarkedField::PlayablePositions),
        K::FieldedNotA { .. } => Some(MarkedField::Pitch),
        K::PlayingStyleRange => Some(MarkedField::PlayingStyle),
        K::SkillCardCap { .. } => Some(MarkedField::Skills),
        K::ComStyleCap { .. } => Some(MarkedField::ComStyles),
        K::RequiredCard { card } => Some(match card {
            aatf::Card::Skill(_) => MarkedField::Skills,
            aatf::Card::ComStyle(_) => MarkedField::ComStyles,
        }),
        K::Cards { .. } | K::MinCards { .. } => Some(MarkedField::CardsLeft),
        K::NoTier
        | K::StatTotal { .. }
        | K::Special { .. }
        | K::Custom { .. }
        | K::RosterOrder { .. }
        | K::Unused { .. }
        | K::UnusedChoice { .. }
        | K::TierCount { .. }
        | K::BracketQuota { .. }
        | K::MedalSystem { .. }
        | K::ExtraHeight { .. }
        | K::Captain
        | K::Goalkeepers { .. }
        | K::TeamSetting { .. }
        | K::FormationShape { .. } => None,
    }
}

/// What a field's cross draws from: the held team's violations and the
/// selected player's slot and team (`violation_line`'s team id), bundled
/// so the view's field sites take one parameter. `None` at a site while
/// `held_violations` names another team or no result — the view's
/// players are not checked then.
#[derive(Clone, Copy)]
pub(super) struct Marks<'a> {
    /// The held team's violations, in the check's order.
    pub(super) violations: &'a [aatf::Violation],
    /// The selected player's roster slot.
    pub(super) slot: u8,
    /// His team.
    pub(super) team_id: u32,
}

impl Marks<'_> {
    /// `field_cross` on the bundle's parts.
    pub(super) fn cross(&self, ui: &mut egui::Ui, field: MarkedField) {
        field_cross(ui, self.violations, self.slot, self.team_id, field);
    }

    /// `paint_field_cross` on the bundle's parts.
    pub(super) fn paint(
        &self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        field: MarkedField,
    ) -> Option<egui::Response> {
        paint_field_cross(ui, rect, self.violations, self.slot, self.team_id, field)
    }
}

/// What a field's cross draws (from `field_mark`): the colour's key and
/// the tooltip.
struct FieldMark {
    /// An Error-severity violation is among them (the cross is
    /// `error_fg_color`; else `warn_fg_color`).
    error: bool,
    /// The violations' `violation_line`s, one per line, in the check's
    /// order.
    tooltip: String,
}

impl FieldMark {
    /// The cross's colour: the highest severity's among the violations.
    fn color(&self, visuals: &egui::Visuals) -> egui::Color32 {
        if self.error {
            visuals.error_fg_color
        } else {
            visuals.warn_fg_color
        }
    }
}

/// The Error and Warning violations on `slot` whose kind maps to `field`
/// as the cross beside it draws them (`None`: nothing is drawn — a
/// suggestion alone marks no field).
fn field_mark(
    violations: &[aatf::Violation],
    slot: u8,
    team_id: u32,
    field: MarkedField,
) -> Option<FieldMark> {
    let marked: Vec<&aatf::Violation> = violations
        .iter()
        .filter(|violation| {
            violation.slot == Some(slot)
                && violation.severity != aatf::Severity::Suggestion
                && marked_field(&violation.kind) == Some(field)
        })
        .collect();
    if marked.is_empty() {
        return None;
    }
    Some(FieldMark {
        error: marked
            .iter()
            .any(|violation| violation.severity == aatf::Severity::Error),
        tooltip: marked
            .iter()
            .map(|violation| violation_line(violation, team_id))
            .collect::<Vec<_>>()
            .join("\n"),
    })
}

/// The `✖` beside a field a violation is about, added to the layout (the
/// header rows, whose next item is nothing the cross could push): one
/// label, `error_fg_color` when one of them is an Error else
/// `warn_fg_color`, its tooltip their `violation_line`s, one per line.
/// Draws nothing when no Error or Warning violation on `slot` maps to
/// `field`. The sites laying the cross into the space between a label
/// and its field use `paint_field_cross` instead.
pub(super) fn field_cross(
    ui: &mut egui::Ui,
    violations: &[aatf::Violation],
    slot: u8,
    team_id: u32,
    field: MarkedField,
) {
    let Some(mark) = field_mark(violations, slot, team_id, field) else {
        return;
    };
    ui.add(egui::Label::new(
        egui::RichText::new(CROSS.to_string()).color(mark.color(ui.visuals())),
    ))
    .on_hover_text(mark.tooltip);
}

/// `field_cross` painted inside `rect` (left edge, vertically centred),
/// for the sites where the cross sits in reserved space — a label
/// column's tail, a counted label's, the gap before a box — so no
/// control's x moves. The glyph's own hover rect shows the tooltip;
/// its response returned for a caller suppressing an overlap's own
/// hover (the counted label's).
pub(super) fn paint_field_cross(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    violations: &[aatf::Violation],
    slot: u8,
    team_id: u32,
    field: MarkedField,
) -> Option<egui::Response> {
    let mark = field_mark(violations, slot, team_id, field)?;
    let color = mark.color(ui.visuals());
    let galley = ui.painter().layout_no_wrap(
        CROSS.to_string(),
        egui::TextStyle::Body.resolve(ui.style()),
        color,
    );
    let pos = egui::pos2(rect.min.x, rect.center().y - galley.size().y / 2.0);
    let glyph = egui::Rect::from_min_size(pos, galley.size());
    ui.painter().galley(pos, galley, color);
    Some(
        ui.interact(
            glyph,
            ui.scope_id().with(("live aatf field cross", field)),
            egui::Sense::hover(),
        )
        .on_hover_text(mark.tooltip),
    )
}

/// `glyph`'s galley width at the body font (`cross_width` and
/// `RowMark::width` read one through it).
fn glyph_width(ui: &egui::Ui, glyph: char) -> f32 {
    ui.painter()
        .layout_no_wrap(
            glyph.to_string(),
            egui::TextStyle::Body.resolve(ui.style()),
            egui::Color32::WHITE,
        )
        .size()
        .x
}

/// The `✖`'s galley width at the body font, what `paint_field_cross`
/// takes: a caller leaving it room in a gap reads it first.
pub(super) fn cross_width(ui: &egui::Ui) -> f32 {
    glyph_width(ui, CROSS)
}

/// What `row_mark` computes for a list row: failing or passing, and the
/// tooltip's counts.
pub(super) struct RowMark {
    /// An Error-severity violation is on the row (warnings and
    /// suggestions are not failures, as `check-aatf`'s exit code counts
    /// them).
    pub(super) failing: bool,
    /// `N error(s), M warning(s)` — a zero count left out, `No errors`
    /// when there are neither.
    pub(super) tooltip: String,
}

impl RowMark {
    /// The glyph and its colour: `✖` in `error_fg_color` when failing,
    /// `✔` in the theme's success green for text (`notice_color` keeps
    /// it readable on both themes) when passing.
    pub(super) fn glyph(&self, visuals: &egui::Visuals) -> (char, egui::Color32) {
        if self.failing {
            (CROSS, visuals.error_fg_color)
        } else {
            (
                TICK,
                studio_core::theme::notice_color(studio_core::NoticeKind::Success, visuals),
            )
        }
    }

    /// The glyph's galley width at the body font: a row leaves it room
    /// at its right end before truncating its text.
    pub(super) fn width(&self, ui: &egui::Ui) -> f32 {
        let (glyph, _) = self.glyph(ui.visuals());
        glyph_width(ui, glyph)
    }
}

/// A list row's mark over the violations on `slot` (`None`: the team's
/// own): failing when one is an Error, else passing.
pub(super) fn row_mark(violations: &[aatf::Violation], slot: Option<u8>) -> RowMark {
    let mut errors = 0usize;
    let mut warnings = 0usize;
    for violation in violations {
        if violation.slot != slot {
            continue;
        }
        match violation.severity {
            aatf::Severity::Error => errors += 1,
            aatf::Severity::Warning => warnings += 1,
            aatf::Severity::Suggestion => {}
        }
    }
    let mut parts = Vec::new();
    for (count, word) in [(errors, "error"), (warnings, "warning")] {
        if count > 0 {
            parts.push(format!(
                "{count} {word}{}",
                if count == 1 { "" } else { "s" }
            ));
        }
    }
    RowMark {
        failing: errors > 0,
        tooltip: if parts.is_empty() {
            "No errors".to_owned()
        } else {
            parts.join(", ")
        },
    }
}

/// `mark` painted at `rect`'s right end vertically centred; the glyph's
/// own hover rect shows the tooltip (the row's own hover, the name,
/// stays on the rest of the row — and a click on the glyph still selects
/// the row, the mark sensing hover only). `key` keeps the interact id
/// unique per row.
pub(super) fn paint_row_mark(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    mark: &RowMark,
    key: impl std::hash::Hash + std::fmt::Debug,
) -> egui::Response {
    let (glyph, color) = mark.glyph(ui.visuals());
    let galley = ui.painter().layout_no_wrap(
        glyph.to_string(),
        egui::TextStyle::Body.resolve(ui.style()),
        color,
    );
    let pos = egui::pos2(
        rect.max.x - galley.size().x,
        rect.center().y - galley.size().y / 2.0,
    );
    let glyph_rect = egui::Rect::from_min_size(pos, galley.size());
    ui.painter().galley(pos, galley, color);
    ui.interact(
        glyph_rect,
        ui.scope_id().with(("live aatf row mark", key)),
        egui::Sense::hover(),
    )
    .on_hover_text(mark.tooltip.as_str())
}
