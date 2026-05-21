# AGENTS.md — tokito_ui

**For AI coding assistants working in this repo.** Read this first.

## What this is

`tokito_ui` is a small **egui 0.29** component library — the shared design
layer for the Tokito desktop schematic studio
([github.com/VtronTokito/tokito](https://github.com/VtronTokito/tokito)).

It is **not a framework**. It is a flat token palette, a theme applier, icon
helpers, and a handful of composable primitives. Think "the design system,"
not "the app."

## Structure

```
src/
  lib.rs         crate root — re-exports, `lerp_color`
  tokens.rs      `Tokens` — the flat colour + metrics palette (dark / light)
  theme.rs       `apply()` — egui visuals + named type scale + spacing
                 `add_phosphor()` — register the icon font
  icons.rs       Phosphor glyphs via a dedicated, collision-free font family
  components.rs  the building blocks (see below)
```

**Components** (`components.rs`): `card`, `new_tile`, `icon_button`,
`text_button`, `link`, `list_row`, `text_input`, `search_field`, `toggle`,
`modal`, `page_header`, `section_header`.

## Rules — keep these true

- **Every component is a free function** `fn(ui: &mut Ui, t: &Tokens, …)`.
  There is no global theme singleton; a consumer can run two themes at once.
- **Primitives, not finished widgets.** `card` is an animated container; a
  "project card" is something the *consumer* composes from it. Don't ship
  domain widgets here.
- **Hover animates.** Interactive components ease a `0.0..=1.0` factor via
  `egui::Context::animate_bool_with_time` and lerp colour — smooth in an
  immediate-mode redraw, no retained scene graph.
- **Icons can't break text.** Phosphor renders through a dedicated
  `FontFamily::Name("phosphor")` so its Private-Use-Area codepoints never
  collide with a text font (Inter Var occupies part of the PUA range).
- **Stable widget ids.** Anything with internal egui state (`text_input`,
  `search_field`) takes an explicit `id_source: impl Hash`. Never derive a
  widget id from layout position — it collides and breaks focus on reflow.
- **`Tokens` is `#[non_exhaustive]`.** Construct via `Tokens::dark()` /
  `light()` and assign fields to customise; new fields won't be breaking.
- **Version coupling.** Pin `egui` 0.29 and `egui-phosphor` 0.7.x together
  (≥ 0.8 targets egui ≥ 0.30). Bumping is a coordinated change.

## Working here

- `cargo clippy` must pass clean — that is the quality bar.
- A new component goes in `components.rs`, keeps the free-function shape, and
  gets a doc comment. Update `README.md`'s component list.
- Tokito consumes this as a **path dependency** during co-development and a
  **git dependency** on its `master`.

## Known follow-ups

- No keyboard focus rings / `widget_info` accessibility yet — components are
  painted rectangles, not screen-reader-navigable. Highest-value next step.
- Missing components a real app will want: tooltip, tabs, toast, checkbox.
