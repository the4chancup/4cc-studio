//! The `settings.toml` document text: the header, `PlayerSettings::parse`
//! and `to_toml`, the in-place `update_toml` rewrite, and the appearance
//! table readers/emitters `team_toml` shares.

use toml_edit::{DocumentMut, Item, Value};

use super::keys::{self};
use super::{
    AppearanceSettings, KeyTable, Kind, NameSetting, PlayerSettings, SettingKey, SettingsError,
    get_appearance, set_appearance,
};

// The TOML half: `parse` reads a `settings.toml` text, `to_toml` emits the
// template (the plan's key-table block verbatim for a fully set settings),
// `update_toml` rewrites the `Some` values inside an existing document while
// preserving its comments and formatting.

/// The header of a generated `settings.toml`: the plan block's five comment
/// lines and the two-line `name` comment, verbatim.
const HEADER: &str = "\
# settings.toml, inside a player folder. Every key is optional: an absent key leaves
# that savefile setting untouched (boots_id/gloves_id: see their comments). A commented
# key shows what can be set and its range. The file never references the export's
# models: what a player wears is decided by the folder contents (models present, link
# files pointing at shared folders); boots_id/gloves_id only name the game's stock models.

# true = derive from the folder name (\"15 - Snuffy\" gives \"Snuffy\"; the whole folder
# name for players.txt-mapped folders); \"text\" = write as is; absent = leave untouched.
";

/// The dotted path of a key ("appearance.strip.sleeves"; a top-level key is
/// just its name).
pub(super) fn dotted(spec: &keys::KeySpec) -> String {
    if spec.table == KeyTable::Top {
        spec.name.to_string()
    } else {
        format!("{}.{}", spec.table.path(), spec.name)
    }
}

/// The stored bound for `key` in `stored_ranges` (team TOML) mode: the
/// widest bit width the key's field is stored at across the version tables —
/// a `Player` source via `schema::widest_bit_width`, a `Face` source via the
/// ingame-face table. The stored value must fit it; the editor ranges the
/// strict mode enforces do not apply.
fn stored_bound(key: SettingKey) -> i64 {
    let width = match key.source() {
        keys::Source::Player(field) => crate::schema::widest_bit_width(field),
        keys::Source::Face(field) => crate::schema::ingame_face::INGAME_FACE_FIELDS
            .iter()
            .find(|spec| spec.field == field)
            .map(|spec| spec.bit_width)
            .expect("every face field is in the table"),
    };
    (1i64 << width) - 1
}

/// A key's TOML value for a stored `u8` (label string, signed physique
/// number, 1-based motion number, bool); `OutOfRange` for a value written
/// directly into a public field that the kind cannot represent. In
/// `stored_ranges` mode the bound is the field's stored width, and a
/// width-valid value with no label emits as the integer.
fn toml_form(key: SettingKey, stored: u8, stored_ranges: bool) -> Result<Value, SettingsError> {
    let spec = &key.spec();
    if stored_ranges {
        let max = stored_bound(key);
        if i64::from(stored) > max {
            return Err(SettingsError::OutOfRange {
                key: dotted(spec),
                value: i64::from(stored),
                range: format!("0 to {max}"),
            });
        }
    } else if !spec.kind.accepts_stored(stored) {
        return Err(SettingsError::OutOfRange {
            key: dotted(spec),
            value: i64::from(stored),
            range: range_text(spec.kind),
        });
    }
    Ok(match spec.kind {
        Kind::Number { .. } => i64::from(stored).into(),
        Kind::Signed7 => (i64::from(stored) - 7).into(),
        Kind::OneBased { .. } => (i64::from(stored) + 1).into(),
        Kind::Bool => (stored != 0).into(),
        // Strict mode's accepts_stored bounds `stored` to a label; stored
        // mode falls back to the integer for a width-valid value with none.
        Kind::Labels(labels) => labels
            .get(usize::from(stored))
            .map_or_else(|| i64::from(stored).into(), |label| (*label).into()),
    })
}

