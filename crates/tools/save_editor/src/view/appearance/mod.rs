//! The Appearance tab (`plans/save_editor.md` "Editing UI", the Appearance
//! bullet and "Read-only aesthetics"; the draft's `appearanceTab` and
//! `paletteRow`): the ownership line and the "Edit aesthetics anyway"
//! switch over the five groups — Edit flags, Physique, Strip style,
//! Motion, Colours. Locked at every start of the tool (`SaveView`'s
//! `edit_aesthetics`), every control of the groups disabled under the
//! ownership line as hover; the switch re-enables them for the rest of the
//! run. A stripped player gets the draft's stripped line whatever the
//! switch. Edits go through `Session::edit_player` under the widget's id,
//! one undo entry per gesture, as `abilities.rs` does; a stored value past
//! a control's range draws as stored (`clamp_existing_to_range` off) and
//! changes only when edited.

pub(super) mod fields;

use pes_savefile::model::{PlayerAppearance, PlayerEditFlags, PlayerEntry, PlayerMotion};
use pes_savefile::schema::fields::PlayerField;
use pes_savefile::schema::ingame_face::IngameFaceField;
use pes_savefile::schema::limits::motion_top;
use pes_savefile::schema::player_max_value;
use pes_savefile::settings_toml::{Kind, SettingKey};
use pes_version::{Engine, PesVersion};

use crate::session::{Session, is_stripped};
use crate::view::abilities::{gating_tooltip, maybe_end_row, number, pairs_per_row};
use crate::view::appearance::fields::{
    MOTIONS, MotionField, PHYSIQUE, Palette, STRIP_ENUMS, combo_options, number_range, option_name,
    palette,
};
use crate::view::{SaveView, edit_player, selected_player};

/// `edit_flags`' Face checkbox.
pub(super) const FACE_LABEL: &str = "Face";
/// `edit_flags`' Hair checkbox.
pub(super) const HAIR_LABEL: &str = "Hair";
/// `edit_flags`' Physique checkbox, which the group's heading shares.
pub(super) const PHYSIQUE_LABEL: &str = "Physique";
/// `edit_flags`' Strip checkbox.
pub(super) const STRIP_LABEL: &str = "Strip";
/// `edit_flags`' Motion checkbox, which the group's heading shares.
pub(super) const MOTION_LABEL: &str = "Motion";
/// `edit_flags`' Base copy checkbox.
pub(super) const BASE_COPY_LABEL: &str = "Base copy";
/// `edit_flags`' copy-from id number.
pub(super) const COPY_FROM_ID_LABEL: &str = "Copy from ID";
/// The strip grid's boots id number.
pub(super) const BOOTS_ID_LABEL: &str = "Boots ID";
/// The strip grid's gloves id number.
pub(super) const GLOVES_ID_LABEL: &str = "Gloves ID";
/// The strip grid's shirttail-out checkbox.
pub(super) const SHIRTTAIL_OUT_LABEL: &str = "Shirttail out";
/// The strip grid's ankle-taping checkbox.
pub(super) const ANKLE_TAPING_LABEL: &str = "Ankle taping";
/// The wrist taping combo.
pub(super) const WRIST_TAPING_LABEL: &str = "Wrist taping";
/// The spectacles combo.
pub(super) const SPECTACLES_LABEL: &str = "Spectacles";
/// The player-gloves checkbox.
pub(super) const PLAYER_GLOVES_LABEL: &str = "Player gloves";
/// The dribbling motion's number.
pub(super) const DRIBBLING_MOTION_LABEL: &str = "Dribbling motion";

/// The tab for the selected player, under `info_strip`'s header.
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((_, _, player_id)) = selected_player(view) else {
        return;
    };
    let SaveView {
        session,
        edit_aesthetics,
        randomize_draw,
        generation,
        hide_ids,
        ..
    } = view;
    let generation = *generation;
    let Some(session) = session.as_mut() else {
        return;
    };
    let (version, stripped, player) = {
        let file = session.file();
        let Some(player) = file.player(player_id) else {
            return;
        };
        (file.version(), is_stripped(file, player_id), player.clone())
    };
    // Every widget id under the tab is the player's within this open, as on
    // the Abilities tab.
    ui.push_id((generation, player_id), |ui| {
        ownership_line(ui, version);
        ui.checkbox(edit_aesthetics, "Edit aesthetics anyway");
        if *edit_aesthetics {
            ui.weak(match version.engine() {
                Engine::PreFox => "A later patch overwrites whatever is edited by hand.",
                Engine::Fox => {
                    "On a stripped save the game still reads the motions; \
                     the look comes from the database tables."
                }
            });
        }
        if stripped {
            ui.colored_label(
                ui.visuals().warn_fg_color,
                "Stripped: the game takes his look from the database tables; \
                 the motions are still read.",
            );
        }
        ui.add_enabled_ui(*edit_aesthetics, |ui| {
            groups(
                ui,
                session,
                player_id,
                version,
                &player,
                randomize_draw.as_mut(),
                *hide_ids,
            );
        })
        .response
        .on_disabled_hover_text(ownership_tip(version));
    });
}

