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
use std::{hash::Hash, time::Duration};

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
    painter.rect_stroke(
        rect.shrink(0.5),
        t.rounding_md(),
        Stroke::new(1.0, border),
        egui::StrokeKind::Outside,
    );

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
/// hover the dashed border eases toward the accent while the tile body stays
/// transparent.
pub fn new_tile(
    ui: &mut Ui,
    t: &Tokens,
    label: &str,
    sublabel: Option<&str>,
    size: Vec2,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hv = hover_t(ui, response.id, response.hovered());

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
/// `side` is the button's width and height; `ink` is the glyph's rest colour
/// (pass `t.text_2` for a muted button, `t.text` or `t.accent` for a
/// prominent one). Hover eases a soft background in and brightens the glyph.
pub fn icon_button(ui: &mut Ui, t: &Tokens, glyph: &str, side: f32, ink: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
    let hv = hover_t(ui, response.id, response.hovered());
    if hv > 0.001 {
        ui.painter()
            .rect_filled(rect, t.rounding_sm(), t.card.gamma_multiply(hv));
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            t.rounding_sm(),
            Stroke::new(1.0, t.border.gamma_multiply(hv)),
            egui::StrokeKind::Outside,
        );
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        icons::font(side * 0.5),
        lerp_color(ink, Color32::WHITE, 0.3 * hv),
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

/// Fill / ink / border for a [`ButtonKind`] at hover factor `hv`. Shared by
/// [`text_button`] and [`icon_text_button`] so the two stay visually
/// identical — only the content laid out inside the rect differs.
fn button_visuals(t: &Tokens, kind: ButtonKind, hv: f32) -> (Color32, Color32, Option<Color32>) {
    match kind {
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
    }
}

/// Paint a button's rounded fill + optional border. Shared tail of
/// [`text_button`] / [`icon_text_button`] after each lays out its own
/// content (label-only vs. icon-plus-label).
fn paint_button_chrome(ui: &Ui, rect: Rect, t: &Tokens, fill: Color32, border: Option<Color32>) {
    ui.painter().rect_filled(rect, t.rounding_sm(), fill);
    if let Some(b) = border {
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            t.rounding_sm(),
            Stroke::new(1.0, b),
            egui::StrokeKind::Outside,
        );
    }
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

    let (fill, ink, border) = button_visuals(t, kind, hv);
    paint_button_chrome(ui, rect, t, fill, border);
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, ink);
    response
}

/// A [`text_button`] with a leading Phosphor glyph (`icon`, an
/// [`icons::ph`] constant). Same visual weights and hover animation as
/// [`text_button`] — use this when the action reads better with an icon
/// ("Sign in to Tokito Cloud", "Retry"), and plain `text_button` otherwise.
pub fn icon_text_button(
    ui: &mut Ui,
    t: &Tokens,
    kind: ButtonKind,
    icon: &str,
    label: &str,
    height: f32,
) -> Response {
    let icon_size = height * 0.46;
    let gap = t.space_2;
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        TextStyle::Button.resolve(ui.style()),
        Color32::PLACEHOLDER,
    );
    let width = icon_size + gap + galley.size().x + t.space_4 * 2.0;
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let hv = hover_t(ui, response.id, response.hovered());

    let (fill, ink, border) = button_visuals(t, kind, hv);
    paint_button_chrome(ui, rect, t, fill, border);

    let content_w = icon_size + gap + galley.size().x;
    let start_x = rect.center().x - content_w / 2.0;
    let icon_pos = pos2(start_x, rect.center().y);
    ui.painter().text(
        icon_pos,
        egui::Align2::LEFT_CENTER,
        icon,
        icons::font(icon_size),
        ink,
    );
    let label_pos = pos2(
        start_x + icon_size + gap,
        rect.center().y - galley.size().y / 2.0,
    );
    ui.painter().galley(label_pos, galley, ink);
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
        egui::StrokeKind::Outside,
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
    id_source: impl Hash + std::fmt::Debug,
    glyph: &str,
    side: f32,
    add_items: impl FnOnce(&mut Ui),
) -> Response {
    let trigger = icon_button(ui, t, glyph, side, t.text_2);
    let popup_id = egui::Id::new(id_source);
    egui::Popup::from_toggle_button_response(&trigger)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
        .show(|ui| {
            ui.set_min_width(184.0);
            add_items(ui);
        });
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
    let galley = ui.fonts_mut(|f| f.layout_job(job));
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
    id_source: impl Hash + std::fmt::Debug,
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
    id_source: impl Hash + std::fmt::Debug,
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
    id_source: impl Hash + std::fmt::Debug,
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
    ui.painter().rect_stroke(
        rect.shrink(0.5),
        t.rounding_sm(),
        Stroke::new(1.0, border),
        egui::StrokeKind::Outside,
    );
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
            // Explicitly dim — egui's default hint color tracks the theme's
            // weak text, which in our dark tokens is nearly full-ink and
            // reads as real content.
            .hint_text(egui::RichText::new(hint).color(t.text_3))
            .frame(egui::Frame::NONE)
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
        egui::CornerRadius::same((track.y / 2.0) as u8),
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

    let screen = ctx.content_rect();
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
            egui::Frame::new()
                .fill(t.bg_chrome)
                .stroke(Stroke::new(1.0, t.border_strong))
                .corner_radius(t.rounding_md())
                .inner_margin(egui::Margin::same((t.space_4) as i8))
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
                            if icon_button(ui, t, icons::ph::X, 26.0, t.text_2).clicked() {
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
        egui::StrokeKind::Outside,
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
        egui::StrokeKind::Outside,
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
    id_source: impl Hash + std::fmt::Debug,
    current: &str,
    width: f32,
    add_options: impl FnOnce(&mut Ui),
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 34.0), Sense::click());
    let popup_id = egui::Id::new(id_source);
    let open = egui::Popup::is_id_open(ui.ctx(), popup_id);
    let hv = hover_t(ui, response.id, response.hovered() || open);

    ui.painter().rect_filled(rect, t.rounding_sm(), t.bg_chrome);
    let border = if open {
        t.accent
    } else {
        lerp_color(t.border, t.border_strong, hv)
    };
    ui.painter().rect_stroke(
        rect.shrink(0.5),
        t.rounding_sm(),
        Stroke::new(1.0, border),
        egui::StrokeKind::Outside,
    );
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
    egui::Popup::from_toggle_button_response(&response)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
        .show(|ui| {
            ui.set_min_width(width);
            add_options(ui);
        });
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
        egui::StrokeKind::Outside,
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
    id_source: impl Hash + std::fmt::Debug,
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

// ---------------------------------------------------------------------------
// cad_tool_button
// ---------------------------------------------------------------------------

/// A square, toggleable CAD-tool-rail button.
///
/// Used for the left-side tool rail in a schematic / PCB editor (select,
/// wire, label, bus, etc.). `side` is the width and height; `selected` paints
/// the active state (accent border + soft accent fill); `tooltip` shows on
/// hover.
///
/// `paint_icon` is invoked with the button's `Painter`, the inner `Rect`, and
/// the current ink colour — the caller decides what symbol to draw (Phosphor
/// glyph, hand-drawn schematic strokes, an image, whatever). For the common
/// Phosphor case, use [`paint_phosphor_glyph`] as the closure.
///
/// Hover eases an underlay fill in; the ink is `accent` when selected,
/// `text` otherwise (interpolated with `text_2` on hover).
pub fn cad_tool_button<F>(
    ui: &mut Ui,
    t: &Tokens,
    side: f32,
    selected: bool,
    tooltip: &str,
    paint_icon: F,
) -> Response
where
    F: FnOnce(&egui::Painter, Rect, Color32),
{
    let (rect, mut response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());

    let factor = hover_t(ui, response.id, response.hovered());
    let painter = ui.painter();

    let (fill, stroke) = if selected {
        let stroke = Stroke::new(1.2, t.accent);
        let fill = lerp_color(t.accent_soft, lighten(t.accent_soft, 0.10), factor);
        (fill, stroke)
    } else {
        let fill = lerp_color(t.card, t.card_hover, factor);
        let stroke_color = lerp_color(t.border, t.border_strong, factor);
        let stroke = Stroke::new(1.0, stroke_color);
        (fill, stroke)
    };

    painter.rect_filled(rect, t.rounding_sm(), fill);
    painter.rect_stroke(rect, t.rounding_sm(), stroke, egui::StrokeKind::Outside);

    let ink = if selected {
        t.accent
    } else {
        lerp_color(t.text_2, t.text, factor)
    };
    paint_icon(painter, rect, ink);

    if !tooltip.is_empty() {
        response = response.on_hover_text(tooltip);
    }
    response
}

/// Helper closure for [`cad_tool_button`] that paints a centred Phosphor
/// glyph at a sensible size for the button.
///
/// Usage:
/// ```ignore
/// cad_tool_button(ui, &t, 38.0, selected, "Wire", paint_phosphor_glyph(icons::ph::PEN_NIB))
/// ```
pub fn paint_phosphor_glyph(glyph: &'static str) -> impl FnOnce(&egui::Painter, Rect, Color32) {
    move |painter, rect, ink| {
        let side = rect.width().min(rect.height());
        let glyph_size = (side * 0.5).clamp(14.0, 24.0);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            glyph,
            icons::font(glyph_size),
            ink,
        );
    }
}