/// The range text a `Kind` reports (`"0 to 7"`, `"-7 to 7"`, `"1 to 10"`,
/// `"true, false"`, or the quoted label list).
pub(super) fn range_text(kind: Kind) -> String {
    match kind {
        Kind::Number { min, max } => format!("{min} to {max}"),
        Kind::Signed7 => "-7 to 7".to_string(),
        Kind::OneBased { max } => format!("1 to {max}"),
        Kind::Bool => "true, false".to_string(),
        Kind::Labels(labels) => labels
            .iter()
            .map(|label| format!("\"{label}\""))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// The value an unset key's commented line shows.
fn neutral_text(kind: Kind) -> String {
    match kind {
        Kind::Number { .. } | Kind::Signed7 => "0".to_string(),
        Kind::OneBased { .. } => "1".to_string(),
        Kind::Bool => "false".to_string(),
        Kind::Labels(labels) => Value::from(labels[0]).to_string(),
    }
}

/// `body` padded so `#` starts at character 33 (1-based); a body longer than
/// the pad still gets one space before `#`. An empty comment leaves `body`
/// unpadded.
pub(crate) fn padded(body: &str, comment: &str) -> String {
    if comment.is_empty() {
        return body.to_string();
    }
    let pad = 32_usize.saturating_sub(body.len()).max(1);
    format!("{body}{}# {comment}", " ".repeat(pad))
}

/// The key's dotted path under `root` (`root` = the appearance table's own
/// path: "appearance" in settings.toml, "players.03.appearance" in team.toml);
/// a `Top` key never reaches it — its callers filter `Top` first.
pub(crate) fn leaf_path(root: &str, spec: &keys::KeySpec) -> String {
    match spec.table.sub_table() {
        None => format!("{root}.{}", spec.name),
        Some(sub) => format!("{root}.{sub}.{}", spec.name),
    }
}

/// The item at a key's `(table, name)` inside the `appearance` table, `None`
/// when absent; `path` is the appearance table's dotted path for error keys;
/// a `Top` key never reaches it — its callers filter `Top` first.
fn lookup<'a>(
    appearance: &'a dyn toml_edit::TableLike,
    spec: &keys::KeySpec,
    path: &str,
) -> Result<Option<&'a Item>, SettingsError> {
    let table = match spec.table.sub_table() {
        None => appearance,
        Some(sub) => {
            let Some(item) = appearance.get(sub) else {
                return Ok(None);
            };
            match item.as_table_like() {
                Some(table) => table,
                None => {
                    return Err(SettingsError::WrongType {
                        key: format!("{path}.{sub}"),
                        expected: "a table",
                    });
                }
            }
        }
    };
    Ok(table.get(spec.name))
}