/// The five groups, the draft's order.
fn groups(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    version: PesVersion,
    player: &PlayerEntry,
    draw: &mut dyn FnMut(u8) -> u8,
    hide_ids: Result<fpc::HideIds, fpc::HideIdsError>,
) {
    ui.strong("Edit flags");
    ui.horizontal_wrapped(|ui| {
        edit_flags(
            ui,
            session,
            player_id,
            &player.edit_flags,
            player.appearance.base_copy_id,
        );
    });
    ui.separator();
    ui.strong(PHYSIQUE_LABEL);
    physique_grid(ui, session, player_id, &player.appearance);
    ui.separator();
    ui.strong("Strip style");
    strip_rows(ui, session, player_id, version, &player.appearance);
    ui.separator();
    ui.strong(MOTION_LABEL);
    motion_grid(ui, session, player_id, version, &player.motion, draw);
    ui.separator();
    ui.strong("Colours");
    colour_rows(ui, session, player_id, version, &player.appearance);
    ui.separator();
    ui.strong("Full Player Customization");
    fpc_group(ui, session, player_id, player, hide_ids);
}

/// The ownership row: the sentence itself (pre-Fox carries the patch step
/// and the Apply aesthetics patch link; the panel is a later slice's, so
/// the link opens nothing and its hover says so).
fn ownership_line(ui: &mut egui::Ui, version: PesVersion) {
    ui.horizontal_wrapped(|ui| {
        ui.label(ownership_tip(version));
        match version.engine() {
            Engine::PreFox => {
                ui.link("Apply aesthetics patch")
                    .on_hover_text("The patch panel is not built yet");
            }
            Engine::Fox => {}
        }
    });
}

/// The ownership sentence (the draft's `ownershipTip`): the line over the
/// tab, the disabled groups' hover and the locked Team-menu items'
/// (`menus.rs`).
pub(crate) fn ownership_tip(version: PesVersion) -> &'static str {
    match version.engine() {
        Engine::PreFox => {
            "Owned by the team's export: edit its settings.toml and recompile, \
             then apply the patch."
        }
        Engine::Fox => "Owned by the team's export: edit its settings.toml and recompile.",
    }
}

