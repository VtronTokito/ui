//! Phosphor icon helpers.
//!
//! Icons render through the dedicated [`ICON_FAMILY`] font family
//! so their Private-Use-Area codepoints can never be intercepted
//! by a text font that occupies the same PUA range. Rendering an icon through
//! the normal proportional family is the classic egui-icon bug — a stray text
//! glyph paints instead of the icon.

use crate::theme::ICON_FAMILY;
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontFamily, FontId, RichText};

/// Re-exported Phosphor Regular glyph constants — `icons::ph::FOLDER`, etc.
pub use egui_phosphor::regular as ph;

fn family() -> FontFamily {
    FontFamily::Name(ICON_FAMILY.into())
}

/// A [`FontId`] in the icon family — for painting glyphs via [`egui::Painter::text`].
pub fn font(size: f32) -> FontId {
    FontId::new(size, family())
}

/// A standalone icon as [`RichText`] — for icon-only buttons and inline glyphs.
pub fn icon(glyph: &str, size: f32, color: Color32) -> RichText {
    RichText::new(glyph).font(font(size)).color(color)
}

/// An icon followed by a text label, as a [`LayoutJob`].
///
/// The icon section renders from the icon family; the text section from the
/// default proportional family. Pass the result to [`egui::Button::new`],
/// [`egui::Ui::label`], [`egui::Ui::button`], [`crate::components::list_row`],
/// etc.
pub fn icon_text(
    glyph: &str,
    icon_size: f32,
    text: &str,
    text_size: f32,
    color: Color32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.append(
        glyph,
        0.0,
        TextFormat {
            font_id: font(icon_size),
            color,
            ..Default::default()
        },
    );
    job.append(
        text,
        8.0,
        TextFormat {
            font_id: FontId::proportional(text_size),
            color,
            ..Default::default()
        },
    );
    job
}
