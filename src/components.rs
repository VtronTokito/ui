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
        egui::Label::new(RichText::new(label).color(t.accent).size(13.0)).sense(Sense::click()),
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// A small bordered pill — a count or status chip, e.g. next to a heading.
pub fn badge(ui: &mut Ui, t: &Tokens, text: &str) -> Response {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        TextStyle::Small.resolve(ui.style()),
        t.text_3,
    );
    let pad = vec2(9.0, 4.0);
    let (rect, response) = ui.allocate_exact_size(galley.size() + pad * 2.0, Sense::hover());
    ui.painter().rect_filled(rect, t.rounding_sm(), t.card);
    ui.painter().rect_stroke(
        rect.shrink(0.5),
        t.rounding_sm(),
        Stroke::new(1.0, t.border),
    );
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, t.text_3);
    response
}

// ---------------------------------------------------------------------------
// menu (kebab / dropdown)
// ---------------------------------------------------------------------------

/// A kebab / dropdown menu: an [`icon_button`] trigger that opens a popup of
/// [`menu_item`]s below it.
///
/// `id_source` must be stable and unique (the popup's open state is keyed off
/// it — pass e.g. `("design_kebab", design_id)`). The popup closes when an
/// item is clicked or the user clicks away. Returns the trigger's [`Response`].
pub fn menu_button(
    ui: &mut Ui,
    t: &Tokens,
    id_source: impl Hash,
    glyph: &str,
    side: f32,
    add_items: impl FnOnce(&mut Ui),
) -> Response {
    let trigger = icon_button(ui, t, glyph, side);
    let popup_id = egui::Id::new(id_source);
    if trigger.clicked() {
        ui.memory_mut(|m| m.toggle_popup(popup_id));
    }
    egui::popup::popup_below_widget(
        ui,
        popup_id,
        &trigger,
        egui::PopupCloseBehavior::CloseOnClick,
        |ui| {
            ui.set_min_width(184.0);
            add_items(ui);
        },
    );
    trigger
}