/// The FPC group, last on the tab (the draft's `fpcGroup`): the
/// classifier's reading (`is_fpc_player` under `hide_ids`, the settings'
/// hide IDs — a player hidden by his model's own boots ID reads as shown,
/// so no checkbox), the Hide body and Show body buttons with the draft's
/// hover texts, then the interference findings of `fpc::check` in its
/// order. Under the lock the whole group is disabled with the rest of the
/// tab — the reading and the findings still draw, since they read and do
/// not write. `hide_ids` an `Err` (a setting outside its band): the line
/// reads the error's text instead of "FPC: …" and both buttons are
/// disabled with it as their hover; the classifier has no IDs to read, so
/// no findings either.
fn fpc_group(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    player: &PlayerEntry,
    hide_ids: Result<fpc::HideIds, fpc::HideIdsError>,
) {
    let ids = match hide_ids {
        Ok(ids) => ids,
        Err(error) => {
            ui.horizontal_wrapped(|ui| {
                ui.label(error.to_string());
                for label in ["Hide body", "Show body"] {
                    ui.add_enabled(false, egui::Button::new(label))
                        .on_disabled_hover_text(error.to_string());
                }
            });
            return;
        }
    };
    let hidden = pes_savefile::ops::fpc::is_fpc_player(player, ids)
        .expect("a loaded player's face run holds the skin field");
    ui.horizontal_wrapped(|ui| {
        ui.label(if hidden {
            "FPC: hidden body"
        } else {
            "FPC: body shown"
        });
        if ui
            .button("Hide body")
            .on_hover_text(
                "The hide preset: long sleeves, tucked, short socks; \
                 the hide IDs from Settings where the IDs are 0 \
                 (a model's own IDs are kept); a Custom skin (PES 2015 to 2017) \
                 becomes Light. Ctrl+I; the whole team: Team menu",
            )
            .clicked()
        {
            session.apply_fpc(player_id, fpc::Preset::Hide, ids);
        }
        if ui
            .button("Show body")
            .on_hover_text(
                "The show preset: short sleeves, untucked, standard socks; \
                 boots and gloves 0 where they are the hide IDs; a Custom skin \
                 (PES 2015 to 2017) becomes Light",
            )
            .clicked()
        {
            session.apply_fpc(player_id, fpc::Preset::Unhide, ids);
        }
    });
    let style = pes_savefile::ops::fpc::strip_style(player)
        .expect("a loaded player's face run holds the strip fields");
    let findings = fpc::check(&style, hidden);
    if findings.is_empty() {
        ui.weak("No interference findings.");
    } else {
        for finding in &findings {
            let message = crate::messages::fpc_finding(finding);
            let text = format!(
                "{}: {}",
                message.severity.name(),
                crate::messages::when_text(&message)
            );
            match message.severity {
                studio_core::Severity::Error | studio_core::Severity::Fatal => {
                    ui.colored_label(ui.visuals().error_fg_color, text);
                }
                studio_core::Severity::Warning => {
                    ui.colored_label(ui.visuals().warn_fg_color, text);
                }
                studio_core::Severity::Info => {
                    ui.label(text);
                }
            }
        }
    }
}

/// The edit flags' row (the draft's Edit flags group): the six flags, then
/// the copy-from id. `base_copy` exists on PES 15 to 18 only; like a gated
/// field it keeps its place, disabled. Ticking it also sets the seven
/// gameplay flags (`plans/save_editor.md`'s "Is a copy?" row: a copy's
/// sections count as edited); unticking clears `base_copy` alone.
fn edit_flags(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    flags: &PlayerEditFlags,
    base_copy_id: u32,
) {
    checkbox(ui, session, player_id, FACE_LABEL, flags.face, |p, v| {
        p.edit_flags.face = v;
    });
    checkbox(ui, session, player_id, HAIR_LABEL, flags.hair, |p, v| {
        p.edit_flags.hair = v;
    });
    checkbox(
        ui,
        session,
        player_id,
        PHYSIQUE_LABEL,
        flags.physique,
        |p, v| {
            p.edit_flags.physique = v;
        },
    );
    checkbox(ui, session, player_id, STRIP_LABEL, flags.strip, |p, v| {
        p.edit_flags.strip = v;
    });
    checkbox(
        ui,
        session,
        player_id,
        MOTION_LABEL,
        flags.motion,
        |p, v| {
            p.edit_flags.motion = v;
        },
    );
    match flags.base_copy {
        Some(held) => checkbox(ui, session, player_id, BASE_COPY_LABEL, held, |p, v| {
            p.edit_flags.base_copy = Some(v);
            if v {
                p.edit_flags.abilities = true;
                p.edit_flags.com_styles = true;
                p.edit_flags.motion = true;
                p.edit_flags.playable_positions = true;
                p.edit_flags.registered_position = true;
                p.edit_flags.skills = true;
                p.edit_flags.playing_style = true;
            }
        }),
        None => {
            ui.add_enabled(false, egui::Checkbox::new(&mut false, BASE_COPY_LABEL))
                .on_disabled_hover_text("PES 15 to 18 only");
        }
    }
    number(
        ui,
        session,
        player_id,
        COPY_FROM_ID_LABEL,
        base_copy_id,
        0..=u32::MAX,
        |player, shown| {
            player.appearance.base_copy_id = shown;
        },
    );
}

