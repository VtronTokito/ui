//! The component building blocks.
//!
//! Each component is a free function taking `&mut Ui` and `&Tokens`. They are
//! intentionally low-level primitives — a [`card`] container, a [`list_row`],
//! an [`icon_button`] — that a consumer composes into domain widgets (a
//! project card, a design card, …) rather than a closed set of finished
//! widgets.
//!
//! Hover states animate through [`egui::Context::animate_bool_with_time`]:
//! every interactive component eases a `0.0..=1.0` factor and lerps colour
//! against it, so highlights are smooth in an immediate-mode redraw (egui
//! self-schedules the repaints while an animation is in flight).
//!
//! All text-style use ([`page_header`], [`section_header`]) assumes
//! [`crate::theme::apply`] has registered the named type scale — call it once
//! at startup.

use crate::tokens::Tokens;
use crate::{icons, lerp_color};
use egui::{
    pos2, vec2, Align, Color32, Layout, Pos2, Rect, Response, RichText, Sense, Stroke, TextStyle,
    Ui, UiBuilder, Vec2,
};
use std::hash::Hash;

/// How long a hover transition takes, in seconds.
const HOVER_TIME: f32 = 0.11;

/// Eased hover factor for `id`, `0.0` (rest) … `1.0` (hovered).
fn hover_t(ui: &Ui, id: egui::Id, hovered: bool) -> f32 {
    ui.ctx().animate_bool_with_time(id, hovered, HOVER_TIME)
}

// ---------------------------------------------------------------------------
// card
// ---------------------------------------------------------------------------

/// An animated, clickable card of fixed `size`.
///
/// On hover it eases fill `card` → `card_hover` and border `border` →
/// `border_strong`. `add_contents` runs inside the card with a uniform 16 px
/// inner margin, clipped to the card bounds.
///
/// Returns the card's [`Response`]. If `add_contents` adds its own interactive
/// widgets (a kebab menu, say), egui resolves the click to the top-most
/// widget — but the card response may still report `clicked()`; check the
/// inner widget's response first when both can fire.
pub fn card(ui: &mut Ui, t: &Tokens, size: Vec2, add_contents: impl FnOnce(&mut Ui)) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hv = hover_t(ui, response.id, response.hovered());

    let fill = lerp_color(t.card, t.card_hover, hv);
    let border = lerp_color(t.border, t.border_strong, hv);
    let painter = ui.painter();
    painter.rect_filled(rect, t.rounding_md(), fill);
    painter.rect_stroke(rect.shrink(0.5), t.rounding_md(), Stroke::new(1.0, border));

    let mut content = ui.new_child(
        UiBuilder::new()
            .max_rect(rect.shrink(t.space_4))
            .layout(Layout::top_down(Align::Min)),
    );
    content.set_clip_rect(rect);
    add_contents(&mut content);
    response
}

/// A dashed "create new …" tile of fixed `size`.
///
/// Centred: a circular `+` mark over `label` (and optional `sublabel`). On
/// hover the dashed border and wash ease toward the accent.
pub fn new_tile(
    ui: &mut Ui,
    t: &Tokens,
    label: &str,
    sublabel: Option<&str>,
    size: Vec2,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hv = hover_t(ui, response.id, response.hovered());

    if hv > 0.001 {
        ui.painter()
            .rect_filled(rect, t.rounding_md(), t.accent_soft.gamma_multiply(hv));
    }
    let border = lerp_color(t.border_strong, t.accent, hv);
    paint_dashed_rect(ui.painter(), rect.shrink(1.0), border, 1.5, 6.0, 4.0);

    let center = rect.center();
    let circle_c = pos2(center.x, center.y - 14.0);
    ui.painter().circle_filled(circle_c, 20.0, t.accent_soft);
    ui.painter().text(
        circle_c,
        egui::Align2::CENTER_CENTER,
        icons::ph::PLUS,
        icons::font(20.0),
        t.accent,
    );
    ui.painter().text(
        pos2(center.x, center.y + 16.0),
        egui::Align2::CENTER_CENTER,
        label,
        TextStyle::Body.resolve(ui.style()),
        t.text,
    );
    if let Some(sub) = sublabel {
        ui.painter().text(
            pos2(center.x, center.y + 33.0),
            egui::Align2::CENTER_CENTER,
            sub,
            TextStyle::Small.resolve(ui.style()),
            t.text_3,
        );
    }
    response
}