/// The stored `u8` behind one present item, range-checked per its `Kind`.
/// `stored_ranges` is `team_toml`'s mode: the editor range is documentation,
/// and any value the field's widest stored bit width can hold is accepted
/// (a real save's face types and celebration numbers go past the selectable
/// range, but not past the storage).
fn value(
    dotted: &str,
    key: SettingKey,
    item: &Item,
    stored_ranges: bool,
) -> Result<u8, SettingsError> {
    let kind = key.spec().kind;
    let wide = |value: i64| -> Result<u8, SettingsError> {
        let max = stored_bound(key);
        if !(0..=max).contains(&value) {
            return Err(SettingsError::OutOfRange {
                key: dotted.to_string(),
                value,
                range: format!("0 to {max}"),
            });
        }
        Ok(u8::try_from(value).expect("bounded by max"))
    };
    let integer = |expected: &'static str| -> Result<i64, SettingsError> {
        item.as_value()
            .and_then(|v| v.as_integer())
            .ok_or_else(|| SettingsError::WrongType {
                key: dotted.to_string(),
                expected,
            })
    };
    match kind {
        Kind::Number { min, max } => {
            let value = integer("an integer")?;
            if stored_ranges {
                return wide(value);
            }
            if !(i64::from(min)..=i64::from(max)).contains(&value) {
                return Err(SettingsError::OutOfRange {
                    key: dotted.to_string(),
                    value,
                    range: range_text(kind),
                });
            }
            Ok(u8::try_from(value).expect("the range check bounds it"))
        }
        Kind::Signed7 => {
            let value = integer("an integer")?;
            if stored_ranges {
                let Some(shifted) = value.checked_add(7) else {
                    return Err(SettingsError::OutOfRange {
                        key: dotted.to_string(),
                        value,
                        range: format!("0 to {}", stored_bound(key)),
                    });
                };
                return wide(shifted);
            }
            if !(-7..=7).contains(&value) {
                return Err(SettingsError::OutOfRange {
                    key: dotted.to_string(),
                    value,
                    range: range_text(kind),
                });
            }
            Ok(u8::try_from(value + 7).expect("in 0..=14"))
        }
        Kind::OneBased { max } => {
            let value = integer("an integer")?;
            if stored_ranges {
                let Some(shifted) = value.checked_sub(1) else {
                    return Err(SettingsError::OutOfRange {
                        key: dotted.to_string(),
                        value,
                        range: format!("0 to {}", stored_bound(key)),
                    });
                };
                return wide(shifted);
            }
            if !(1..=i64::from(max)).contains(&value) {
                return Err(SettingsError::OutOfRange {
                    key: dotted.to_string(),
                    value,
                    range: range_text(kind),
                });
            }
            Ok(u8::try_from(value - 1).expect("in 0..=max-1"))
        }
        Kind::Bool => item
            .as_value()
            .and_then(|v| v.as_bool())
            .map(u8::from)
            .ok_or_else(|| SettingsError::WrongType {
                key: dotted.to_string(),
                expected: "true or false",
            }),
        Kind::Labels(labels) => {
            if let Some(text) = item.as_value().and_then(|v| v.as_str()) {
                labels
                    .iter()
                    .position(|label| *label == text)
                    .map(|index| u8::try_from(index).expect("labels fit u8"))
                    .ok_or_else(|| SettingsError::UnknownLabel {
                        key: dotted.to_string(),
                        label: text.to_string(),
                        allowed: range_text(kind),
                    })
            } else if stored_ranges {
                // A width-valid value with no label round-trips as the
                // integer the emitter wrote.
                let value = integer("a label string or an integer")?;
                wide(value)
            } else {
                Err(SettingsError::WrongType {
                    key: dotted.to_string(),
                    expected: "a string",
                })
            }
        }
    }
}

/// The spec's table path under the appearance root `root` (`Appearance` →
/// `root`, `Physique` → `root.physique`; a top-level key's table is the root
/// itself, though its callers filter `Top` first).
fn absolute(root: &str, table: KeyTable) -> String {
    match table.sub_table() {
        None => root.to_string(),
        Some(sub) => format!("{root}.{sub}"),
    }
}

/// Everything the value walk did not consume is refused. A leaf is known
/// iff its dotted path is `name` or some key's; a table is known iff its
/// path is `appearance` or some key's table. The first unknown item is
/// `UnknownKey`; a table position holding a non-table is `WrongType`.
fn reject_unknown(document: &DocumentMut) -> Result<(), SettingsError> {
    for (name, item) in document.iter() {
        match name {
            "name" => {}
            "appearance" => reject_unknown_table(item, "appearance", "appearance")?,
            _ if SettingKey::ALL
                .iter()
                .any(|key| key.spec().table == KeyTable::Top && key.spec().name == name) => {}
            _ => {
                return Err(SettingsError::UnknownKey {
                    key: name.to_string(),
                });
            }
        }
    }
    Ok(())
}

/// The recursive half of [`reject_unknown`] and [`parse_appearance`]: `root`
/// is the appearance table's own dotted path, `path` the table being walked.
fn reject_unknown_table(item: &Item, root: &str, path: &str) -> Result<(), SettingsError> {
    let Some(table) = item.as_table_like() else {
        return Err(SettingsError::WrongType {
            key: path.to_string(),
            expected: "a table",
        });
    };
    for (leaf, sub) in table.iter() {
        let child = format!("{path}.{leaf}");
        if SettingKey::ALL.iter().any(|key| {
            key.spec().table != KeyTable::Top && absolute(root, key.spec().table) == child
        }) {
            if !sub.is_table_like() {
                return Err(SettingsError::WrongType {
                    key: child,
                    expected: "a table",
                });
            }
            reject_unknown_table(sub, root, &child)?;
        } else if !SettingKey::ALL.iter().any(|key| {
            key.spec().table != KeyTable::Top
                && absolute(root, key.spec().table) == path
                && key.spec().name == leaf
        }) {
            return Err(SettingsError::UnknownKey { key: child });
        }
    }
    Ok(())
}

