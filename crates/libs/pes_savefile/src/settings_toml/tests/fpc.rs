//! `PlayerSettings::apply_fpc_preset` and `strip_style`: the marker presets
//! on a `settings.toml`'s model (`fpc_strip_conflict`'s keys, the values the
//! row writes) and the `::fpc::check` inputs they give.

use super::*;

/// The hide preset over an authored `sleeves` of short: the preset's long
/// wins and the key is reported.
#[test]
fn the_hide_preset_overrides_an_authored_sleeves_and_reports_it() {
    let mut settings =
        PlayerSettings::parse("[appearance.strip]\nsleeves = \"short\"\n").expect("parses");
    assert_eq!(settings.get(SettingKey::Sleeves), Some(1), "authored short");
    let overridden = settings.apply_fpc_preset(&::fpc::preset(::fpc::Preset::Hide));
    assert_eq!(overridden, ["appearance.strip.sleeves"]);
    assert_eq!(settings.get(SettingKey::Sleeves), Some(2), "preset long");
    assert_eq!(settings.get(SettingKey::Socks), Some(2), "preset short");
    assert_eq!(settings.get(SettingKey::Untucked), Some(0), "preset tucked");
    // The boots and gloves keys are the resolution's, not the preset's.
    assert_eq!(settings.boots_id, None);
    assert_eq!(settings.gloves_id, None);
}

/// An authored `socks` equal to the preset's is kept and not reported.
#[test]
fn an_authored_value_equal_to_the_preset_s_is_kept_and_unreported() {
    let mut settings =
        PlayerSettings::parse("[appearance.strip]\nsocks = \"short\"\n").expect("parses");
    let overridden = settings.apply_fpc_preset(&::fpc::preset(::fpc::Preset::Hide));
    assert_eq!(overridden, Vec::<String>::new());
    assert_eq!(settings.get(SettingKey::Socks), Some(2));
}

/// A custom skin under the hide preset resets to 1 and is reported; any
/// other authored skin is left alone.
#[test]
fn a_custom_skin_resets_and_reports_where_another_skin_is_left_alone() {
    let mut settings = PlayerSettings::parse("[appearance]\nskin_color = 7\n").expect("parses");
    let overridden = settings.apply_fpc_preset(&::fpc::preset(::fpc::Preset::Hide));
    assert_eq!(overridden, ["appearance.skin_color"]);
    assert_eq!(settings.get(SettingKey::SkinColor), Some(1));

    let mut settings = PlayerSettings::parse("[appearance]\nskin_color = 3\n").expect("parses");
    assert_eq!(
        settings.apply_fpc_preset(&::fpc::preset(::fpc::Preset::Hide)),
        Vec::<String>::new(),
        "a preset skin is no conflict"
    );
    assert_eq!(settings.get(SettingKey::SkinColor), Some(3));
}

/// The strip style of a file setting inners and wrist taping, the absent
/// keys at their defaults and the given gloves ID carried.
#[test]
fn the_strip_style_reads_the_file_s_keys_and_their_defaults() {
    let settings = PlayerSettings::parse(
        "[appearance.strip]\ninners = \"turtleneck\"\nwrist_taping = \"left\"\n",
    )
    .expect("parses");
    let style = settings.strip_style(11);
    assert!(style.inners_worn);
    assert!(style.wrist_taping);
    assert!(!style.undershorts_worn, "absent is off");
    assert!(!style.ankle_taping);
    assert!(!style.gloves_checkbox);
    assert_eq!(style.gloves_id, 11);
    assert_eq!(style.skin_color, ::fpc::SkinColor::Preset);

    let settings = PlayerSettings::parse("[appearance]\nskin_color = 7\n").expect("parses");
    assert_eq!(settings.strip_style(0).skin_color, ::fpc::SkinColor::Custom);
}
