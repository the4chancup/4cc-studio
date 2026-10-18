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
Its first host is the Save editor's color fields (team colors, skin, iris,
wrist tape; `plans/save_editor.md` "Crate layout"), which is what puts it in
Phase 8; then the Kit config editor (config colors and kit `colors.txt` colors)
and the Team creator; any later tool that needs a color field uses it too.

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