// ---------------------------------------------------------------------------
// table
// ---------------------------------------------------------------------------

/// Sort direction for a [`SortState`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortDir {
    /// Unsorted — the natural row order.
    #[default]
    None,
    /// Ascending — `A..Z`, `0..9`, oldest-first.
    Asc,
    /// Descending — `Z..A`, `9..0`, newest-first.
    Desc,
}

impl SortDir {
    fn arrow(self) -> &'static str {
        match self {
            SortDir::None => "",
            SortDir::Asc => " ▲",
            SortDir::Desc => " ▼",
        }
    }
}

/// Which column is currently the sort key and in what direction.
///
/// Stored by the consumer (e.g. one [`SortState`] per visible table). Drive
/// from [`sortable_header`] click responses, then read in your row-building
/// loop to decide order.
#[derive(Debug, Clone, Copy, Default)]
pub struct SortState {
    pub column: usize,
    pub dir: SortDir,
}

impl SortState {
    /// Click handler for header column `col`. Cycles
    /// `None → Asc → Desc → None`, or resets to `Asc` when switching columns.
    pub fn toggle(&mut self, col: usize) {
        if self.column == col {
            self.dir = match self.dir {
                SortDir::None => SortDir::Asc,
                SortDir::Asc => SortDir::Desc,
                SortDir::Desc => SortDir::None,
            };
        } else {
            self.column = col;
            self.dir = SortDir::Asc;
        }
    }
}

/// A clickable column-header label that updates a [`SortState`].
///
/// Shows an arrow suffix when this column is the active sort key. Returns
/// `true` on the frame the label is clicked (caller uses this only if it
/// wants side-effects beyond `state.toggle(col)`).
pub fn sortable_header(
    ui: &mut Ui,
    t: &Tokens,
    label: &str,
    col: usize,
    state: &mut SortState,
) -> bool {
    let active = state.column == col && state.dir != SortDir::None;
    let arrow = if active { state.dir.arrow() } else { "" };
    let text = RichText::new(format!("{label}{arrow}"))
        .strong()
        .color(if active { t.text } else { t.text_2 });

    let resp = ui.add(egui::Label::new(text).sense(Sense::click()));
    let clicked = resp.clicked();
    if clicked {
        state.toggle(col);
    }
    clicked
}

/// A scrollable table with [`sortable_header`]-driven sortable columns.
///
/// `id_source` salts the inner scroll area's id so multiple tables on one
/// screen don't collide. `headers` is one label per column. `row_height` is
/// the per-row height for [`egui_extras::TableBuilder`]. `cols` describes the
/// column widths — pass [`egui_extras::Column`] values.
///
/// The `build_row` closure paints one cell per column for a given row index.
/// Callers usually pre-sort their data by `state` *before* calling this, then
/// index into the sorted vector inside `build_row`.
pub fn data_table<F>(
    ui: &mut Ui,
    t: &Tokens,
    id_source: impl Hash + std::fmt::Debug,
    headers: &[&str],
    cols: Vec<egui_extras::Column>,
    state: &mut SortState,
    row_count: usize,
    row_height: f32,
    mut build_row: F,
) where
    F: FnMut(&mut egui_extras::TableRow<'_, '_>, usize),
{
    let id = ui.make_persistent_id(id_source);
    let mut builder = egui_extras::TableBuilder::new(ui)
        .id_salt(id)
        .striped(true)
        .resizable(false)
        .cell_layout(Layout::left_to_right(Align::Center));
    for c in cols {
        builder = builder.column(c);
    }
    builder
        .header(22.0, |mut header| {
            for (col, label) in headers.iter().enumerate() {
                header.col(|ui| {
                    sortable_header(ui, t, label, col, state);
                });
            }
        })
        .body(|body| {
            body.rows(row_height, row_count, |row| {
                let idx = row.index();
                let mut row = row;
                build_row(&mut row, idx);
            });
        });
}

// ---------------------------------------------------------------------------
// toast
// ---------------------------------------------------------------------------

/// Visual + semantic kind of a [`Toast`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToastKind {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl ToastKind {
    /// Leading Phosphor glyph painted in the toast header.
    fn icon(self) -> &'static str {
        match self {
            ToastKind::Info => icons::ph::INFO,
            ToastKind::Success => icons::ph::CHECK_CIRCLE,
            ToastKind::Warning => icons::ph::WARNING,
            ToastKind::Error => icons::ph::WARNING_CIRCLE,
        }
    }

    /// Short header label.
    fn title(self) -> &'static str {
        match self {
            ToastKind::Info => "Info",
            ToastKind::Success => "Success",
            ToastKind::Warning => "Warning",
            ToastKind::Error => "Error",
        }
    }

    /// The kind's accent colour (icon, title, border).
    fn accent(self, t: &Tokens) -> Color32 {
        match self {
            ToastKind::Info => t.accent,
            ToastKind::Success => t.success,
            ToastKind::Warning => t.warning,
            ToastKind::Error => t.danger,
        }
    }

    /// Whether a toast of this kind persists until manually dismissed.
    /// Errors and warnings are actionable, so they stick; transient
    /// info/success toasts auto-expire after [`ToastStack::DEFAULT_TTL`].
    fn sticky(self) -> bool {
        matches!(self, ToastKind::Error | ToastKind::Warning)
    }
}

/// One notification message.
#[derive(Debug, Clone)]
pub struct Toast {
    /// Stable identity, used to dismiss a specific toast.
    id: u64,
    /// Optional dedupe key. A keyed toast replaces any existing toast with
    /// the same key instead of stacking a duplicate — for live status that
    /// updates in place (e.g. an ERC summary) rather than discrete events.
    key: Option<String>,
    /// What the user sees.
    pub message: String,
    /// Visual + semantic class.
    pub kind: ToastKind,
    /// When this toast auto-expires. `None` means it sticks until the user
    /// dismisses it with the ✕ button (errors and warnings).
    until: Option<std::time::Instant>,
}

/// A queue of [`Toast`]s, drained by [`toast_overlay`].
///
/// Holders own this struct in their app state and call `push` from anywhere
/// in the update loop; once per frame they hand it to [`toast_overlay`] to
/// paint. Expired entries are pruned automatically; sticky entries stay until
/// dismissed.
#[derive(Debug, Default, Clone)]
pub struct ToastStack {
    items: Vec<Toast>,
    next_id: u64,
}

impl ToastStack {
    /// Default visible time per non-sticky toast — currently 4 seconds.
    pub const DEFAULT_TTL: std::time::Duration = std::time::Duration::from_secs(4);

    /// Newest toasts rendered at once; older live ones queue behind them.
    pub const MAX_VISIBLE: usize = 5;

    /// Hard cap on retained toasts, visible or not. `push`/`set_keyed` evict
    /// down to this once it's exceeded, so a recurring non-keyed
    /// `push_error`/`push_warning` (sticky, never auto-expiring) can't grow
    /// `items` without bound over a long-running session. The currently
    /// *visible* window (the newest [`Self::MAX_VISIBLE`] toasts, including
    /// sticky errors) is never evicted — only older, already-invisible
    /// entries are dropped, oldest/already-expired first.
    pub const MAX_RETAINED: usize = 100;

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    fn expiry(kind: ToastKind) -> Option<std::time::Instant> {
        if kind.sticky() {
            None
        } else {
            Some(std::time::Instant::now() + Self::DEFAULT_TTL)
        }
    }

    /// Push a new toast. Sticky kinds (error/warning) persist until dismissed;
    /// others auto-expire after [`Self::DEFAULT_TTL`].
    pub fn push(&mut self, message: impl Into<String>, kind: ToastKind) {
        let id = self.alloc_id();
        self.items.push(Toast {
            id,
            key: None,
            message: message.into(),
            kind,
            until: Self::expiry(kind),
        });
        self.enforce_capacity();
    }

    /// Upsert a **keyed** toast: if one with `key` already exists its message
    /// and kind are updated in place; otherwise a new one is inserted. Use for
    /// live status that changes over time rather than discrete events, so it
    /// never stacks duplicates. Keyed toasts are always sticky.
    pub fn set_keyed(
        &mut self,
        key: impl Into<String>,
        message: impl Into<String>,
        kind: ToastKind,
    ) {
        let key = key.into();
        let message = message.into();
        if let Some(existing) = self
            .items
            .iter_mut()
            .find(|t| t.key.as_deref() == Some(&key))
        {
            existing.message = message;
            existing.kind = kind;
            existing.until = None;
            return;
        }
        let id = self.alloc_id();
        self.items.push(Toast {
            id,
            key: Some(key),
            message,
            kind,
            until: None,
        });
        self.enforce_capacity();
    }

    /// Remove the keyed toast with `key`, if present. No-op otherwise.
    pub fn clear_keyed(&mut self, key: &str) {
        self.items.retain(|t| t.key.as_deref() != Some(key));
    }

