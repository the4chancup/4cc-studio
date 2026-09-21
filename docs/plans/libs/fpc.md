# 4cc Studio — Library crates plan: fpc

Part of the [Library crates plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## `libs/fpc`

Full Player Customization is one *system* that spans two formats: kit configs (every kit of the
team, GK included, must carry shirt model 176, shorts model 16, collar 105, winter collar 105) and
the savefile's player appearance (the hide preset: long sleeves, tucked shirt, short socks, a
nonexistent boots ID such as 55, a nonexistent gloves ID such as 11; the un-hide preset: short
sleeves, untucked, standard/long socks, boots 0, gloves 0 or 1–10 for keepers; the partial-hide
preset for a custom body inside a stock jersey: short sleeves, untucked, standard socks, skin color
Custom). Neither `kit_config` nor `pes_savefile` is the natural owner, and having each hold half
would either duplicate the knowledge or make one format crate depend on the other. So the system's
knowledge is a **leaf crate with no dependencies**, holding data and pure rules only:

- `kit.rs` — the kit-config FPC values per PES version (the modern system: PES 19+ and the 2024
  reimplementation for 16/17; the retro 16/17 system is documented as legacy and not supported).
  `kit_values(version)` returns the same four values (shirt model 176, shorts model 16, collar
  105, winter collar 105) for PES 16, 17, 19, 20 and 21 and `None` for PES 15 and 18, where no FPC
  system exists. The wiki page documents the PES 17 values and calls the PES 19+ system "mostly
  identical"; that the four values are the same on PES 19+ is still to be confirmed against a
  real PES 21 FPC team's kit config (none was identified on the writing machine at Phase 2
  converge; the stock configs `kit_config` was measured on are not FPC kits). The check belongs
  to the first compile of a PES 21 FPC export (Phase 4's kit step): `matches_fpc` on its configs.
- `player.rs` — the three appearance presets above, expressed in the crate's own small vocabulary
  (`Sleeves`, `Tuck`, `Socks`, boots/gloves IDs, skin color), not in `pes_savefile` field terms;
  and `custom_skin_available(version) -> bool`, true for PES 15 to 17 only (the Fox games dropped
  the Custom skin; the reference editor resets it to the default there), so the partial-hide
  preset's one version-dependent fact lives with the rest of the FPC knowledge.
- `interference.rs` — the settings that break or bend FPC, as findings with severity: inners ≠ None
  and undershorts ≠ Off/Off break hiding (error on an FPC player); wrist/ankle taping ≠ None
  selectively shows pieces (info — deliberate in custom setups); the Gloves checkbox with gloves ID
  0 causes winter gloves in winter conditions (warning); skin color Custom on a non-FPC player hides
  the body but not the jersey (warning).

Consumers: `kit_config` (`apply_fpc` / `matches_fpc` use `fpc::kit`; there is no revert, since the
template config already carries the FPC values and the compiler never auto-reverts them), `pes_savefile`
(`ops/fpc.rs` maps `fpc::player` presets onto `PlayerEntry` and runs the interference check), and
through them the Team compiler (`fpc.on`/`fpc.off` markers, kit reconciliation, `settings.toml`
validation), the Save editor (FPC toggle, Appearance-tab warnings), the Kit config editor (FPC
indicator), and the Player aesthetics editor (marker toggles). The wiki's FPC guide
(`resources/FPC.wikitext` in this repo) is the source text; each constant carries a doc comment pointing at
its line there.

---