/// The 14 physique fields in a wrapping grid, each on its key's `Signed`
/// scale (±7: `shown - limit` stores `shown + limit`).
fn physique_grid(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    appearance: &PlayerAppearance,
) {
    let per_row = pairs_per_row(ui, 190.0);
    egui::Grid::new("appearance physique").show(ui, |ui| {
        for (index, (label, key, get, set)) in PHYSIQUE.iter().enumerate() {
            let Kind::Signed { limit } = key.spec().kind else {
                unreachable!("the physique keys are Signed");
            };
            let limit = i32::from(limit);
            let stored = get(appearance);
            let mut shown = i32::from(stored) - limit;
            ui.label(*label);
            let response = ui.add(
                egui::DragValue::new(&mut shown)
                    .range(-limit..=limit)
                    .clamp_existing_to_range(false),
            );
            if response.changed() {
                edit_player(session, player_id, response.id.value(), |player| {
                    set(
                        &mut player.appearance,
                        u8::try_from(shown + limit)
                            .expect("the control keeps the value inside the key's range"),
                    );
                });
            }
            maybe_end_row(ui, index + 1, per_row);
        }
    });
}

/// The draft's Strip style group: the ids, the enums and checkboxes in the
/// grid, then the wrist-taping, spectacles and player-gloves rows with
/// their palettes.
fn strip_rows(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    version: PesVersion,
    appearance: &PlayerAppearance,
) {
    let per_row = pairs_per_row(ui, 190.0);
    egui::Grid::new("appearance strip").show(ui, |ui| {
        // The id fields take the record's own bound (`player_max_value`,
        // wider than `settings.toml`'s stock range): a compile-allocated
        // ID such as 627 is a value the save holds.
        number(
            ui,
            session,
            player_id,
            BOOTS_ID_LABEL,
            appearance.boots_id,
            0..=u32::try_from(
                player_max_value(version, PlayerField::BootsId)
                    .expect("the boots id is stored on every version"),
            )
            .expect("the boots id's bound fits u32"),
            |player, shown| {
                player.appearance.boots_id = shown;
            },
        );
        maybe_end_row(ui, 1, per_row);
        number(
            ui,
            session,
            player_id,
            GLOVES_ID_LABEL,
            appearance.gloves_id,
            0..=u32::try_from(
                player_max_value(version, PlayerField::GlovesId)
                    .expect("the gloves id is stored on every version"),
            )
            .expect("the gloves id's bound fits u32"),
            |player, shown| {
                player.appearance.gloves_id = shown;
            },
        )
        .on_hover_text("Any player's; 0 = normal hands");
        maybe_end_row(ui, 2, per_row);
        for (index, (label, key, get, set)) in STRIP_ENUMS.iter().enumerate() {
            combo(
                ui,
                session,
                player_id,
                label,
                *key,
                get(appearance),
                move |player, shown| {
                    set(&mut player.appearance, shown);
                },
            );
            maybe_end_row(ui, 3 + index, per_row);
        }
        checkbox(
            ui,
            session,
            player_id,
            SHIRTTAIL_OUT_LABEL,
            appearance.untucked,
            |p, v| {
                p.appearance.untucked = v;
            },
        );
        checkbox(
            ui,
            session,
            player_id,
            ANKLE_TAPING_LABEL,
            appearance.ankle_taping,
            |p, v| {
                p.appearance.ankle_taping = v;
            },
        );
    });
    ui.horizontal_wrapped(|ui| {
        combo(
            ui,
            session,
            player_id,
            WRIST_TAPING_LABEL,
            SettingKey::WristTaping,
            appearance.wrist_taping,
            |player, shown| {
                player.appearance.wrist_taping = shown;
            },
        );
        ui.label("Left");
        palette_row(
            ui,
            session,
            player_id,
            &palette(version, SettingKey::WristTapeColorLeft),
            appearance.wrist_tape_color_left,
            |player, shown| {
                player.appearance.wrist_tape_color_left = shown;
            },
        );
        ui.label("Right");
        palette_row(
            ui,
            session,
            player_id,
            &palette(version, SettingKey::WristTapeColorRight),
            appearance.wrist_tape_color_right,
            |player, shown| {
                player.appearance.wrist_tape_color_right = shown;
            },
        );
    });
    ui.horizontal_wrapped(|ui| {
        combo(
            ui,
            session,
            player_id,
            SPECTACLES_LABEL,
            SettingKey::Spectacles,
            appearance.spectacles_style,
            |player, shown| {
                player.appearance.spectacles_style = shown;
            },
        );
        palette_row(
            ui,
            session,
            player_id,
            &palette(version, SettingKey::SpectaclesColor),
            appearance.spectacles_color,
            |player, shown| {
                player.appearance.spectacles_color = shown;
            },
        );
    });
    ui.horizontal_wrapped(|ui| {
        let gloves = face(appearance, IngameFaceField::PlayerGloves);
        checkbox(
            ui,
            session,
            player_id,
            PLAYER_GLOVES_LABEL,
            gloves != 0,
            |p, v| {
                p.appearance
                    .ingame_face
                    .set(IngameFaceField::PlayerGloves, u8::from(v))
                    .expect("a loaded player's face run holds player gloves");
            },
        );
        let gloves_color = face(appearance, IngameFaceField::PlayerGlovesColor);
        palette_row(
            ui,
            session,
            player_id,
            &palette(version, SettingKey::GlovesColor),
            gloves_color,
            |player, shown| {
                player
                    .appearance
                    .ingame_face
                    .set(IngameFaceField::PlayerGlovesColor, shown)
                    .expect("a loaded player's face run holds the gloves colour");
            },
        );
    });
}

