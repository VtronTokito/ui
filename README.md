# tokito_ui

A small, opinionated [`egui`](https://github.com/emilk/egui) **0.29** component
library — the shared design layer for the [Tokito](https://github.com/VtronTokito/tokito)
desktop schematic studio.

It is not a framework. It is a flat palette, a theme applier, icon helpers,
and a handful of composable primitives.

## Modules

| Module | What it provides |
|---|---|
| `tokens` | [`Tokens`] — a flat colour + metrics palette (`Tokens::dark()` / `light()`). |
| `theme` | `apply(ctx, &tokens, dark)` — visuals, a named type scale, spacing. `add_phosphor(&mut fonts)` — the icon font. |
| `icons` | Phosphor glyphs rendered through a dedicated font family (no PUA collisions). `icons::ph::*` constants. |
| `components` | `card`, `new_tile`, `icon_button`, `text_button`, `link`, `list_row`, `search_field`, `page_header`, `section_header`. |

## Design rules

- **No global theme.** Every component takes `&Tokens`. A consumer can run two
  themes at once.
- **Primitives, not finished widgets.** `card` is an animated container; a
  "project card" is something the *consumer* composes from it.
- **Hover animates.** Interactive components ease a `0.0..=1.0` factor via
  `egui::Context::animate_bool` and lerp colour / offset — smooth in an
  immediate-mode redraw, no retained scene graph.
- **Icons can't break text.** Phosphor renders through a dedicated font family
  so its Private-Use-Area codepoints never collide with a text font.

## Usage

```rust
// once, at startup
let mut fonts = egui::FontDefinitions::default();
// ... register your text fonts ...
tokito_ui::theme::add_phosphor(&mut fonts);
ctx.set_fonts(fonts);

// every frame the theme could change
let tokens = tokito_ui::Tokens::dark();
tokito_ui::theme::apply(ctx, &tokens, true);

// in UI code
use tokito_ui::components as c;
c::page_header(ui, &tokens, "Projects", "Open a recent project.");
let resp = c::card(ui, &tokens, egui::vec2(300.0, 134.0), |ui| {
    ui.label("Arduino Shield v2");
});
if resp.clicked() { /* open it */ }
```

## Version coupling

`tokito_ui` pins `egui` **0.29** and `egui-phosphor` **0.7.x** (≥ 0.8 targets
egui ≥ 0.30). Bump all three together.

## License

MIT.