    /// Push an [`ToastKind::Info`] toast.
    pub fn push_info(&mut self, message: impl Into<String>) {
        self.push(message, ToastKind::Info);
    }

    /// Push a [`ToastKind::Success`] toast.
    pub fn push_success(&mut self, message: impl Into<String>) {
        self.push(message, ToastKind::Success);
    }

    /// Push a [`ToastKind::Warning`] toast.
    pub fn push_warning(&mut self, message: impl Into<String>) {
        self.push(message, ToastKind::Warning);
    }

    /// Push a [`ToastKind::Error`] toast.
    pub fn push_error(&mut self, message: impl Into<String>) {
        self.push(message, ToastKind::Error);
    }

    /// True when there are no live toasts.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn prune(&mut self) {
        let now = std::time::Instant::now();
        self.items.retain(|t| t.until.is_none_or(|u| u > now));
    }

    /// Bound `items` to [`Self::MAX_RETAINED`] after an insert.
    ///
    /// `items` is oldest-first (pushes append), and [`toast_overlay`] renders
    /// the newest [`Self::MAX_VISIBLE`] — that tail is the "currently
    /// visible" window and is never touched here, so a live sticky error on
    /// screen can never be evicted out from under the user. Already-expired
    /// entries are dropped first (mirrors [`Self::prune`], since `push` can
    /// run several times between overlay frames); if that alone isn't enough,
    /// the oldest remaining invisible entries are dropped next, regardless of
    /// kind — an off-screen sticky error the user will never scroll back to
    /// is retained no better than a duplicate one further down the queue.
    fn enforce_capacity(&mut self) {
        if self.items.len() <= Self::MAX_RETAINED {
            return;
        }
        let now = std::time::Instant::now();
        self.items.retain(|t| t.until.is_none_or(|u| u > now));

        let visible = Self::MAX_VISIBLE.min(self.items.len());
        while self.items.len() > Self::MAX_RETAINED {
            let evictable = self.items.len() - visible;
            if evictable == 0 {
                // Nothing left outside the visible window to evict — the cap
                // is smaller than MAX_VISIBLE (misconfiguration); stop rather
                // than evict something on screen.
                break;
            }
            self.items.remove(0);
        }
    }
}

/// Paint live toasts anchored to the bottom-right of the egui screen.
///
/// Call once per frame. The stack is mutated in place: expired (non-sticky)
/// toasts are pruned, toasts the user dismisses via the ✕ button are removed,
/// and egui is asked to repaint while a timed toast is still live so its
/// auto-dismissal happens on time. Each toast shows a kind icon + title, the
/// message, and a dismiss button.
pub fn toast_overlay(ctx: &egui::Context, t: &Tokens, stack: &mut ToastStack) {
    stack.prune();
    if stack.is_empty() {
        return;
    }

    // Keep repainting only while a *timed* toast is live, so it vanishes on
    // time even when nothing else moves; all-sticky stacks need no ticking.
    if stack.items.iter().any(|toast| toast.until.is_some()) {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    let mut dismiss: Option<u64> = None;
    egui::Area::new(egui::Id::new("tokito_ui_toasts"))
        .anchor(egui::Align2::RIGHT_BOTTOM, [-16.0, -16.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Newest-first: most-recent push appears on top.
                for toast in stack.items.iter().rev().take(ToastStack::MAX_VISIBLE) {
                    let accent = toast.kind.accent(t);
                    egui::Frame::popup(ui.style())
                        .fill(t.card)
                        .stroke(Stroke::new(1.0, accent))
                        .corner_radius(t.rounding_md())
                        .inner_margin(egui::Margin::symmetric((12.0) as i8, (10.0) as i8))
                        .show(ui, |ui| {
                            ui.set_width(300.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(toast.kind.icon())
                                        .color(accent)
                                        .font(icons::font(15.0)),
                                );
                                ui.add_space(t.space_1);
                                ui.label(RichText::new(toast.kind.title()).strong().color(accent));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if icon_button(ui, t, icons::ph::X, 20.0, t.text_3).clicked() {
                                        dismiss = Some(toast.id);
                                    }
                                });
                            });
                            ui.add_space(t.space_1);
                            ui.add(
                                egui::Label::new(RichText::new(&toast.message).color(t.text))
                                    .wrap(),
                            );
                        });
                    ui.add_space(t.space_2);
                }
            });
        });

    if let Some(id) = dismiss {
        stack.items.retain(|toast| toast.id != id);
    }
}

#[cfg(test)]
mod toast_stack_tests {
    use super::*;

    /// A recurring failure condition (e.g. repeated catalog-transport
    /// errors) hammering `push_error` for a long-running session must not
    /// grow `items` without bound — that was the whole bug (#26).
    #[test]
    fn push_error_is_bounded_under_hammering() {
        let mut stack = ToastStack::default();
        for i in 0..10_000 {
            stack.push_error(format!("error {i}"));
        }
        assert!(
            stack.items.len() <= ToastStack::MAX_RETAINED,
            "items grew unbounded: {} entries after 10k pushes",
            stack.items.len()
        );
    }

    /// Same hammering, mixing in warnings (also sticky) and infos (timed) —
    /// the mix shouldn't change the bound.
    #[test]
    fn mixed_kinds_stay_bounded_under_hammering() {
        let mut stack = ToastStack::default();
        for i in 0..10_000 {
            match i % 3 {
                0 => stack.push_error(format!("error {i}")),
                1 => stack.push_warning(format!("warning {i}")),
                _ => stack.push_info(format!("info {i}")),
            }
        }
        assert!(stack.items.len() <= ToastStack::MAX_RETAINED);
    }

    /// `toast_overlay` renders `items.iter().rev().take(MAX_VISIBLE)` — the
    /// newest `MAX_VISIBLE` toasts. Those must survive eviction even under
    /// heavy hammering, so a sticky error currently on screen never
    /// disappears out from under the user.
    #[test]
    fn newest_visible_stickies_survive_hammering() {
        let mut stack = ToastStack::default();
        for i in 0..10_000u32 {
            stack.push_error(format!("error {i}"));
        }
        let visible: Vec<&str> = stack
            .items
            .iter()
            .rev()
            .take(ToastStack::MAX_VISIBLE)
            .map(|t| t.message.as_str())
            .collect();
        let expected: Vec<String> = (10_000 - ToastStack::MAX_VISIBLE as u32..10_000)
            .rev()
            .map(|i| format!("error {i}"))
            .collect();
        assert_eq!(visible, expected);
    }

    /// Dismissing a toast (the ✕ button in `toast_overlay`, reproduced here
    /// via the same `retain(|t| t.id != id)` it uses) drops it immediately —
    /// it must not linger invisibly waiting for capacity eviction.
    #[test]
    fn dismissed_toast_is_removed_immediately() {
        let mut stack = ToastStack::default();
        stack.push_error("first");
        stack.push_error("second");
        stack.push_error("third");
        assert_eq!(stack.items.len(), 3);

        let dismiss_id = stack.items[1].id;
        stack.items.retain(|t| t.id != dismiss_id);

        assert_eq!(stack.items.len(), 2);
        assert!(stack.items.iter().all(|t| t.id != dismiss_id));
        assert_eq!(stack.items[0].message, "first");
        assert_eq!(stack.items[1].message, "third");
    }

    /// When eviction is forced, an already-expired (but not yet pruned)
    /// entry goes before any live one, even an older live one.
    #[test]
    fn capacity_eviction_prefers_expired_over_oldest_live() {
        let mut stack = ToastStack::default();
        stack.push_info("stale");
        stack.items[0].until = Some(std::time::Instant::now() - std::time::Duration::from_secs(1));

        for i in 0..ToastStack::MAX_RETAINED {
            stack.push_error(format!("error {i}"));
        }

        assert!(stack.items.len() <= ToastStack::MAX_RETAINED);
        assert!(
            stack.items.iter().all(|t| t.kind == ToastKind::Error),
            "expired info toast should have been evicted before any live error"
        );
    }

    /// `set_keyed` goes through the same capacity enforcement as `push`.
    #[test]
    fn set_keyed_is_bounded_under_hammering() {
        let mut stack = ToastStack::default();
        for i in 0..10_000 {
            stack.set_keyed(format!("key-{i}"), format!("status {i}"), ToastKind::Error);
        }
        assert!(stack.items.len() <= ToastStack::MAX_RETAINED);
    }
}

// ---------------------------------------------------------------------------
// chip
// ---------------------------------------------------------------------------

/// A small toggleable pill — like [`badge`], but clickable and with a
/// selected state.
///
/// Used for filter chips, tag pills, and any narrow on/off control where a
/// full [`toggle`] is too heavy. Returns `true` on the frame the chip is
/// clicked; the caller flips its own `selected` state.
pub fn chip(ui: &mut Ui, t: &Tokens, label: &str, selected: bool) -> bool {
    let (fill, stroke_color, ink) = if selected {
        (t.accent_soft, t.accent, t.text)
    } else {
        (t.card, t.border, t.text_2)
    };
    let resp = ui.add(
        egui::Button::new(RichText::new(label).size(11.0).color(ink))
            .fill(fill)
            .stroke(Stroke::new(1.0, stroke_color))
            .corner_radius(t.rounding_sm())
            .min_size(vec2(0.0, 28.0)),
    );
    resp.clicked()
}

