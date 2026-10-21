# 4cc Studio — Library crates plan: color_tools

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## `libs/color_tools`

### Color-picker widget (Phase 8)

egui ships a bare HSVA picker (`egui::color_picker`); this widget wraps it into the
suite's standard picker popup: hex entry (`#RRGGBB`, matching the export text
formats), RGB fields, suggestion swatches (e.g. the extraction results for the kit
being edited), and recent colors. The widget also reports a **hover candidate**:
while a swatch is hovered or the picker is dragged, the host gets the candidate
color to live-preview in its own UI (the Kit config editor tints its menu-UI
mockup and icon gallery with it), reverting when the hover ends without a click.
Its first host is the Save editor's team color fields (`plans/save_editor.md`
"Crate layout"; its other colors, skin or spectacles, are palette indices,
chosen from their palettes, not picked), which is what puts it in
Phase 8; then the Kit config editor (config colors and kit `colors.txt` colors)
and the Team creator; any later tool that needs a color field uses it too.

The field is a color button showing the color; a click opens the popup, which holds, top to
bottom: egui's saturation-value square and hue slider, opaque (no export or save stores an
alpha); the hex field, committed on Enter or when it loses focus, a text that does not read
as `#RRGGBB` (the `#` optional, either case) put back to the current color when it loses
focus; the R, G and B fields, 0-255; the host's suggestion swatches; the recent colors.
Suggestions come from the host and may be none: the Save editor passes none, its team colors
having no kit texture to extract from. Recent colors are the session's, shared by every
picker in the program, newest first, at most eight: a color joins when the popup closes on a
color other than the one it opened on, so a drag across the square adds one entry, not a
hundred. They are not saved across restarts, as the help window's geometry is not: saving
would rewrite the settings file for each pick, for a convenience. The contract is the one
`libs/team_widgets` states (`plans/save_editor.md` "`libs/team_widgets`"): the widget takes the
color as `&mut [u8; 3]`, draws, and returns whether it changed and the hover candidate; the
host converts to what it stores (the Save editor's 6 bits per channel). A drag in the square,
the hue slider or an R/G/B field is a candidate while it lasts and one change when released,
as a click on a swatch or a committed hex text is one change: the host's undo then takes one
step per gesture, not one per frame of a drag.

### Kit menu icon rendering (Phase 8)

The 24 kit menu icons (`icon_<N>`, 0–23) are simple two-color kit icon patterns —
plain, striped, hooped, sashed, halved, contrast-sleeves, and so on. In game they
only ever appear in the PES 15/16 prematch gameplan screens, but suite-side they
make a great compact kit-color visual, so the crate recreates them as **vector
draw routines** (egui painter) parameterized by the icon number, a size, and the
kit's two menu colors — no game bitmaps shipped, tintable with any colors, crisp
at any size. Two render modes: the **full shirt shape** (the editor's icon
gallery) and **pattern-only** — just the two-color fill pattern as a flat square
swatch, for sizes where a shirt silhouette would turn to mush (the Team
compiler's grid cells). The pattern shapes are transcribed from the PES 15
settings reference sheet (`Kiticons.png`, kept with the crate's test data).

Consumers: `team_compiler` (extraction fallback; grid kit-cell miniatures),
`kit_config_editor` (picker + swatches; icon gallery), the Team creator.

---