/// The draft's Motion group: the nine measured motions in the draft's
/// order, the dribbling motion between the kicks and the celebrations (PES
/// 20+, drawn disabled with the version tooltip before it).
fn motion_grid(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    version: PesVersion,
    motion: &PlayerMotion,
    draw: &mut dyn FnMut(u8) -> u8,
) {
    let per_row = pairs_per_row(ui, 190.0);
    egui::Grid::new("appearance motion").show(ui, |ui| {
        let mut pair = 0usize;
        for field in &MOTIONS[..7] {
            motion_number(ui, session, player_id, version, field, motion);
            pair += 1;
            maybe_end_row(ui, pair, per_row);
        }
        // The dribbling motion sits between the kicks and the celebrations
        // (the draft's second row's last cell); a version without it draws
        // it disabled at 0 with the `PES NN+` tooltip.
        ui.label(DRIBBLING_MOTION_LABEL);
        match motion.dribbling {
            Some(stored) => {
                let mut shown = stored;
                let range = number_range(SettingKey::Dribbling);
                let response = ui.add(
                    egui::DragValue::new(&mut shown)
                        .range(range)
                        .clamp_existing_to_range(false),
                );
                if response.changed() {
                    edit_player(session, player_id, response.id.value(), |player| {
                        player.motion.dribbling = Some(shown);
                    });
                }
            }
            None => {
                ui.add_enabled_ui(false, |ui| {
                    ui.add(
                        egui::DragValue::new(&mut 0u8).range(number_range(SettingKey::Dribbling)),
                    );
                })
                .response
                .on_disabled_hover_text(gating_tooltip(PlayerField::DribblingMotion));
            }
        }
        pair += 1;
        maybe_end_row(ui, pair, per_row);
        for field in &MOTIONS[7..] {
            motion_number(ui, session, player_id, version, field, motion);
            pair += 1;
            maybe_end_row(ui, pair, per_row);
        }
    });
    // Every motion but the dribbling motion, from 1 to the version's shown
    // top, one undoable edit (DECISIONS.md 2026-10-22's ruling).
    if ui
        .button("Randomize")
        .on_hover_text("Random motion values (the dribbling motion is left as it is)")
        .clicked()
    {
        randomize(session, player_id, version, draw);
    }
}

/// One motion's label/`DragValue` pair on the shown scale: a 1-based motion
/// shows 1 to `motion_top` + 1 (stored one below), a celebration 0 to the
/// top (stored as shown).
fn motion_number(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    version: PesVersion,
    field: &MotionField,
    motion: &PlayerMotion,
) {
    let top = motion_top(version, field.field).expect("a motion field has a top");
    let stored = (field.get)(motion);
    let (shown, range) = if field.one_based {
        (stored + 1, 1..=top + 1)
    } else {
        (stored, 0..=top)
    };
    let set = field.set;
    let one_based = field.one_based;
    number(
        ui,
        session,
        player_id,
        field.label,
        shown,
        range,
        move |player, shown| {
            set(
                &mut player.motion,
                if one_based { shown - 1 } else { shown },
            );
        },
    );
}

/// Randomize (the draft's button): every motion but the dribbling motion
/// drawn from 1 to its shown top, the writes landing as one undo entry.
fn randomize(
    session: &mut Session,
    player_id: u32,
    version: PesVersion,
    draw: &mut dyn FnMut(u8) -> u8,
) {
    edit_player(
        session,
        player_id,
        egui::Id::unique(("appearance randomize", player_id)).value(),
        |player| {
            for field in MOTIONS {
                let top = motion_top(version, field.field).expect("a motion field has a top");
                let shown_top = if field.one_based { top + 1 } else { top };
                let drawn = draw(shown_top);
                (field.set)(
                    &mut player.motion,
                    if field.one_based { drawn - 1 } else { drawn },
                );
            }
        },
    );
}