/// One row of a [`menu_button`] popup: a leading icon + a label.
///
/// Returns `true` on the frame it is clicked (which also closes the menu).
pub fn menu_item(ui: &mut Ui, t: &Tokens, glyph: &str, label: &str) -> bool {
    let job = icons::icon_text(glyph, 14.0, label, 12.5, t.text);
    list_row(ui, t, job, false).clicked()
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
pub fn list_row(ui: &mut Ui, t: &Tokens, job: egui::text::LayoutJob, selected: bool) -> Response {
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

/// A bordered single-line text input.
///
/// `id_source` must be **stable and unique** across frames — the inner
/// `TextEdit`'s identity (focus, cursor, undo history) is keyed off it. Pass
/// something like `"project_name"`, never a value derived from layout
/// position. `width` fixes the field width. Returns the `TextEdit` response.
pub fn text_input(
    ui: &mut Ui,
    t: &Tokens,
    id_source: impl Hash,
    value: &mut String,
    hint: &str,
    width: f32,
) -> Response {
    bordered_input(
        ui,
        t,
        egui::Id::new(id_source),
        value,
        hint,
        width,
        None,
        false,
    )
}

/// A bordered single-line input that **masks** its content (API keys, secrets).
///
/// Identical to [`text_input`] but the characters render as dots. Same
/// stable-`id_source` rule.
pub fn secret_input(
    ui: &mut Ui,
    t: &Tokens,
    id_source: impl Hash,
    value: &mut String,
    hint: &str,
    width: f32,
) -> Response {
    bordered_input(
        ui,
        t,
        egui::Id::new(id_source),
        value,
        hint,
        width,
        None,
        true,
    )
}

/// A bordered search field: a magnifier glyph + a single-line text edit.
///
/// Same identity rules as [`text_input`].
pub fn search_field(
    ui: &mut Ui,
    t: &Tokens,
    id_source: impl Hash,
    query: &mut String,
    hint: &str,
    width: f32,
) -> Response {
    bordered_input(
        ui,
        t,
        egui::Id::new(id_source),
        query,
        hint,
        width,
        Some(icons::ph::MAGNIFYING_GLASS),
        false,
    )
}

/// Shared implementation behind [`text_input`], [`search_field`] and
/// [`secret_input`].
fn bordered_input(
    ui: &mut Ui,
    t: &Tokens,
    id: egui::Id,
    value: &mut String,
    hint: &str,
    width: f32,
    leading_glyph: Option<&str>,
    mask: bool,
) -> Response {
    let height = 34.0;
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let focused = ui.memory(|m| m.has_focus(id));
    let border = if focused { t.accent } else { t.border };
    ui.painter().rect_filled(rect, t.rounding_sm(), t.bg_chrome);
    ui.painter()
        .rect_stroke(rect.shrink(0.5), t.rounding_sm(), Stroke::new(1.0, border));
    let text_left = if let Some(glyph) = leading_glyph {
        ui.painter().text(
            pos2(rect.left() + 12.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            glyph,
            icons::font(14.0),
            t.text_3,
        );
        rect.left() + 32.0
    } else {
        rect.left() + 11.0
    };
    let edit_rect = Rect::from_min_max(
        pos2(text_left, rect.top()),
        pos2(rect.right() - 9.0, rect.bottom()),
    );
    let mut edit_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(edit_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    edit_ui.add(
        egui::TextEdit::singleline(value)
            .id(id)
            .hint_text(hint)
            .frame(false)
            .password(mask)
            .desired_width(edit_rect.width()),
    )
}

/// A switch / toggle with a trailing `label`.
///
/// Flips `*value` on click and animates the knob + track. Returns the row's
/// [`Response`].
pub fn toggle(ui: &mut Ui, t: &Tokens, value: &mut bool, label: &str) -> Response {
    let track = vec2(38.0, 22.0);
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        TextStyle::Body.resolve(ui.style()),
        Color32::PLACEHOLDER,
    );
    let total = vec2(track.x + 9.0 + galley.size().x, track.y);
    let (rect, mut response) = ui.allocate_exact_size(total, Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    let on = ui
        .ctx()
        .animate_bool_with_time(response.id, *value, HOVER_TIME);

    let track_rect = Rect::from_min_size(rect.min, track);
    ui.painter().rect_filled(
        track_rect,
        egui::Rounding::same(track.y / 2.0),
        lerp_color(t.border_strong, t.accent, on),
    );
    let knob_x = egui::lerp((track_rect.left() + 11.0)..=(track_rect.right() - 11.0), on);
    ui.painter().circle_filled(
        pos2(knob_x, track_rect.center().y),
        8.0,
        Color32::from_rgb(0xfa, 0xfb, 0xfc),
    );
    ui.painter().galley(
        pos2(
            track_rect.right() + 9.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        t.text,
    );
    response
}

// ---------------------------------------------------------------------------
// modal
// ---------------------------------------------------------------------------

/// A centred modal dialog over a dimmed backdrop.
///
/// Renders only while `*open` is `true`. Sets `*open = false` when the user
/// presses Escape, clicks the backdrop, or clicks the close button.
/// `add_contents` runs inside the dialog body (16 px inset, `width` wide).
///
/// Call this at the top level of a frame (like a context-menu / overlay), not
/// nested inside a panel.
pub fn modal(
    ctx: &egui::Context,
    t: &Tokens,
    open: &mut bool,
    title: &str,
    width: f32,
    add_contents: impl FnOnce(&mut Ui),
) {
    if !*open {
        return;
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        *open = false;
        return;
    }

    let screen = ctx.screen_rect();
    // Dimmed backdrop — a full-screen click target that closes the modal.
    let backdrop = egui::Area::new(egui::Id::new(("tokito_ui_modal_backdrop", title)))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let resp = ui.allocate_rect(screen, Sense::click());
            ui.painter()
                .rect_filled(screen, 0.0, Color32::from_black_alpha(150));
            resp
        });
    if backdrop.inner.clicked() {
        *open = false;
    }

    // The dialog itself, centred, above the backdrop.
    egui::Area::new(egui::Id::new(("tokito_ui_modal", title)))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, -20.0])
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(t.bg_chrome)
                .stroke(Stroke::new(1.0, t.border_strong))
                .rounding(t.rounding_md())
                .inner_margin(egui::Margin::same(t.space_4))
                .show(ui, |ui| {
                    ui.set_width(width);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(title)
                                .text_style(TextStyle::Name("h2".into()))
                                .strong()
                                .color(t.text),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_button(ui, t, icons::ph::X, 26.0).clicked() {
                                *open = false;
                            }
                        });
                    });
                    ui.add_space(t.space_3);
                    add_contents(ui);
                });
        });
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
// vertical navigation
// ---------------------------------------------------------------------------

