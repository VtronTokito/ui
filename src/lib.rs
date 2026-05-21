//! # tokito_ui
//!
//! A small, opinionated component library for [`egui`] 0.29 — the shared
//! design layer for the Tokito desktop app.
//!
//! It is deliberately not a full framework. It provides:
//!
//! - [`Tokens`] — a flat colour + metrics palette (light / dark).
//! - [`theme`] — apply the tokens to an egui [`Context`](egui::Context):
//!   visuals, a named type scale, spacing, and the Phosphor icon font.
//! - [`icons`] — render Phosphor icons without the Private-Use-Area glyph
//!   collisions that plague a naive icon-font setup.
//! - [`components`] — the building blocks: an animated [`card`](components::card),
//!   [`icon_button`](components::icon_button), [`new_tile`](components::new_tile),
//!   [`list_row`](components::list_row), headers, a search field and buttons.
//!
//! Everything is built for an immediate-mode toolkit: solid fills, 1 px
//! borders, rounded rects, painted glyphs. Hover states animate through
//! [`egui::Context::animate_bool`], so they are smooth without a retained
//! scene graph.

pub mod components;
pub mod icons;
pub mod theme;
pub mod tokens;

pub use egui_phosphor;
pub use tokens::Tokens;

/// Linear-interpolate between two colours (channel-wise, non-premultiplied).
pub fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    egui::Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}