// ---------------------------------------------------------------------------
// content_card
// ---------------------------------------------------------------------------

/// A bordered, padded panel for grouping content — the typical "section in a
/// settings page" or "block in a side panel" container.
///
/// Unlike [`card`], `content_card` is **not** click-able and **not** fixed
/// size: it grows to fit `add_contents`. Width is whatever the parent layout
/// gives it. Padding is `space_4` on all sides.
pub fn content_card(ui: &mut Ui, t: &Tokens, add_contents: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(t.card)
        .corner_radius(t.rounding_md())
        .inner_margin(egui::Margin::same((t.space_4) as i8))
        .stroke(Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            add_contents(ui);
        });
}

// ---------------------------------------------------------------------------
// inspector_row
// ---------------------------------------------------------------------------

/// A label-on-the-left, value-on-the-right key/value row.
///
/// Used in inspector / property panels and detail cards. The label is muted
/// (`text_2`), the value uses the primary `text` colour. Both are small.
pub fn inspector_row(ui: &mut Ui, t: &Tokens, label: &str, value: impl Into<String>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).small().color(t.text_2));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value.into()).small().color(t.text));
        });
    });
    ui.add_space(2.0);
}

// ---------------------------------------------------------------------------
// list_section_label
// ---------------------------------------------------------------------------

/// A small "Symbols (24)" style label that groups items in a list / side
/// panel.
///
/// Smaller and lighter than [`section_header`] — meant for use within a
/// dense scrollable list, not as a top-of-page heading. The count appears
/// in parentheses after the label.
pub fn list_section_label(ui: &mut Ui, t: &Tokens, label: &str, count: usize) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(format!("{label} ({count})"))
            .small()
            .strong()
            .color(t.text_2),
    );
    ui.add_space(4.0);
}

// ---------------------------------------------------------------------------
// empty_state
// ---------------------------------------------------------------------------

/// A muted "nothing to show here" placeholder card.
///
/// Used as the body content of a panel that would otherwise be empty
/// (no search results, no items in the list, no recent files). Centred
/// text, soft card background, no border.
pub fn empty_state(ui: &mut Ui, t: &Tokens, message: &str) {
    egui::Frame::new()
        .fill(t.card)
        .corner_radius(t.rounding_sm())
        .inner_margin(egui::Margin::same((14.0) as i8))
        .show(ui, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(message).size(12.0).color(t.text_2));
            });
        });
}

// ---------------------------------------------------------------------------
// gate_overlay
// ---------------------------------------------------------------------------

/// A full-bleed gated / locked state: a heavy tinted scrim filling `ui`'s
/// current available rect, with a centred vertical stack — a rounded
/// brand-tile visual carrying a small corner badge glyph, a display-weight
/// heading, one muted subline, and a primary CTA with a leading icon.
///
/// egui has no real backdrop blur, so the scrim is approximated with a
/// high-opacity base fill plus a soft two-layer accent glow behind the
/// card for depth, rather than a translucent wash over live content —
/// give this its own `Ui` scoped to exactly the region that should read as
/// gated (e.g. a `CentralPanel` spanning a message list *and* its
/// composer), not a thin strip over just part of it.
///
/// Generalized on purpose: only the icon, heading, body copy, and button
/// icon/label are baked in here — what the button *does* is entirely the
/// caller's concern. Today's only consumer is the chat panel's Tokito
/// Cloud sign-in gate, but any future gated/empty chrome state (a locked
/// feature, an unconfigured integration, …) can reuse this unchanged.
///
/// Returns `true` on the frame the CTA is clicked.
#[allow(clippy::too_many_arguments)]
pub fn gate_overlay(
    ui: &mut Ui,
    t: &Tokens,
    badge_icon: &str,
    heading: &str,
    body: &str,
    button_icon: &str,
    button_label: &str,
) -> bool {
    let rect = ui.available_rect_before_wrap();
    // Claim the whole area as click-and-drag so nothing behind it is
    // clickable while gated — `Sense::hover()` alone does not intercept
    // clicks under egui 0.35's hit-testing (a hover-only rect never blocks
    // a click from reaching whatever egui would otherwise hit at that
    // position), so it would silently fail to actually gate the area.
    ui.allocate_rect(rect, Sense::click_and_drag());

    // Layer 1: a near-opaque base — the closest egui gets to a heavy
    // backdrop scrim without real blur support.
    let scrim = if t.dark {
        t.bg.gamma_multiply(0.97)
    } else {
        t.bg
    };
    ui.painter().rect_filled(rect, 0.0, scrim);

    // Layer 2: a soft accent glow centred behind the card, for depth — not
    // a real blur, but a multi-step alpha ramp (several concentric fills,
    // each fainter and larger than the last) reads as a soft falloff
    // rather than the hard-edged concentric rings a single pair of flat
    // circles produced. Colour drifts from `accent` at the rim to
    // `accent_2` at the core so the glow itself carries the two-tone brand
    // gradient instead of a flat wash.
    let glow_r = (rect.width().min(rect.height()) * 0.42).max(120.0);
    const GLOW_STEPS: usize = 7;
    for step in 0..GLOW_STEPS {
        // `f` sweeps 0.0 (outermost, faintest) .. 1.0 (innermost, most
        // saturated) — painted in that order so each smaller, stronger
        // ring layers on top of the softer ones behind it.
        let f = step as f32 / (GLOW_STEPS - 1) as f32;
        let r = glow_r * (1.0 - f * 0.8);
        let colour = lerp_color(t.accent_soft, t.accent_2_soft, f);
        let step_colour = colour.gamma_multiply(0.3 + 0.7 * f);
        ui.painter().circle_filled(rect.center(), r, step_colour);
    }

    let mut clicked = false;
    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.allocate_ui_with_layout(rect.size(), Layout::top_down(Align::Center), |ui| {
            ui.add_space((rect.height() * 0.5 - 150.0).max(rect.height() * 0.12));

            // Visual: the brand tile with a small lock badge overlapped at
            // its bottom-right corner (falls back to a plain rounded tile
            // if the caller ever needs a badge-less variant — not needed
            // today, so not parameterised yet).
            let tile_side = 72.0;
            let tile_resp = crate::brand::brand_tile(ui, tile_side);
            let badge_d = 30.0;
            let badge_c = tile_resp.rect.right_bottom() - Vec2::splat(badge_d * 0.32);
            // Separation ring so the badge reads as sitting *on* the tile
            // rather than merging into it.
            ui.painter()
                .circle_filled(badge_c, badge_d * 0.5 + 3.0, scrim);
            ui.painter().circle_filled(badge_c, badge_d * 0.5, t.accent);
            ui.painter().text(
                badge_c,
                egui::Align2::CENTER_CENTER,
                badge_icon,
                icons::font(badge_d * 0.56),
                t.accent_ink,
            );

            ui.add_space(t.space_5);
            ui.label(
                RichText::new(heading)
                    .text_style(TextStyle::Heading)
                    .strong()
                    .color(t.text),
            );
            ui.add_space(t.space_2);
            ui.label(RichText::new(body).size(14.0).color(t.text_2));
            ui.add_space(t.space_5);
            if icon_text_button(ui, t, ButtonKind::Primary, button_icon, button_label, 38.0)
                .clicked()
            {
                clicked = true;
            }
        });
    });
    clicked
}

// ---------------------------------------------------------------------------
// app_header
// ---------------------------------------------------------------------------

/// Actions emitted by [`app_header`] in a single frame.
#[derive(Clone, Debug, Default)]
pub struct AppHeaderActions {
    /// User clicked the back chevron.
    pub back: bool,
    /// User clicked the settings gear.
    pub settings: bool,
    /// User committed an inline rename (Enter on a non-empty trimmed string).
    /// The new name is already written into the `project_name` buffer the
    /// caller passed in; this is just the signal to persist.
    pub renamed: bool,
}