/// A vertical-navigation row — a full-width clickable item with a solid
/// accent fill when `selected`.
///
/// For sidebars: a settings dialog's section list, a wizard's steps. Unlike
/// [`list_row`] (a menu / list row with a *soft* selection wash), `nav_item`
/// paints a solid `accent` pill for the active item. Returns its [`Response`].
pub fn nav_item(ui: &mut Ui, t: &Tokens, label: &str, selected: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::click());
    let hv = hover_t(ui, response.id, response.hovered());
    let bg = if selected {
        t.accent
    } else {
        t.card_hover.gamma_multiply(hv)
    };
    if bg.a() > 0 {
        ui.painter().rect_filled(rect, t.rounding_sm(), bg);
    }
    let ink = if selected {
        t.accent_ink
    } else {
        lerp_color(t.text_2, t.text, hv)
    };
    ui.painter().text(
        pos2(rect.left() + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        TextStyle::Body.resolve(ui.style()),
        ink,
    );
    if response.hovered() && !selected {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

// ---------------------------------------------------------------------------
// form controls
// ---------------------------------------------------------------------------

/// A square checkbox with a `label` and an optional `description` line beneath.
///
/// Clicking anywhere on the row flips `*value`; the box eases an animated tick
/// in. `description` is muted helper text under the label. Returns the row's
/// [`Response`] — test `.changed()` to react to a flip. Use this (not
/// [`toggle`]) when the control is one of several settings in a form; reach
/// for [`toggle`] for a single prominent on/off switch.
pub fn checkbox(
    ui: &mut Ui,
    t: &Tokens,
    value: &mut bool,
    label: &str,
    description: Option<&str>,
) -> Response {
    let box_side = 18.0_f32;
    let gap = 10.0;
    let label_galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        TextStyle::Body.resolve(ui.style()),
        t.text,
    );
    let desc_galley = description.map(|d| {
        ui.painter()
            .layout_no_wrap(d.to_owned(), TextStyle::Small.resolve(ui.style()), t.text_3)
    });
    let label_h = label_galley.size().y;
    let text_w = label_galley
        .size()
        .x
        .max(desc_galley.as_ref().map_or(0.0, |g| g.size().x));
    let text_h = label_h + desc_galley.as_ref().map_or(0.0, |g| 3.0 + g.size().y);
    let row_h = box_side.max(text_h);

    let (rect, mut response) =
        ui.allocate_exact_size(vec2(box_side + gap + text_w, row_h), Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let on = ui
        .ctx()
        .animate_bool_with_time(response.id, *value, HOVER_TIME);
    let hv = hover_t(ui, response.id.with("hover"), response.hovered());

    let box_rect = Rect::from_min_size(
        pos2(rect.left(), rect.center().y - box_side / 2.0),
        Vec2::splat(box_side),
    );
    // A checkbox box reads as a square — `radius_sm` on an 18 px box looks
    // like a circle, so use the tighter `radius_xs`.
    ui.painter()
        .rect_filled(box_rect, t.rounding_xs(), lerp_color(t.card, t.accent, on));
    let border = lerp_color(lerp_color(t.border, t.border_strong, hv), t.accent, on);
    ui.painter().rect_stroke(
        box_rect.shrink(0.5),
        t.rounding_xs(),
        Stroke::new(1.0, border),
    );
    if on > 0.01 {
        let c = box_rect.center();
        let stroke = Stroke::new(2.0, t.accent_ink.gamma_multiply(on));
        ui.painter().line_segment(
            [
                pos2(c.x - box_side * 0.24, c.y + box_side * 0.02),
                pos2(c.x - box_side * 0.04, c.y + box_side * 0.20),
            ],
            stroke,
        );
        ui.painter().line_segment(
            [
                pos2(c.x - box_side * 0.04, c.y + box_side * 0.20),
                pos2(c.x + box_side * 0.26, c.y - box_side * 0.18),
            ],
            stroke,
        );
    }

    let text_x = rect.left() + box_side + gap;
    let text_top = rect.top() + (row_h - text_h) / 2.0;
    ui.painter()
        .galley(pos2(text_x, text_top), label_galley, t.text);
    if let Some(g) = desc_galley {
        ui.painter()
            .galley(pos2(text_x, text_top + label_h + 3.0), g, t.text_3);
    }
    response
}

/// A horizontal segmented control — a row of mutually-exclusive options.
///
/// `*selected` is the index of the active segment; clicking a segment sets it.
/// Segments split `width` evenly. Returns the row [`Response`]; `.changed()`
/// fires on a new selection.
pub fn segmented(
    ui: &mut Ui,
    t: &Tokens,
    options: &[&str],
    selected: &mut usize,
    width: f32,
) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(vec2(width, 34.0), Sense::hover());
    ui.painter().rect_filled(rect, t.rounding_sm(), t.card);
    ui.painter().rect_stroke(
        rect.shrink(0.5),
        t.rounding_sm(),
        Stroke::new(1.0, t.border),
    );

    let n = options.len().max(1);
    let seg_w = rect.width() / n as f32;
    let font = TextStyle::Button.resolve(ui.style());
    for (i, label) in options.iter().enumerate() {
        let seg = Rect::from_min_size(
            pos2(rect.left() + seg_w * i as f32, rect.top()),
            vec2(seg_w, rect.height()),
        );
        let id = response.id.with(i);
        let seg_resp = ui.interact(seg, id, Sense::click());
        let active = i == *selected;
        if seg_resp.clicked() && !active {
            *selected = i;
            response.mark_changed();
        }
        let hv = hover_t(ui, id, seg_resp.hovered());
        if active {
            ui.painter()
                .rect_filled(seg.shrink(3.0), t.rounding_sm(), t.accent);
        } else if hv > 0.001 {
            ui.painter().rect_filled(
                seg.shrink(3.0),
                t.rounding_sm(),
                t.card_hover.gamma_multiply(hv),
            );
        }
        let ink = if active {
            t.accent_ink
        } else {
            lerp_color(t.text_2, t.text, hv)
        };
        ui.painter().text(
            seg.center(),
            egui::Align2::CENTER_CENTER,
            *label,
            font.clone(),
            ink,
        );
    }
    response
}

/// A dropdown select. The trigger box shows `current` and a caret; clicking it
/// opens a popup below, which `add_options` fills with [`select_option`] rows.
///
/// `id_source` must be stable and unique — the popup's open state is keyed off
/// it. `width` fixes the trigger width. Returns the trigger [`Response`]; the
/// caller learns of a new choice from the [`select_option`] it builds.
pub fn select(
    ui: &mut Ui,
    t: &Tokens,
    id_source: impl Hash,
    current: &str,
    width: f32,
    add_options: impl FnOnce(&mut Ui),
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 34.0), Sense::click());
    let popup_id = egui::Id::new(id_source);
    let open = ui.memory(|m| m.is_popup_open(popup_id));
    let hv = hover_t(ui, response.id, response.hovered() || open);

    ui.painter().rect_filled(rect, t.rounding_sm(), t.bg_chrome);
    let border = if open {
        t.accent
    } else {
        lerp_color(t.border, t.border_strong, hv)
    };
    ui.painter()
        .rect_stroke(rect.shrink(0.5), t.rounding_sm(), Stroke::new(1.0, border));
    ui.painter().text(
        pos2(rect.left() + 11.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        current,
        TextStyle::Body.resolve(ui.style()),
        t.text,
    );
    ui.painter().text(
        pos2(rect.right() - 11.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        icons::ph::CARET_DOWN,
        icons::font(13.0),
        t.text_3,
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if response.clicked() {
        ui.memory_mut(|m| m.toggle_popup(popup_id));
    }
    egui::popup::popup_below_widget(
        ui,
        popup_id,
        &response,
        egui::PopupCloseBehavior::CloseOnClick,
        |ui| {
            ui.set_min_width(width);
            add_options(ui);
        },
    );
    response
}

/// One option row inside a [`select`] popup. Shows a tick when `selected`,
/// and returns `true` on the frame it is clicked (which also closes the menu).
pub fn select_option(ui: &mut Ui, t: &Tokens, label: &str, selected: bool) -> bool {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        icons::ph::CHECK,
        0.0,
        egui::text::TextFormat {
            font_id: icons::font(13.0),
            // Transparent (not omitted) so selected and unselected rows align.
            color: if selected {
                t.accent
            } else {
                Color32::TRANSPARENT
            },
            ..Default::default()
        },
    );
    job.append(
        label,
        8.0,
        egui::text::TextFormat {
            font_id: TextStyle::Body.resolve(ui.style()),
            color: t.text,
            ..Default::default()
        },
    );
    list_row(ui, t, job, selected).clicked()
}

// ---------------------------------------------------------------------------
// banner & collapsing
// ---------------------------------------------------------------------------

/// Visual tone of a [`banner`] — picks its accent colour.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BannerKind {
    /// Positive / ready state — the `success` colour.
    Success,
    /// Error / blocking state — the `danger` colour.
    Danger,
    /// Caution — the `warning` colour.
    Warning,
    /// Neutral information — a muted grey.
    Info,
}