/// A top-level model-ID key's parsed value: `None` when absent or `""`
/// (both mean default), `Some` for an integer inside the key's range. Any
/// other string is `WrongType`; a non-integer defers to the `Number` kind's
/// errors.
fn model_id(document: &DocumentMut, key: SettingKey) -> Result<Option<u8>, SettingsError> {
    let name = key.spec().name;
    let Some(item) = document.get(name) else {
        return Ok(None);
    };
    if let Some(text) = item.as_value().and_then(|v| v.as_str()) {
        return if text.is_empty() {
            Ok(None)
        } else {
            Err(SettingsError::WrongType {
                key: name.to_string(),
                expected: "an integer or \"\"",
            })
        };
    }
    value(name, key, item, false).map(Some)
}

/// The `[appearance]` table at `item` (dotted path `path`: "appearance" in a
/// settings.toml, "players.03.appearance" in a team.toml) parsed into stored
/// values. Shared with `team_toml` so the nested key handling has one home.
pub(crate) fn parse_appearance(
    item: &Item,
    path: &str,
    stored_ranges: bool,
) -> Result<AppearanceSettings, SettingsError> {
    let Some(appearance) = item.as_table_like() else {
        return Err(SettingsError::WrongType {
            key: path.to_string(),
            expected: "a table",
        });
    };
    let mut out = AppearanceSettings::default();
    for key in SettingKey::ALL {
        let spec = key.spec();
        if spec.table == KeyTable::Top {
            continue; // top-level keys are not in the appearance tables
        }
        let Some(item) = lookup(appearance, &spec, path)? else {
            continue;
        };
        let leaf = leaf_path(path, &spec);
        let stored = value(&leaf, key, item, stored_ranges)?;
        set_appearance(&mut out, key, Some(stored));
    }
    reject_unknown_table(item, path, path)?;
    Ok(out)
}

/// The `[prefix]` table plus its four subtables appended to `out`, in key
/// order: a `Some` emits its line with the key's comment, a `None` the
/// commented neutral line. Shared with `team_toml`'s `[players.NN.appearance]`.
pub(crate) fn emit_appearance(
    out: &mut String,
    prefix: &str,
    appearance: &AppearanceSettings,
    stored_ranges: bool,
) -> Result<(), SettingsError> {
    let mut table = KeyTable::Top;
    for key in SettingKey::ALL {
        let spec = key.spec();
        if spec.table == KeyTable::Top {
            continue; // top-level keys are not emitted inside a table
        }
        if spec.table != table {
            out.push_str(&format!("\n[{}]\n", absolute(prefix, spec.table)));
            table = spec.table;
        }
        let stored = get_appearance(appearance, key);
        let body = match stored {
            Some(stored) => format!("{} = {}", spec.name, toml_form(key, stored, stored_ranges)?),
            None => format!("{} = {}", spec.name, neutral_text(spec.kind)),
        };
        let line = padded(&body, spec.comment);
        match stored {
            Some(_) => out.push_str(&line),
            None => out.push_str(&format!("# {line}")),
        }
        out.push('\n');
    }
    Ok(())
}

impl PlayerSettings {
    /// Reads a `settings.toml` text. Every key is optional; anything the key
    /// table does not know (a stray table, a compiler-owned field such as
    /// `boots_id`) is `UnknownKey`. The first error met, in `SettingKey::ALL`
    /// order then the unknown-key sweep, is the one reported.
    pub fn parse(text: &str) -> Result<Self, SettingsError> {
        let document: DocumentMut = text
            .parse()
            .map_err(|e: toml_edit::TomlError| SettingsError::Toml(e.to_string()))?;
        let mut settings = PlayerSettings::default();
        if let Some(item) = document.get("name") {
            settings.name = Some(match item.as_value() {
                Some(Value::Boolean(b)) if *b.value() => NameSetting::FromFolder,
                Some(Value::String(s)) if s.value().contains('\0') => {
                    return Err(SettingsError::WrongType {
                        key: "name".to_string(),
                        expected: "a string without NUL",
                    });
                }
                Some(Value::String(s)) => NameSetting::Explicit(s.value().clone()),
                _ => {
                    return Err(SettingsError::WrongType {
                        key: "name".to_string(),
                        expected: "true or a string",
                    });
                }
            });
        }
        settings.boots_id = model_id(&document, SettingKey::BootsId)?;
        settings.gloves_id = model_id(&document, SettingKey::GlovesId)?;
        if let Some(item) = document.get("appearance") {
            settings.appearance = parse_appearance(item, "appearance", false)?;
        }
        reject_unknown(&document)?;
        Ok(settings)
    }