/// The top studio header: back chevron · brand · `|` · project name · gear.
///
/// The project name is **inline-editable**: pass `&mut project_name` and a
/// `&mut is_editing` flag. Click the name to enter edit mode, Enter to commit
/// (returns `renamed: true`), Esc to cancel (caller restores the previous
/// name from its own copy if needed).
/// `status` optionally renders a shared chip beside the name; the boolean
/// selects its active/accent treatment.
///
/// The brand block paints the Tokito mark (via [`crate::brand_mark`]); the
/// mark itself carries the wordmark, so no system-font label is rendered.
pub fn app_header(
    ui: &mut Ui,
    t: &Tokens,
    project_name: &mut String,
    is_editing: &mut bool,
    status: Option<(&str, bool)>,
) -> AppHeaderActions {
    let mut actions = AppHeaderActions::default();
    let height = 52.0;

    egui::Frame::new()
        .fill(t.bg_chrome)
        .inner_margin(egui::Margin::symmetric((t.space_3) as i8, (0.0) as i8))
        .show(ui, |ui| {
            ui.set_height(height);
            ui.horizontal_centered(|ui| {
                if icon_button(ui, t, icons::ph::CARET_LEFT, 32.0, t.text_2).clicked() {
                    actions.back = true;
                }
                ui.add_space(t.space_2);

                // Brand block: the "App icon" lockup (dark tile + mark).
                // The mark itself carries the wordmark in the design, so
                // no system-font label is rendered alongside.
                crate::brand::brand_tile(ui, 34.0);

                // Divider `|`.
                ui.add_space(t.space_3);
                let (sep_rect, _) =
                    ui.allocate_exact_size(vec2(1.0, height - 24.0), Sense::hover());
                ui.painter().line_segment(
                    [sep_rect.center_top(), sep_rect.center_bottom()],
                    Stroke::new(1.0, t.border),
                );
                ui.add_space(t.space_3);

                // Project name — display or inline-edit.
                if *is_editing {
                    let resp = ui.add(
                        egui::TextEdit::singleline(project_name)
                            .desired_width(260.0)
                            .margin(egui::Margin::symmetric((8.0) as i8, (4.0) as i8)),
                    );
                    if resp.lost_focus() {
                        *is_editing = false;
                        if ui.input(|i| i.key_pressed(egui::Key::Enter))
                            && !project_name.trim().is_empty()
                        {
                            actions.renamed = true;
                        }
                    } else if !resp.has_focus() {
                        resp.request_focus();
                    }
                } else {
                    let label = RichText::new(project_name.as_str()).color(t.text_2);
                    let resp = ui.add(egui::Label::new(label).sense(Sense::click()));
                    if resp.clicked() {
                        *is_editing = true;
                    }
                }

                if let Some((label, active)) = status {
                    ui.add_space(t.space_3);
                    let _ = chip(ui, t, label, active);
                }

                // Right side: settings gear.
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_button(ui, t, icons::ph::GEAR_SIX, 32.0, t.text_2).clicked() {
                        actions.settings = true;
                    }
                });
            });
        });

    actions
}

// ---------------------------------------------------------------------------
// tab_bar
// ---------------------------------------------------------------------------

/// One entry in a [`tab_bar`]: either a tab with an icon + label, or a
/// vertical divider for visual grouping.
#[derive(Clone, Copy, Debug)]
pub enum TabItem<'a> {
    Tab {
        icon: &'a str,
        label: &'a str,
    },
    /// A thin vertical separator. Visually groups tabs (e.g.
    /// `Chat · Plan · Artifacts | Schematic · PCB · BOM · …`).
    Divider,
}

/// Flat top-of-page tab strip.
///
/// `items` is the ordered slice of tabs / dividers. `selected` is the **index
/// into `items`** of the active tab (dividers count as positions for
/// counting purposes; they're never selected).
///
/// Returns the clicked tab's index in `items`, or `None` if no tab was
/// clicked this frame.
///
/// Visual: selected tab is a filled accent pill; unselected tabs render an
/// icon + label in muted ink with a subtle hover wash.
pub fn tab_bar(ui: &mut Ui, t: &Tokens, items: &[TabItem<'_>], selected: usize) -> Option<usize> {
    let mut clicked = None;
    egui::Frame::new()
        .fill(t.bg_chrome)
        .stroke(Stroke::new(1.0, t.border_soft))
        .inner_margin(egui::Margin::symmetric(
            (t.space_3) as i8,
            (t.space_1) as i8,
        ))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = t.space_1;
                for (i, item) in items.iter().enumerate() {
                    match item {
                        TabItem::Tab { icon, label } => {
                            if tab_pill(ui, t, icon, label, selected == i) {
                                clicked = Some(i);
                            }
                        }
                        TabItem::Divider => {
                            ui.add_space(t.space_2);
                            let (rect, _) = ui.allocate_exact_size(vec2(1.0, 18.0), Sense::hover());
                            ui.painter().line_segment(
                                [rect.center_top(), rect.center_bottom()],
                                Stroke::new(1.0, t.border),
                            );
                            ui.add_space(t.space_2);
                        }
                    }
                }
            });
        });
    clicked
}

fn tab_pill(ui: &mut Ui, t: &Tokens, icon: &str, label: &str, selected: bool) -> bool {
    let h = 30.0;
    // Pre-measure label width so the pill grows naturally.
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        TextStyle::Body.resolve(ui.style()),
        if selected { t.accent_ink } else { t.text_2 },
    );
    let pad = 12.0;
    let icon_w = 18.0;
    let gap = 6.0;
    let w = pad + icon_w + gap + galley.size().x + pad;

    let (rect, response) = ui.allocate_exact_size(vec2(w, h), Sense::click());
    let hv = hover_t(ui, response.id, response.hovered());

    // Active tab reads as a *defined* tinted pill (soft accent wash + 1px
    // accent border + accent ink), not a solid-mint slab. Keeps mint as a
    // hierarchy signal instead of flooding it across the chrome — the same
    // "soft tint + border" language the active thread row uses.
    let fill = if selected {
        t.accent_soft
    } else {
        lerp_color(Color32::TRANSPARENT, t.card_hover, hv)
    };
    let ink = if selected {
        t.accent
    } else {
        lerp_color(t.text_2, t.text, hv)
    };

    ui.painter().rect_filled(rect, t.rounding_sm(), fill);
    if selected {
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            t.rounding_sm(),
            Stroke::new(1.0, t.accent),
            egui::StrokeKind::Outside,
        );
    }

    let mut x = rect.left() + pad;
    let center_y = rect.center().y;
    ui.painter().text(
        pos2(x + icon_w * 0.5, center_y),
        egui::Align2::CENTER_CENTER,
        icon,
        icons::font(15.0),
        ink,
    );
    x += icon_w + gap;
    ui.painter().text(
        pos2(x, center_y),
        egui::Align2::LEFT_CENTER,
        label,
        TextStyle::Body.resolve(ui.style()),
        ink,
    );

    response.clicked()
}

// ---------------------------------------------------------------------------
// chat surface — avatar, bubble, composer
// ---------------------------------------------------------------------------

/// The party a [`chat_bubble`] or [`chat_avatar`] belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BubbleKind {
    Assistant,
    User,
}

/// Avatar disc shown next to a [`chat_bubble`]. The assistant variant paints
/// a sparkle glyph; the user variant paints up to two initial letters.
pub fn chat_avatar(ui: &mut Ui, t: &Tokens, kind: BubbleKind, initials: &str) -> Response {
    let side = 28.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::hover());
    let (fill, ink, glyph_size) = match kind {
        BubbleKind::Assistant => (t.chat_avatar_bg, t.accent, 14.0),
        BubbleKind::User => (t.chat_avatar_bg_user, t.text, 12.0),
    };
    ui.painter().circle_filled(rect.center(), side * 0.5, fill);
    match kind {
        BubbleKind::Assistant => {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                icons::ph::SPARKLE,
                icons::font(glyph_size),
                ink,
            );
        }
        BubbleKind::User => {
            let label = initials.chars().take(2).collect::<String>().to_uppercase();
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &label,
                TextStyle::Small.resolve(ui.style()),
                ink,
            );
        }
    }
    response
}

/// A standalone Phosphor glyph centered in a tinted disc — the hero/greeting
/// avatar used by empty states (e.g. the chat "What do you want to build
/// today?" surface). Unlike [`chat_avatar`] this is not tied to a
/// [`BubbleKind`]: the caller picks the `glyph` and `diameter`, and the disc
/// uses the chat-avatar fill with the brand `accent` ink. The glyph scales to
/// ~46 % of the disc so larger avatars stay visually balanced.
pub fn icon_avatar(ui: &mut Ui, t: &Tokens, glyph: &str, diameter: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(diameter), Sense::hover());
    ui.painter()
        .circle_filled(rect.center(), diameter * 0.5, t.chat_avatar_bg);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        icons::font(diameter * 0.46),
        t.accent,
    );
    response
}

/// A single chat message bubble.
///
/// The bubble is left-aligned with the assistant avatar for `Assistant`, and
/// right-aligned (avatar on the right) for `User`. `body` runs inside the
/// bubble's content rect — it can render plain text, markdown later, or
/// stacked widgets (tool-call cards, mutation cards). The bubble grows to
/// fit; the caller controls the column width via the surrounding `Ui`.
///
/// `initials` is used by the user-kind avatar; pass an empty string for the
/// assistant kind.
pub fn chat_bubble(
    ui: &mut Ui,
    t: &Tokens,
    kind: BubbleKind,
    initials: &str,
    body: impl FnOnce(&mut Ui),
) {
    let layout = match kind {
        BubbleKind::Assistant => Layout::left_to_right(Align::Min),
        BubbleKind::User => Layout::right_to_left(Align::Min),
    };
    ui.with_layout(layout, |ui| {
        chat_avatar(ui, t, kind, initials);
        ui.add_space(t.space_2);
        let fill = match kind {
            BubbleKind::Assistant => t.chat_bubble_bg,
            BubbleKind::User => t.chat_bubble_bg_user,
        };
        egui::Frame::new()
            .fill(fill)
            .corner_radius(t.rounding_sm())
            .inner_margin(egui::Margin::symmetric((14.0) as i8, (12.0) as i8))
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width().min(640.0));
                body(ui);
            });
    });
}