/// A full-width status callout: a leading icon, a bold `title`, and a wrapped
/// muted `body` line, on a tinted panel.
///
/// `kind` sets the accent colour; `glyph` is the leading [`icons::ph`] icon.
/// The banner sizes its height to the wrapped body text.
pub fn banner(
    ui: &mut Ui,
    t: &Tokens,
    kind: BannerKind,
    glyph: &str,
    title: &str,
    body: &str,
) -> Response {
    let accent = match kind {
        BannerKind::Success => t.success,
        BannerKind::Danger => t.danger,
        BannerKind::Warning => t.warning,
        BannerKind::Info => t.text_2,
    };
    let pad = t.space_3;
    let icon_box = 22.0;
    let width = ui.available_width();
    let text_left = pad + icon_box + 10.0;

    let title_galley = ui.painter().layout_no_wrap(
        title.to_owned(),
        TextStyle::Body.resolve(ui.style()),
        t.text,
    );
    let body_galley = ui.painter().layout(
        body.to_owned(),
        TextStyle::Small.resolve(ui.style()),
        t.text_2,
        (width - text_left - pad).max(40.0),
    );
    let title_h = title_galley.size().y;
    let content_h = title_h + 3.0 + body_galley.size().y;
    let height = (content_h + pad * 2.0).max(icon_box + pad * 2.0);

    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    ui.painter().rect_filled(
        rect,
        t.rounding_md(),
        accent.gamma_multiply(if t.dark { 0.16 } else { 0.10 }),
    );
    ui.painter().rect_stroke(
        rect.shrink(0.5),
        t.rounding_md(),
        Stroke::new(1.0, accent.gamma_multiply(0.55)),
    );
    ui.painter().text(
        pos2(
            rect.left() + pad + icon_box / 2.0,
            rect.top() + pad + icon_box / 2.0,
        ),
        egui::Align2::CENTER_CENTER,
        glyph,
        icons::font(18.0),
        accent,
    );
    let tx = rect.left() + text_left;
    let ty = rect.top() + (height - content_h) / 2.0;
    ui.painter().galley(pos2(tx, ty), title_galley, t.text);
    ui.painter()
        .galley(pos2(tx, ty + title_h + 3.0), body_galley, t.text_2);
    response
}

