//! Apply [`Tokens`] to an egui [`Context`](egui::Context).
//!
//! [`apply`] sets the visuals, a named type scale and spacing. [`add_phosphor`]
//! wires the Phosphor icon font — call it from the consumer's font setup,
//! *after* the consumer has registered its own text fonts.

use crate::tokens::Tokens;
use egui::{Context, FontDefinitions, FontFamily, FontId, Stroke, TextStyle, Visuals};
use std::sync::Arc;

/// The dedicated egui font family that [`crate::icons`] renders glyphs through.
///
/// Phosphor is registered both as a *fallback* on the proportional family
/// (so a stray icon in normal text still resolves) and as this *dedicated*
/// family. Rendering icons through the dedicated family guarantees Phosphor's
/// Private-Use-Area codepoints are never intercepted by a text font that
/// happens to occupy the same PUA range (e.g. Inter Variable).
pub const ICON_FAMILY: &str = "phosphor";

/// Register the Phosphor icon font on `fonts`.
///
/// Call this from the consumer's font setup after inserting text fonts:
///
/// ```ignore
/// let mut fonts = egui::FontDefinitions::default();
/// // ... insert Inter / your text fonts ...
/// tokito_ui::theme::add_phosphor(&mut fonts);
/// ctx.set_fonts(fonts);
/// ```
pub fn add_phosphor(fonts: &mut FontDefinitions) {
    egui_phosphor::add_to_fonts(fonts, egui_phosphor::Variant::Regular);
    fonts.families.insert(
        FontFamily::Name(ICON_FAMILY.into()),
        vec!["phosphor".to_owned()],
    );
}

/// Apply tokens to the context: visuals, the named type scale and spacing.
///
/// The theme (light / dark) comes from [`Tokens::dark`] — one source of truth.
/// Fonts are the consumer's responsibility (see [`add_phosphor`]); this only
/// touches [`egui::Style`] and [`egui::Visuals`].
pub fn apply(ctx: &Context, t: &Tokens) {
    let mut visuals = if t.dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    visuals.override_text_color = Some(t.text);
    visuals.window_fill = t.bg_chrome;
    visuals.panel_fill = t.bg;
    visuals.extreme_bg_color = t.card; // editable widget backgrounds
    visuals.faint_bg_color = t.card;
    visuals.hyperlink_color = t.accent;

    visuals.widgets.noninteractive.bg_fill = t.card;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.text_3);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.border_soft);
    visuals.widgets.noninteractive.rounding = t.rounding_sm();

    visuals.widgets.inactive.bg_fill = t.card;
    visuals.widgets.inactive.weak_bg_fill = t.bg_chrome;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text_2);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, t.border);
    visuals.widgets.inactive.rounding = t.rounding_sm();

    visuals.widgets.hovered.bg_fill = t.card_hover;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, t.text);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, t.border_strong);
    visuals.widgets.hovered.rounding = t.rounding_sm();

    visuals.widgets.active.bg_fill = t.card_hover;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, t.text);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, t.accent);
    visuals.widgets.active.rounding = t.rounding_sm();

    visuals.widgets.open.bg_fill = t.card;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, t.border_strong);
    visuals.widgets.open.rounding = t.rounding_sm();

    visuals.selection.bg_fill = t.accent_soft;
    visuals.selection.stroke = Stroke::new(1.0, t.accent);

    visuals.window_rounding = t.rounding_md();
    visuals.menu_rounding = t.rounding_sm();
    visuals.window_stroke = Stroke::new(1.0, t.border_strong);
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    visuals.popup_shadow = egui::epaint::Shadow::NONE;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    // Named type scale — consume via RichText::text_style(TextStyle::Name(..)).
    let proportional = |size: f32| FontId::new(size, FontFamily::Proportional);
    style
        .text_styles
        .insert(TextStyle::Heading, proportional(27.0));
    style
        .text_styles
        .insert(TextStyle::Name(Arc::from("h2")), proportional(16.0));
    style
        .text_styles
        .insert(TextStyle::Name(Arc::from("h3")), proportional(13.0));
    style.text_styles.insert(TextStyle::Body, proportional(14.0));
    style
        .text_styles
        .insert(TextStyle::Button, proportional(13.5));
    style.text_styles.insert(TextStyle::Small, proportional(12.0));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace));

    style.spacing.item_spacing = egui::vec2(t.space_3, t.space_3);
    style.spacing.button_padding = egui::vec2(t.space_3, t.space_2);
    style.spacing.window_margin = egui::Margin::same(t.space_2);
    style.spacing.menu_margin = egui::Margin::same(t.space_2);
    style.spacing.indent = 18.0;

    ctx.set_style(style);
}