/// Assistant activity bubble for slow background work.
///
/// This is the "premium but quiet" loading state for chat: normal assistant
/// bubble chrome, a compact status row, three breathing dots, and product-
/// facing detail copy. It deliberately avoids backend/provider wording; the
/// host supplies the `label` and `detail` copy.
///
/// `id_source` scopes child ids so multiple activity bubbles can coexist
/// without reflow making their animation state fight.
pub fn chat_activity(
    ui: &mut Ui,
    t: &Tokens,
    id_source: impl Hash + std::fmt::Debug,
    label: &str,
    detail: &str,
) {
    ui.ctx().request_repaint_after(Duration::from_millis(50));
    ui.push_id(id_source, |ui| {
        chat_bubble(ui, t, BubbleKind::Assistant, "", |ui| {
            let available = ui.available_width().max(0.0);
            let content_width = if available < 240.0 {
                available
            } else {
                available.min(420.0)
            };
            let time = ui.ctx().input(|i| i.time) as f32;
            ui.allocate_ui_with_layout(
                vec2(content_width, 0.0),
                Layout::top_down(Align::Min).with_cross_justify(true),
                |ui| {
                    ui.horizontal(|ui| {
                        badge(ui, t, label);
                        ui.add_space(t.space_1);
                        activity_dots(ui, t, time);
                    });

                    if !detail.trim().is_empty() {
                        ui.add_space(t.space_2);
                        ui.add(
                            egui::Label::new(RichText::new(detail).italics().color(t.text_2))
                                .wrap(),
                        );
                    }
                },
            );
        });
    });
}

fn activity_dots(ui: &mut Ui, t: &Tokens, time: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(34.0, 14.0), Sense::hover());
    let painter = ui.painter();
    for i in 0..3 {
        let x = rect.left() + 7.0 + i as f32 * 10.0;
        let wave = ((time * 2.8 + i as f32 * 0.72).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        let radius = 2.0 + wave * 1.2;
        let ink = lerp_color(t.text_3, t.accent, 0.35 + wave * 0.55);
        painter.circle_filled(pos2(x, rect.center().y), radius, ink.gamma_multiply(0.75));
    }
    response
}

/// State for [`chat_composer`].
#[derive(Clone, Debug, Default)]
pub struct ChatComposerState {
    /// The current draft text.
    pub text: String,
    /// `true` while the assistant is streaming — flips the send glyph to a
    /// Stop affordance and disables submit on Enter.
    pub streaming: bool,
}

/// Composer actions emitted in a single frame.
#[derive(Clone, Debug)]
pub enum ComposerAction {
    /// User submitted the draft. The state's `text` has already been cleared.
    Submit(String),
    /// User pressed the Stop affordance while streaming.
    Stop,
}

/// Multi-line chat composer with a Send / Stop affordance.
///
/// Enter submits; Shift+Enter inserts a newline. Submitting clears the text
/// buffer and returns the submitted string in `ComposerAction::Submit`. While
/// `state.streaming` is `true`, the trailing button paints as a Stop glyph
/// and Enter no longer submits — clicking emits `ComposerAction::Stop`.
pub fn chat_composer(
    ui: &mut Ui,
    t: &Tokens,
    state: &mut ChatComposerState,
    hint: &str,
) -> Option<ComposerAction> {
    let mut action = None;

    egui::Frame::new()
        .fill(t.card)
        .stroke(Stroke::new(1.0, t.border))
        .corner_radius(t.rounding_md())
        .inner_margin(egui::Margin::symmetric(
            (t.space_3) as i8,
            (t.space_2) as i8,
        ))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let send_side = 36.0;
                let composer_w = (ui.available_width() - send_side - t.space_2).max(0.0);
                let resp = ui.add_sized(
                    [composer_w, 0.0],
                    egui::TextEdit::multiline(&mut state.text)
                        .frame(egui::Frame::NONE)
                        .desired_rows(1)
                        .hint_text(hint),
                );

                // Enter to submit (Shift+Enter falls through to TextEdit and
                // inserts a newline). Only when not streaming.
                let pressed_enter =
                    ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);
                if resp.has_focus()
                    && pressed_enter
                    && !state.streaming
                    && !state.text.trim().is_empty()
                {
                    let submitted = std::mem::take(&mut state.text);
                    action = Some(ComposerAction::Submit(submitted.trim().to_string()));
                }

                ui.add_space(t.space_2);
                // Send / Stop button.
                let (glyph, enabled) = if state.streaming {
                    (icons::ph::STOP, true)
                } else {
                    (icons::ph::PAPER_PLANE_RIGHT, !state.text.trim().is_empty())
                };
                if send_button(ui, t, glyph, send_side, enabled).clicked() {
                    if state.streaming {
                        action = Some(ComposerAction::Stop);
                    } else if !state.text.trim().is_empty() {
                        let submitted = std::mem::take(&mut state.text);
                        action = Some(ComposerAction::Submit(submitted.trim().to_string()));
                    }
                }
            });
        });

    action
}

fn send_button(ui: &mut Ui, t: &Tokens, glyph: &str, side: f32, enabled: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
    let hv = if enabled {
        hover_t(ui, response.id, response.hovered())
    } else {
        0.0
    };
    let fill = if enabled {
        lerp_color(t.accent, t.accent.gamma_multiply(1.15), hv)
    } else {
        t.card_hover
    };
    let ink = if enabled {
        t.accent_ink
    } else {
        t.text_disabled
    };
    ui.painter().rect_filled(rect, t.rounding_sm(), fill);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        icons::font(16.0),
        ink,
    );
    response
}

// ---------------------------------------------------------------------------
// floating_help_button
// ---------------------------------------------------------------------------

/// The bottom-right circular `?` affordance that opens a help overlay.
///
/// Render this in an [`egui::Area`] anchored to the bottom-right of the
/// surface — the primitive itself just paints the button.
pub fn floating_help_button(ui: &mut Ui, t: &Tokens) -> Response {
    let side = 32.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
    let hv = hover_t(ui, response.id, response.hovered());
    let fill = lerp_color(t.card, t.card_hover, hv);
    let border = lerp_color(t.border, t.border_strong, hv);
    ui.painter().circle_filled(rect.center(), side * 0.5, fill);
    ui.painter()
        .circle_stroke(rect.center(), side * 0.5 - 0.5, Stroke::new(1.0, border));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "?",
        TextStyle::Body.resolve(ui.style()),
        t.text_2,
    );
    response
}

// ---------------------------------------------------------------------------
// ai_helper_rail
// ---------------------------------------------------------------------------

/// Visibility / size state of the [`ai_helper_rail`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiHelperRailState {
    /// Not rendered at all.
    Hidden,
    /// Thin (~44 px) icon-only rail with a single sparkle button to expand.
    CollapsedGlyph,
    /// Full ~280 px rail with suggestions and a quick composer.
    Expanded,
}

/// Action emitted by the [`ai_helper_rail`] in a single frame.
#[derive(Clone, Debug)]
pub enum AiHelperRailAction {
    /// The user clicked a suggestion chip; index into the slice passed in.
    SuggestionClicked(usize),
    /// The rail's quick composer submitted.
    Submit(String),
    /// The rail's quick composer Stop button pressed.
    Stop,
    /// The user hit the `×` to fully hide the rail.
    Close,
    /// The user hit the collapse button → caller should set state to
    /// `CollapsedGlyph`.
    Collapse,
    /// The user clicked the collapsed glyph → caller should set state to
    /// `Expanded`.
    Expand,
}

/// AI Helper rail — a slim right-side surface present on every studio tab.
///
/// Three visual states:
///
/// - [`AiHelperRailState::Hidden`] — the primitive returns immediately
///   without drawing.
/// - [`AiHelperRailState::CollapsedGlyph`] — paints a single ~44 px column
///   with a sparkle icon button; clicking emits `Expand`.
/// - [`AiHelperRailState::Expanded`] — paints the full rail: header with
///   close + collapse, a stack of suggestion chips, and a quick composer.
///
/// The caller is expected to host this in an [`egui::Panel::right`] (or
/// equivalent) and adjust the panel's `exact_size` to match the current
/// state.
pub fn ai_helper_rail(
    ui: &mut Ui,
    t: &Tokens,
    state: AiHelperRailState,
    suggestions: &[&str],
    composer: &mut ChatComposerState,
) -> Option<AiHelperRailAction> {
    match state {
        AiHelperRailState::Hidden => None,
        AiHelperRailState::CollapsedGlyph => collapsed_glyph_rail(ui, t),
        AiHelperRailState::Expanded => expanded_rail(ui, t, suggestions, composer),
    }
}

fn collapsed_glyph_rail(ui: &mut Ui, t: &Tokens) -> Option<AiHelperRailAction> {
    let mut out = None;
    egui::Frame::new()
        .fill(t.bg_chrome)
        .inner_margin(egui::Margin::symmetric((6.0) as i8, (t.space_3) as i8))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                if icon_button(ui, t, icons::ph::SPARKLE, 32.0, t.accent).clicked() {
                    out = Some(AiHelperRailAction::Expand);
                }
            });
        });
    out
}