// ---------------------------------------------------------------------------
// buttons
// ---------------------------------------------------------------------------

/// A square, frameless icon button. `glyph` is a [`icons::ph`] constant.
///
/// `side` is the button's width and height. Hover eases a soft background in.
pub fn icon_button(ui: &mut Ui, t: &Tokens, glyph: &str, side: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
    let hv = hover_t(ui, response.id, response.hovered());
    if hv > 0.001 {
        ui.painter()
            .rect_filled(rect, t.rounding_sm(), t.card.gamma_multiply(hv));
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            t.rounding_sm(),
            Stroke::new(1.0, t.border.gamma_multiply(hv)),
        );
    }
    let ink = lerp_color(t.text_2, t.text, hv);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        icons::font(side * 0.5),
        ink,
    );
    response
}

/// Visual weight of a [`text_button`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    /// Accent fill — the one primary action on a surface.
    Primary,
    /// Card fill + border — a normal action.
    Secondary,
}

/// A text button. `height` fixes the row height; width fits the label.
pub fn text_button(
    ui: &mut Ui,
    t: &Tokens,
    kind: ButtonKind,
    label: &str,
    height: f32,
) -> Response {
    // PLACEHOLDER so the galley has no baked colour — `Painter::galley`'s
    // fallback colour then applies, letting us colour by hover state.
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        TextStyle::Button.resolve(ui.style()),
        Color32::PLACEHOLDER,
    );
    let width = galley.size().x + t.space_4 * 2.0;
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let hv = hover_t(ui, response.id, response.hovered());

    let (fill, ink, border) = match kind {
        ButtonKind::Primary => (
            lerp_color(t.accent, lighten(t.accent, 0.12), hv),
            t.accent_ink,
            None,
        ),
        ButtonKind::Secondary => (
            lerp_color(t.card, t.card_hover, hv),
            lerp_color(t.text_2, t.text, hv),
            Some(lerp_color(t.border, t.border_strong, hv)),
        ),
    };
    ui.painter().rect_filled(rect, t.rounding_sm(), fill);
    if let Some(b) = border {
        ui.painter()
            .rect_stroke(rect.shrink(0.5), t.rounding_sm(), Stroke::new(1.0, b));
    }
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, ink);
    response
}