/// A collapsible section: a clickable header (caret + `label`) that shows or
/// hides `add_body`.
///
/// Open state persists in egui memory under `id_source`, which must be stable
/// and unique. Use it for "Advanced options" disclosure.
pub fn collapsing(
    ui: &mut Ui,
    t: &Tokens,
    id_source: impl Hash,
    label: &str,
    add_body: impl FnOnce(&mut Ui),
) {
    let id = egui::Id::new(id_source).with("tokito_ui_collapsing");
    let mut open = ui.data(|d| d.get_temp::<bool>(id).unwrap_or(false));

    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::click());
    if response.clicked() {
        open = !open;
        ui.data_mut(|d| d.insert_temp(id, open));
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let hv = hover_t(ui, response.id, response.hovered());
    let ink = lerp_color(t.text_2, t.text, hv);
    let caret = if open {
        icons::ph::CARET_DOWN
    } else {
        icons::ph::CARET_RIGHT
    };
    ui.painter().text(
        pos2(rect.left() + 2.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        caret,
        icons::font(13.0),
        ink,
    );
    ui.painter().text(
        pos2(rect.left() + 20.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        TextStyle::Button.resolve(ui.style()),
        ink,
    );
    if open {
        ui.add_space(t.space_2);
        add_body(ui);
    }
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