fn expanded_rail(
    ui: &mut Ui,
    t: &Tokens,
    suggestions: &[&str],
    composer: &mut ChatComposerState,
) -> Option<AiHelperRailAction> {
    let mut out = None;
    egui::Frame::new()
        .fill(t.bg_chrome)
        .stroke(Stroke::new(1.0, t.border_soft))
        .inner_margin(egui::Margin::same((t.space_3) as i8))
        .show(ui, |ui| {
            // Header — title + collapse + close.
            ui.horizontal(|ui| {
                ui.label(icons::icon_text(
                    icons::ph::SPARKLE,
                    14.0,
                    "AI Helper",
                    13.0,
                    t.text,
                ));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_button(ui, t, icons::ph::X, 24.0, t.text_2).clicked() {
                        out = Some(AiHelperRailAction::Close);
                    }
                    if icon_button(ui, t, icons::ph::CARET_DOUBLE_RIGHT, 24.0, t.text_2).clicked() {
                        out = Some(AiHelperRailAction::Collapse);
                    }
                });
            });
            ui.add_space(t.space_2);
            ui.label(
                RichText::new("Ask Tokito to modify your design instead of hunting through panels")
                    .size(12.0)
                    .color(t.text_3),
            );
            ui.add_space(t.space_3);

            // Suggestion chips — stacked vertically.
            for (i, s) in suggestions.iter().enumerate() {
                if rail_suggestion(ui, t, s).clicked() {
                    out = Some(AiHelperRailAction::SuggestionClicked(i));
                }
                ui.add_space(t.space_1);
            }

            // Composer at bottom — push to bottom of available space.
            ui.with_layout(
                Layout::bottom_up(Align::Min).with_cross_justify(true),
                |ui| {
                    if let Some(action) =
                        chat_composer(ui, t, composer, "Ask Tokito to change something…")
                    {
                        out = Some(match action {
                            ComposerAction::Submit(s) => AiHelperRailAction::Submit(s),
                            ComposerAction::Stop => AiHelperRailAction::Stop,
                        });
                    }
                },
            );
        });
    out
}

fn rail_suggestion(ui: &mut Ui, t: &Tokens, label: &str) -> Response {
    let h = 38.0;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::click());
    let hv = hover_t(ui, response.id, response.hovered());
    let fill = lerp_color(t.card, t.card_hover, hv);
    let border = lerp_color(t.border, t.border_strong, hv);
    ui.painter().rect_filled(rect, t.rounding_sm(), fill);
    ui.painter().rect_stroke(
        rect.shrink(0.5),
        t.rounding_sm(),
        Stroke::new(1.0, border),
        egui::StrokeKind::Outside,
    );
    ui.painter().text(
        pos2(rect.left() + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        TextStyle::Body.resolve(ui.style()),
        t.text,
    );
    response
}

// ---------------------------------------------------------------------------
// thread_row + conversation_sidebar
// ---------------------------------------------------------------------------

/// One row in the [`conversation_sidebar`] list.
///
/// Two-line layout: title (bold when `selected`) + relative time on top,
/// muted preview snippet underneath. Set `workshop = true` to paint a globe
/// glyph beside the title for the cross-design Workshop thread.
pub fn thread_row(
    ui: &mut Ui,
    t: &Tokens,
    title: &str,
    preview: &str,
    time: &str,
    selected: bool,
    workshop: bool,
) -> Response {
    let h = 56.0;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::click());
    let hv = hover_t(ui, response.id, response.hovered());
    let fill = if selected {
        t.accent_soft
    } else {
        lerp_color(Color32::TRANSPARENT, t.card_hover, hv)
    };
    ui.painter().rect_filled(rect, t.rounding_sm(), fill);
    // Active row: 1px accent border turns the soft wash into a defined card
    // instead of a muddy slab (matches the active tab pill).
    if selected {
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            t.rounding_sm(),
            Stroke::new(1.0, t.accent),
            egui::StrokeKind::Outside,
        );
    }

    let pad = t.space_2;
    let inner = rect.shrink2(vec2(pad + 4.0, pad));
    let painter = ui.painter().with_clip_rect(inner);
    let mut top = inner.left_top();
    let title_y = top.y + 8.0;
    let preview_y = top.y + 28.0;

    if workshop {
        painter.text(
            pos2(top.x, title_y),
            egui::Align2::LEFT_CENTER,
            icons::ph::GLOBE,
            icons::font(13.0),
            if selected { t.accent } else { t.text_2 },
        );
        top.x += 18.0;
    }
    // Selected vs unselected uses the row's wash (`accent_soft`) as the
    // signal; title text stays the same ink either way for legibility.
    let time_font = TextStyle::Small.resolve(ui.style());
    let time_width = if time.is_empty() {
        0.0
    } else {
        ui.painter()
            .layout_no_wrap(time.to_owned(), time_font.clone(), Color32::WHITE)
            .size()
            .x
            + t.space_2
    };
    let title_width = (inner.right() - time_width - top.x).max(0.0);
    painter.text(
        pos2(top.x, title_y),
        egui::Align2::LEFT_CENTER,
        truncate_for(ui, title, TextStyle::Body, title_width),
        TextStyle::Body.resolve(ui.style()),
        t.text,
    );

    // Time right-aligned on the top row.
    painter.text(
        pos2(inner.right(), title_y),
        egui::Align2::RIGHT_CENTER,
        time,
        time_font,
        t.text_3,
    );

    // Preview -- single-line, hard truncate.
    painter.text(
        pos2(inner.left(), preview_y),
        egui::Align2::LEFT_CENTER,
        truncate_for(ui, preview, TextStyle::Small, inner.width()),
        TextStyle::Small.resolve(ui.style()),
        if selected { t.text_2 } else { t.text_3 },
    );

    response
}

fn truncate_for(ui: &Ui, s: &str, text_style: TextStyle, max_w: f32) -> String {
    if max_w <= 0.0 {
        return String::new();
    }
    let one_line = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let s = one_line.trim();
    if s.is_empty() {
        return String::new();
    }
    let style = text_style.resolve(ui.style());
    let galley = ui
        .painter()
        .layout_no_wrap(s.to_string(), style.clone(), Color32::WHITE);
    if galley.size().x <= max_w {
        return s.to_string();
    }
    let mut buf = String::new();
    for ch in s.chars() {
        buf.push(ch);
        let g = ui
            .painter()
            .layout_no_wrap(format!("{buf}…"), style.clone(), Color32::WHITE);
        if g.size().x > max_w {
            buf.pop();
            buf.push('…');
            return buf;
        }
    }
    buf
}

/// Actions emitted by the [`conversation_sidebar`] header / chrome (not the
/// individual rows — those return their own `Response`).
#[derive(Clone, Debug)]
pub enum SidebarAction {
    /// User hit "+ New conversation".
    NewConversation,
    /// User collapsed the sidebar.
    Collapse,
}

/// The Chat-tab conversation sidebar.
///
/// Chrome only — the caller paints the rows inside `body` by calling
/// [`thread_row`]. The expected layout is: design-scoped threads, then
/// [`sidebar_divider`], then the pinned Workshop row.
pub fn conversation_sidebar(
    ui: &mut Ui,
    t: &Tokens,
    body: impl FnOnce(&mut Ui),
) -> Option<SidebarAction> {
    let mut out = None;
    egui::Frame::new()
        .fill(t.bg_chrome)
        .stroke(Stroke::new(1.0, t.border_soft))
        .inner_margin(egui::Margin::same((t.space_2) as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Conversations")
                        .small()
                        .strong()
                        .color(t.text_2),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = t.space_1;
                    if icon_button(ui, t, icons::ph::CARET_DOUBLE_LEFT, 26.0, t.text_2)
                        .on_hover_text("Collapse sidebar")
                        .clicked()
                    {
                        out = Some(SidebarAction::Collapse);
                    }
                    if icon_button(ui, t, icons::ph::PLUS, 26.0, t.text_2)
                        .on_hover_text("New conversation")
                        .clicked()
                    {
                        out = Some(SidebarAction::NewConversation);
                    }
                });
            });
            // Ground the title + buttons as a header band with a hairline,
            // so the controls stop reading as floating glyphs.
            ui.add_space(t.space_1);
            ui.separator();
            ui.add_space(t.space_2);

            egui::ScrollArea::vertical()
                .id_salt("sidebar_threads")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    body(ui);
                });
        });
    out
}

/// Hairline divider intended for use inside the [`conversation_sidebar`]
/// body, between the design-scoped threads and the pinned Workshop row.
pub fn sidebar_divider(ui: &mut Ui, t: &Tokens) {
    ui.add_space(t.space_2);
    ui.separator();
    ui.add_space(t.space_1);
}

// ---------------------------------------------------------------------------
// suggestion_card
// ---------------------------------------------------------------------------