/// An inline text link in the accent colour.
pub fn link(ui: &mut Ui, t: &Tokens, label: &str) -> Response {
    let resp = ui.add(
        egui::Label::new(RichText::new(label).color(t.accent).size(13.0))
            .sense(Sense::click()),
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

// ---------------------------------------------------------------------------
// rows & inputs
// ---------------------------------------------------------------------------

/// A full-width, **left-aligned**, hover-highlighted clickable list row.
///
/// `egui`'s `SelectableLabel` / `Button` centre their text and `add_sized`
/// centres the widget — both produce floating, centred labels that read as
/// broken in a menu or list. This paints the row manually: a background fill
/// on hover / selection and the `job` galley pinned to the left edge. Build
/// `job` with [`icons::icon_text`].
pub fn list_row(
    ui: &mut Ui,
    t: &Tokens,
    job: egui::text::LayoutJob,
    selected: bool,
) -> Response {
    let height = 32.0;
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click());
    let hv = hover_t(ui, response.id, response.hovered());
    let bg = if selected {
        t.accent_soft
    } else {
        t.card_hover.gamma_multiply(hv)
    };
    if bg.a() > 0 {
        ui.painter().rect_filled(rect, t.rounding_sm(), bg);
    }
    let galley = ui.fonts(|f| f.layout_job(job));
    let pos = pos2(rect.left() + 10.0, rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(pos, galley, t.text);
    response
}

/// A bordered search field: a magnifier glyph + a single-line text edit.
///
/// `id_source` must be **stable and unique** across frames — the inner
/// `TextEdit`'s identity (focus, cursor, undo history) is keyed off it. Pass
/// something like `"design_search"`, never a value derived from layout
/// position. `width` fixes the field width. Returns the `TextEdit` response.
pub fn search_field(
    ui: &mut Ui,
    t: &Tokens,
    id_source: impl Hash,
    query: &mut String,
    hint: &str,
    width: f32,
) -> Response {
    let id = egui::Id::new(id_source);
    let height = 32.0;
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let focused = ui.memory(|m| m.has_focus(id));
    let border = if focused { t.accent } else { t.border };
    ui.painter().rect_filled(rect, t.rounding_sm(), t.bg_chrome);
    ui.painter()
        .rect_stroke(rect.shrink(0.5), t.rounding_sm(), Stroke::new(1.0, border));
    ui.painter().text(
        pos2(rect.left() + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        icons::ph::MAGNIFYING_GLASS,
        icons::font(14.0),
        t.text_3,
    );
    let edit_rect = Rect::from_min_max(
        pos2(rect.left() + 32.0, rect.top()),
        pos2(rect.right() - 8.0, rect.bottom()),
    );
    let mut edit_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(edit_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    edit_ui.add(
        egui::TextEdit::singleline(query)
            .id(id)
            .hint_text(hint)
            .frame(false)
            .desired_width(edit_rect.width()),
    )
}

// ---------------------------------------------------------------------------
// headers
// ---------------------------------------------------------------------------

/// A page header: a large title over a muted subtitle.
pub fn page_header(ui: &mut Ui, t: &Tokens, title: &str, subtitle: &str) {
    ui.label(
        RichText::new(title)
            .text_style(TextStyle::Heading)
            .strong()
            .color(t.text),
    );
    ui.add_space(6.0);
    ui.label(RichText::new(subtitle).size(14.0).color(t.text_2));
}

/// A section header: an `h2` title with an optional right-aligned action link.
///
/// Returns `Some(response)` for the action link when `action` is given, so the
/// caller can test `.clicked()` (and anything else a [`Response`] carries).
pub fn section_header(
    ui: &mut Ui,
    t: &Tokens,
    title: &str,
    action: Option<&str>,
) -> Option<Response> {
    let mut action_resp = None;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(title)
                .text_style(TextStyle::Name("h2".into()))
                .strong()
                .color(t.text),
        );
        if let Some(action) = action {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                action_resp = Some(link(ui, t, action));
            });
        }
    });
    action_resp
}

// ---------------------------------------------------------------------------
// painting helpers
// ---------------------------------------------------------------------------

/// Paint a dashed rectangle outline (sharp corners).
fn paint_dashed_rect(
    painter: &egui::Painter,
    rect: Rect,
    color: Color32,
    thickness: f32,
    dash: f32,
    gap: f32,
) {
    let stroke = Stroke::new(thickness.max(0.1), color);
    let dash = dash.max(0.5);
    // Guard the loop step — a non-positive (dash + gap) would never advance.
    let step = (dash + gap).max(0.5);
    let edge = |a: Pos2, b: Pos2| {
        let total = (b - a).length();
        if total <= 0.0 {
            return;
        }
        let dir = (b - a) / total;
        let mut d = 0.0;
        while d < total {
            let s = a + dir * d;
            let e = a + dir * (d + dash).min(total);
            painter.line_segment([s, e], stroke);
            d += step;
        }
    };
    edge(rect.left_top(), rect.right_top());
    edge(rect.right_top(), rect.right_bottom());
    edge(rect.right_bottom(), rect.left_bottom());
    edge(rect.left_bottom(), rect.left_top());
}

/// Lighten a colour toward white by `amount` (`0.0..=1.0`).
fn lighten(c: Color32, amount: f32) -> Color32 {
    lerp_color(c, Color32::WHITE, amount)
}