/// The draft's Colours group: skin and iris, each its palette's entries as
/// swatches. The skin palette drops the Custom entry on Fox (PES 15 to 17
/// alone have it, the three pre-Fox versions).
fn colour_rows(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    version: PesVersion,
    appearance: &PlayerAppearance,
) {
    for key in [SettingKey::SkinColor, SettingKey::IrisColor] {
        let field = key
            .face_field()
            .expect("a colour palette's is a face field");
        let palette = palette(version, key);
        let stored = face(appearance, field);
        ui.horizontal_wrapped(|ui| {
            ui.label(palette.label);
            palette_row(ui, session, player_id, &palette, stored, |player, shown| {
                player
                    .appearance
                    .ingame_face
                    .set(field, shown)
                    .expect("a loaded player's face run holds the tab's colour fields");
            });
        });
    }
}

/// One label/checkbox pair; a change writes `set` through `edit_player`
/// under the widget's id.
fn checkbox(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    label: &'static str,
    held: bool,
    set: impl FnOnce(&mut PlayerEntry, bool),
) {
    let mut value = held;
    let response = ui.checkbox(&mut value, label);
    if response.changed() {
        edit_player(session, player_id, response.id.value(), |player| {
            set(player, value);
        });
    }
}

/// One label/`ComboBox` pair over `key`'s option labels (`combo_options`);
/// a stored index past the list shows its number, a pick writes it.
fn combo(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    label: &'static str,
    key: SettingKey,
    stored: u8,
    set: impl FnOnce(&mut PlayerEntry, u8),
) {
    ui.label(label);
    let options = combo_options(key);
    let mut picked = stored;
    let selected = option_name(key, stored);
    let combo = egui::ComboBox::from_id_salt(("appearance", label))
        .selected_text(selected)
        .show_ui(ui, |ui| {
            for (index, option) in options.iter().enumerate() {
                ui.selectable_value(
                    &mut picked,
                    u8::try_from(index).expect("the key's options fit a u8"),
                    option,
                );
            }
        });
    if picked != stored {
        edit_player(session, player_id, combo.response.id.value(), |player| {
            set(player, picked)
        });
    }
}

/// One palette row (the draft's `paletteRow`): a framed swatch per palette
/// entry 0 to `palette.top`, the entry's name or index in its widget label
/// and hover; a stored value past `top` draws its number after the
/// swatches. A click writes the swatch's index through `edit_player`.
fn palette_row(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    palette: &Palette,
    stored: u8,
    set: impl Fn(&mut PlayerEntry, u8),
) {
    // The swatch names once per row (each is `option_name`'s entry: the
    // palette keys' `Kind::Number` ranges start at 0, so `combo_options`
    // holds one entry per index up to the key's top, and every drawn
    // index has one).
    let names = combo_options(palette.key);
    ui.push_id(("appearance palette", palette.label), |ui| {
        for (index, rgb) in palette.colors[..=usize::from(palette.top)]
            .iter()
            .enumerate()
        {
            let name = &names[index];
            let widget_label = format!("{} {name}", palette.label);
            let response =
                swatch(ui, *rgb, index == usize::from(stored), &widget_label).on_hover_text(name);
            if response.clicked() {
                edit_player(session, player_id, response.id.value(), |player| {
                    set(
                        player,
                        u8::try_from(index).expect("a palette has at most eleven entries"),
                    );
                });
            }
        }
        if stored > palette.top {
            ui.weak(stored.to_string());
        }
    });
}

/// One palette entry's framed square (the draft's `.swatch`): the colour in
/// a dark frame, the held entry outlined as `.on` is.
fn swatch(ui: &mut egui::Ui, rgb: [u8; 3], held: bool, label: &str) -> egui::Response {
    let size = egui::vec2(14.0, 14.0);
    let (rect, response) = ui.allocate_at_least(size, egui::Sense::click());
    response.widget_info(|| {
        let mut info =
            egui::WidgetInfo::labeled(egui::Role::ColorWell, ui.is_enabled(), label.to_owned());
        info.selected = Some(held);
        info
    });
    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(2),
            egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
        );
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::same(2),
            egui::Stroke::new(1.0, egui::Color32::BLACK),
            egui::StrokeKind::Inside,
        );
        if held {
            ui.painter().rect_stroke(
                rect.expand(1.0),
                egui::CornerRadius::same(2),
                egui::Stroke::new(1.0, ui.visuals().selection.stroke.color),
                egui::StrokeKind::Outside,
            );
        }
    }
    response
}