/// Lifecycle of a [`suggestion_card`] — drives status badge styling + which
/// footer buttons are enabled.
///
/// The caller owns this state and updates it in response to user interaction
/// (Apply / Discard) and to drift events from the document layer (stale-on-
/// apply rejection).
#[derive(Clone, Debug)]
pub enum SuggestionCardStatus {
    /// User has not acted yet; Apply / Discard buttons are live.
    Pending,
    /// Every op in the batch applied successfully.
    AppliedAll,
    /// Some ops applied, others were unchecked before apply.
    AppliedPartial { applied: usize, total: usize },
    /// User dismissed the proposal without applying any op.
    Discarded,
    /// Apply was attempted but the underlying document had drifted since
    /// the batch was proposed (see `docs/CHAT_SCHEMATIC_CONTROL.md`
    /// §Snapshot + staleness). The card is now read-only.
    StaleRejected,
}

/// Per-row state for [`suggestion_card`]. The caller initialises one entry
/// per op (typically `vec![true; ops.len()]`) and the card mutates the
/// selected flag in place as the user toggles per-op checkboxes.
#[derive(Clone, Debug, Default)]
pub struct SuggestionCardState {
    /// One entry per op in the batch. `true` means the op will be applied
    /// when the user clicks "Apply selected".
    pub op_selected: Vec<bool>,
    /// Last-hovered op index — reported back to the caller via
    /// [`SuggestionCardAction::HoverOp`] so the host can mirror the hover
    /// state into a canvas highlight without a second pass.
    pub last_hover: Option<usize>,
}

impl SuggestionCardState {
    /// Default state for a batch of `n` ops: all selected, no hover.
    pub fn all_selected(n: usize) -> Self {
        Self {
            op_selected: vec![true; n],
            last_hover: None,
        }
    }
}

/// One op row inside a [`suggestion_card`].
///
/// `summary` is the human-readable description (e.g. `"Add U3 (LM2596) at
/// (120, 140)"`) — usually `SchematicEditOp::summary()` upstream.
/// `provenance_label` is the short chip text (e.g. `"AI model"`, `"BOM"`,
/// `"ERC fix"`) — typically derived from `EditProvenance` via the host's
/// own label function.
pub struct SuggestionOpRow<'a> {
    pub summary: &'a str,
    pub provenance_label: &'a str,
}

/// Events emitted by [`suggestion_card`] in a single frame. The card never
/// mutates host state beyond the per-row selection flags; everything else
/// is reported back via this enum so the host can decide what to do.
#[derive(Clone, Debug)]
pub enum SuggestionCardAction {
    /// Hover landed on a different op row (or left the card). The host
    /// uses this to highlight the affected element on the canvas — see
    /// `docs/CHAT_SCHEMATIC_CONTROL.md` §Ghost preview on canvas — deferred.
    HoverOp(Option<usize>),
    /// "Apply selected" pressed. The host reads `state.op_selected` for
    /// the accepted subset.
    ApplySelected,
    /// "Discard all" pressed.
    DiscardAll,
}

/// The "AI proposes, you approve" surface — one bordered card showing a
/// proposed batch of typed edit operations with per-op accept/reject
/// checkboxes and a footer of bulk actions.
///
/// Rendered inline in chat (when the editor agent emits a Suggestion event)
/// and also in the Build review panel. The card is the single primitive
/// every AI mutation surface in Tokito routes through — keeping the apply
/// gate consistent with the spec.
///
/// **Status semantics.** Apply / Discard buttons are enabled only while
/// `status` is [`SuggestionCardStatus::Pending`]. Other states render as
/// a read-only badge ("Applied", "Applied 2 of 3", "Discarded", "Stale —
/// schematic changed; re-ask").
///
/// **Per-row vs bulk.** The footer's "Apply selected" applies only the ops
/// whose `state.op_selected[i]` is `true`. The Discard button always
/// discards the whole batch regardless of per-row state.
///
/// **Hover → canvas highlight.** When the user hovers a row, the function
/// returns [`SuggestionCardAction::HoverOp`] with `Some(i)`; when hover
/// leaves every row, the variant carries `None`. Host surfaces use this
/// to flash the affected refdes / net on the canvas (a cheap V1
/// substitute for full ghost-preview rendering, which is deferred per
/// `docs/AI_ROADMAP.md` §B in the tokito repo).
pub fn suggestion_card(
    ui: &mut Ui,
    t: &Tokens,
    header: &str,
    status: SuggestionCardStatus,
    ops: &[SuggestionOpRow<'_>],
    state: &mut SuggestionCardState,
) -> Option<SuggestionCardAction> {
    // Defensive: keep per-row state in sync with the ops slice the caller
    // passed this frame. Callers that mutate the batch shouldn't have to
    // reach into op_selected manually.
    if state.op_selected.len() != ops.len() {
        state.op_selected.resize(ops.len(), true);
    }

    let mut action: Option<SuggestionCardAction> = None;
    let interactive = matches!(status, SuggestionCardStatus::Pending);
    let mut hover_this_frame: Option<usize> = None;

    egui::Frame::new()
        .fill(t.card)
        .corner_radius(t.rounding_md())
        .inner_margin(egui::Margin::same((t.space_4) as i8))
        .stroke(Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            // Header row: title + status badge on the right.
            ui.horizontal(|ui| {
                ui.label(RichText::new(header).strong().color(t.text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    suggestion_status_badge(ui, t, &status);
                });
            });
            ui.add_space(t.space_2);

            // Per-row chips only earn their place when they DIFFER across rows
            // (e.g. a partial apply: some "Applied", some "Skipped"). When every
            // row carries the same label it just parrots the header status badge
            // in a noisy, far-floating column — drop it entirely.
            let distinct_labels: std::collections::HashSet<&str> = ops
                .iter()
                .map(|o| o.provenance_label)
                .filter(|s| !s.is_empty())
                .collect();
            let show_row_badges = distinct_labels.len() > 1;

            // Op rows.
            for (i, op) in ops.iter().enumerate() {
                let row_resp = render_op_row(
                    ui,
                    t,
                    i,
                    op,
                    &mut state.op_selected[i],
                    interactive,
                    show_row_badges,
                );
                if row_resp.hovered() {
                    hover_this_frame = Some(i);
                }
            }

            if interactive && !ops.is_empty() {
                ui.add_space(t.space_2);
                ui.separator();
                ui.add_space(t.space_2);
            }

            // Footer: Apply selected / Discard all.
            if interactive {
                ui.horizontal(|ui| {
                    let applied_count = state.op_selected.iter().filter(|b| **b).count();
                    let apply_enabled = applied_count > 0;

                    let apply_label = if applied_count == ops.len() || applied_count == 0 {
                        "Apply selected".to_string()
                    } else {
                        format!("Apply selected ({applied_count})")
                    };

                    let apply_kind = if apply_enabled {
                        ButtonKind::Primary
                    } else {
                        ButtonKind::Secondary
                    };
                    let apply = text_button(ui, t, apply_kind, &apply_label, 32.0);
                    if apply.clicked() && apply_enabled {
                        action = Some(SuggestionCardAction::ApplySelected);
                    }

                    let discard = text_button(ui, t, ButtonKind::Secondary, "Discard all", 32.0);
                    if discard.clicked() {
                        action = Some(SuggestionCardAction::DiscardAll);
                    }
                });
            }
        });

    // Report hover change.
    if hover_this_frame != state.last_hover {
        state.last_hover = hover_this_frame;
        if action.is_none() {
            action = Some(SuggestionCardAction::HoverOp(hover_this_frame));
        }
    }

    action
}

/// One op row: checkbox + summary, and — only when `show_badge` is set — a
/// right-justified provenance/status chip. The chip uses the existing
/// [`badge`] primitive so it picks up the same border / fill / typography as
/// elsewhere. `show_badge` is false when every row shares the same label, so
/// the card doesn't repeat the header status on every line.
fn render_op_row(
    ui: &mut Ui,
    t: &Tokens,
    _index: usize,
    op: &SuggestionOpRow<'_>,
    selected: &mut bool,
    interactive: bool,
    show_badge: bool,
) -> Response {
    // Reserve a horizontal row, allocate the whole strip as a hoverable
    // sense rect so we can report hover even when the user moves between
    // the checkbox and the label.
    let row_height = 28.0;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), row_height), Sense::hover());

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.horizontal(|ui| {
            ui.add_space(2.0);
            // Per-row checkbox (egui's native — cheap; the bespoke
            // tokito_ui::checkbox is heavier-weight than needed here).
            let cb = ui.add_enabled(interactive, egui::Checkbox::new(selected, ""));
            ui.add_space(t.space_2);
            ui.label(RichText::new(op.summary).color(t.text));
            if show_badge && !op.provenance_label.is_empty() {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    badge(ui, t, op.provenance_label);
                });
            }
            let _ = cb;
        });
    });

    response
}

fn suggestion_status_badge(ui: &mut Ui, t: &Tokens, status: &SuggestionCardStatus) {
    let label = match status {
        SuggestionCardStatus::Pending => "Pending review",
        SuggestionCardStatus::AppliedAll => "Applied",
        SuggestionCardStatus::AppliedPartial { .. } => "Applied partial",
        SuggestionCardStatus::Discarded => "Discarded",
        SuggestionCardStatus::StaleRejected => "Stale — schematic changed",
    };
    let text = match status {
        SuggestionCardStatus::AppliedPartial { applied, total } => {
            format!("Applied {applied}/{total}")
        }
        _ => label.to_string(),
    };
    badge(ui, t, &text);
}