    /// The template `settings.toml`: the plan block verbatim for a settings
    /// where every key is `Some`; an unset key is its line commented out with
    /// the kind's neutral example value. `OutOfRange` for a value written
    /// directly into a public field that the key's kind cannot represent.
    pub fn to_toml(&self) -> Result<String, SettingsError> {
        let mut out = HEADER.to_string();
        match &self.name {
            Some(NameSetting::FromFolder) => out.push_str("name = true\n"),
            Some(NameSetting::Explicit(name)) => {
                out.push_str(&format!("name = {}\n", Value::from(name.as_str())));
            }
            None => out.push_str("# name = true\n"),
        }
        // The stock-model IDs are uncommented in every case: `""` is what a
        // template shows for default (TOML has no bare `key =`).
        for (key, id) in [
            (SettingKey::BootsId, self.boots_id),
            (SettingKey::GlovesId, self.gloves_id),
        ] {
            let spec = key.spec();
            let body = match id {
                Some(id) => format!("{} = {}", spec.name, toml_form(key, id, false)?),
                None => format!("{} = \"\"", spec.name),
            };
            out.push_str(&padded(&body, spec.comment));
            out.push('\n');
        }
        emit_appearance(&mut out, "appearance", &self.appearance, false)?;
        Ok(out)
    }

    /// Rewrites the `Some` values of an existing document, preserving its
    /// comments and formatting; `None` keys are left as the user wrote them
    /// (a commented key stays commented). Missing tables are created as
    /// standard tables; no comments are added. A table position
    /// (`appearance` or one of its four subtables) holding a non-table item
    /// is `WrongType`.
    pub fn update_toml(&self, document: &mut DocumentMut) -> Result<(), SettingsError> {
        fn set(item: &mut Item, value: Value) {
            let decor = item.as_value().map(|old| old.decor().clone());
            *item = Item::Value(value);
            if let (Some(decor), Item::Value(new)) = (decor, item) {
                *new.decor_mut() = decor;
            }
        }

        match &self.name {
            Some(NameSetting::FromFolder) => set(&mut document["name"], true.into()),
            Some(NameSetting::Explicit(name)) => {
                set(&mut document["name"], name.as_str().into());
            }
            None => {}
        }
        for (key, id) in [
            (SettingKey::BootsId, self.boots_id),
            (SettingKey::GlovesId, self.gloves_id),
        ] {
            if let Some(id) = id {
                set(&mut document[key.spec().name], toml_form(key, id, false)?);
            }
        }
        for key in SettingKey::ALL {
            let spec = key.spec();
            if spec.table == KeyTable::Top {
                continue; // top-level keys were written above
            }
            let Some(stored) = self.get(key) else {
                continue;
            };
            let mut item = document.as_item_mut();
            let mut path = String::new();
            for segment in spec.table.path().split('.') {
                if !path.is_empty() {
                    path.push('.');
                }
                path.push_str(segment);
                let child = match item {
                    Item::Table(table) => {
                        if !table.contains_key(segment) {
                            table.insert(segment, Item::Table(toml_edit::Table::new()));
                        }
                        table.get_mut(segment).expect("just inserted")
                    }
                    Item::Value(Value::InlineTable(_)) => {
                        let child = &mut item[segment];
                        if matches!(child, Item::None) {
                            *child = Item::Value(Value::InlineTable(toml_edit::InlineTable::new()));
                        }
                        child
                    }
                    _ => {
                        return Err(SettingsError::WrongType {
                            key: path,
                            expected: "a table",
                        });
                    }
                };
                match child {
                    Item::Table(_) | Item::Value(Value::InlineTable(_)) => item = child,
                    _ => {
                        return Err(SettingsError::WrongType {
                            key: path,
                            expected: "a table",
                        });
                    }
                }
            }
            set(&mut item[spec.name], toml_form(key, stored, false)?);
        }
        Ok(())
    }
}