/// `field`'s value inside `appearance`'s ingame-face run: the run a loaded
/// save's players always carry, and a field `layout()` names for every
/// version (the four the tab edits are `None`, both layouts).
fn face(appearance: &PlayerAppearance, field: IngameFaceField) -> u8 {
    appearance
        .ingame_face
        .get(field)
        .expect("a loaded player's face run holds the tab's colour fields")
}

/// A xorshift64 (13, 7, 17) draw in `1..=top` from `seed` (the Randomize
/// source). The picks need only be unpredictable to a member, not
/// cryptographic.
pub(crate) fn xorshift_draw(seed: u64) -> impl FnMut(u8) -> u8 {
    let mut state = seed;
    move |top| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        1 + u8::try_from(state % u64::from(top.max(1))).expect("a remainder below a u8 top fits u8")
    }
}

/// `xorshift_draw` seeded from the wall clock (`SaveView::new` wires it as
/// `randomize_draw`; the tests substitute a fixed source).
pub(crate) fn seeded_draw() -> impl FnMut(u8) -> u8 {
    xorshift_draw(clock_seed())
}

/// The wall-clock seed for `seeded_draw`, forced odd: a zero state draws 1
/// forever.
fn clock_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| odd_seed(u64::from(since.subsec_nanos()), since.as_secs()))
        .unwrap_or(0x9E37_79B9_7F4A_7C15)
}

/// `clock_seed`'s arithmetic as a pure function (its test needs even
/// inputs, which a real clock does not hand it on demand): `nanos ^ secs`
/// forced odd, a zero state drawing 1 forever.
fn odd_seed(nanos: u64, secs: u64) -> u64 {
    nanos ^ secs | 1
}

#[cfg(test)]
mod tests {
    use super::{clock_seed, odd_seed, seeded_draw, xorshift_draw};

    // The Randomize generator is xorshift64 (13, 7, 17): each draw is
    // `1 + state % top` after one round of the three shifts. The first
    // state from seed 1 is 1082269761; the three draws' states are stepped
    // out here by hand, so the test never calls the code under test.
    #[test]
    fn xorshift_draw_returns_the_states_remainders_plus_one() {
        let mut state = 1u64;
        let mut expected = Vec::new();
        for _ in 0..3 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            expected.push(1 + u8::try_from(state % 162).expect("a remainder fits u8"));
        }
        let mut first_state = 1u64;
        first_state ^= first_state << 13;
        first_state ^= first_state >> 7;
        first_state ^= first_state << 17;
        assert_eq!(first_state, 1082269761, "the pinned first state");
        let mut draw = xorshift_draw(1);
        let drawn: Vec<u8> = (0..3).map(|_| draw(162)).collect();
        assert_eq!(drawn, expected);
    }

    // The clock seed must be odd (a zero state draws 1 forever).
    #[test]
    fn the_clock_seed_is_odd() {
        assert_eq!(clock_seed() % 2, 1, "an even or zero seed draws 1 forever");
    }

    // The forcing is the `| 1`: on even inputs `& 1` would hand back 0.
    #[test]
    fn the_odd_seed_is_odd_on_even_inputs() {
        // Exact values: an odd-only check passes a constant 1, and on
        // (2, 4) `| 1` -> `^ 1` gives 7 too; on (1, 2) it gives 2.
        assert_eq!(odd_seed(2, 4), 7, "2 ^ 4 forced odd");
        assert_eq!(odd_seed(1, 2), 3, "1 ^ 2, already odd, kept");
    }

    // `seeded_draw` returns values in `1..=top`, both ends occurring over a
    // long enough run (the source need only be unpredictable to a member).
    #[test]
    fn the_seeded_draw_covers_the_whole_range() {
        let mut draw = seeded_draw();
        for top in [1u8, 5, 162] {
            let (mut low, mut high) = (u8::MAX, u8::MIN);
            for _ in 0..10_000 {
                let value = draw(top);
                assert!((1..=top).contains(&value), "{value} out of 1..={top}");
                low = low.min(value);
                high = high.max(value);
            }
            assert_eq!((low, high), (1, top), "top {top}: both ends occur");
        }
    }
}
