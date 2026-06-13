//! [`Tokens`] — the flat design palette.
//!
//! One struct, two presets ([`Tokens::dark`] / [`Tokens::light`]). Every
//! component takes a `&Tokens`; there is no global theme singleton, so a
//! consumer can run two themes side by side if it ever needs to.
//!
//! To customise, start from a preset and assign fields:
//!
//! ```
//! let mut t = tokito_ui::Tokens::dark();
//! t.accent = egui::Color32::from_rgb(0xff, 0x6b, 0x35);
//! ```

use egui::Color32;

/// Colour + metrics palette. Cheap to copy.
///
/// `#[non_exhaustive]` — construct via [`Tokens::dark`] / [`Tokens::light`]
/// and assign fields; new fields can then be added without a breaking change.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Tokens {
    /// Whether this palette is a dark theme. Drives the egui base visuals.
    pub dark: bool,

    /// Window background — the surface everything sits on.
    pub bg: Color32,
    /// Chrome surfaces: top bar, menus, overlays.
    pub bg_chrome: Color32,
    /// Card / raised surface at rest.
    pub card: Color32,
    /// Card / raised surface when hovered.
    pub card_hover: Color32,

    /// Default 1 px border.
    pub border: Color32,
    /// Quieter border — section dividers, soft separators.
    pub border_soft: Color32,
    /// Emphasised border — hover, focus rings.
    pub border_strong: Color32,

    /// Primary text.
    pub text: Color32,
    /// Secondary text — subtitles, secondary labels.
    pub text_2: Color32,
    /// Muted text — metadata, timestamps, placeholders.
    pub text_3: Color32,
    /// Disabled text / disabled control foreground.
    pub text_disabled: Color32,

    /// Brand / primary accent.
    pub accent: Color32,
    /// Readable ink on top of [`Self::accent`] fills.
    pub accent_ink: Color32,
    /// Translucent accent wash — tinted backgrounds, soft highlights.
    pub accent_soft: Color32,

    /// Secondary accent — a distinct category colour (e.g. templates).
    pub accent_2: Color32,
    /// Translucent secondary-accent wash.
    pub accent_2_soft: Color32,

    /// Danger / destructive.
    pub danger: Color32,
    /// Warning / caution.
    pub warning: Color32,
    /// Success / positive.
    pub success: Color32,

    /// Corner radii.
    /// Extra-small — tight controls where `radius_sm` reads as a circle
    /// (checkbox boxes, small swatches).
    pub radius_xs: f32,
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,

    /// Spacing scale.
    pub space_1: f32,
    pub space_2: f32,
    pub space_3: f32,
    pub space_4: f32,
    pub space_5: f32,

    // -----------------------------------------------------------------------
    // Schematic palette
    // -----------------------------------------------------------------------
    //
    // These describe colours for a schematic / CAD-style canvas. They live
    // here (in the shared design layer) so a consumer's editor canvas pulls
    // from the same `Tokens` value that drives its chrome — one token source.
    //
    // A non-canvas consumer can ignore these fields; the presets below pick
    // reasonable defaults.
    /// Schematic sheet background.
    pub canvas_bg: Color32,
    /// Minor grid line — quiet, for the dense grid step.
    pub canvas_grid_minor: Color32,
    /// Major grid line — stronger, every Nth grid step.
    pub canvas_grid_major: Color32,
    /// Sheet frame border / title block strokes.
    pub canvas_frame: Color32,

    /// Symbol body ink — schematic outlines at rest.
    pub sym_ink: Color32,
    /// Symbol body fill — pale KiCad-style component body.
    pub sym_body_fill: Color32,
    /// Symbol body ink when hovered.
    pub sym_ink_hover: Color32,
    /// Symbol body ink when selected.
    pub sym_ink_selected: Color32,
    /// Subtle outline ring (anti-aliasing halo, soft separators).
    pub sym_outline: Color32,
    /// Selection ring around the symbol bounding box.
    pub sym_sel_ring: Color32,

    /// Wire — default ink.
    pub wire: Color32,
    /// Wire — highlighted (same net under cursor / search match).
    pub wire_highlight: Color32,
    /// Wire — selected.
    pub wire_selected: Color32,

    /// Net label ink.
    pub label_ink: Color32,
    /// Reference designator ink.
    pub refdes_ink: Color32,
    /// Pin name / number ink.
    pub pin_ink: Color32,
    /// Pin "hot" ink — active connection point, current drag.
    pub pin_hot: Color32,

    /// Marquee / multi-select fill.
    pub selection: Color32,
    /// Preview backdrop (place-tool ghost, drag preview).
    pub preview_bg: Color32,

    // -----------------------------------------------------------------------
    // Chat palette
    // -----------------------------------------------------------------------
    //
    // Surfaces for the chat / AI helper UI. Bubbles sit one step lighter than
    // [`Self::bg`]; avatars use a tinted disc behind a glyph or initials.
    /// Assistant chat bubble fill.
    pub chat_bubble_bg: Color32,
    /// User chat bubble fill.
    pub chat_bubble_bg_user: Color32,
    /// Assistant avatar disc — tinted with the brand accent.
    pub chat_avatar_bg: Color32,
    /// User avatar disc — neutral.
    pub chat_avatar_bg_user: Color32,
}

impl Tokens {
    /// Dark theme — the default. Matches the locked Tokito design.
    pub fn dark() -> Self {
        Self {
            dark: true,
            bg: Color32::from_rgb(0x0c, 0x0d, 0x10),
            bg_chrome: Color32::from_rgb(0x11, 0x12, 0x16),
            card: Color32::from_rgb(0x16, 0x18, 0x1d),
            card_hover: Color32::from_rgb(0x1c, 0x1f, 0x26),
            border: Color32::from_rgb(0x24, 0x26, 0x2d),
            border_soft: Color32::from_rgb(0x1d, 0x1f, 0x25),
            border_strong: Color32::from_rgb(0x36, 0x39, 0x43),
            text: Color32::from_rgb(0xec, 0xed, 0xf0),
            text_2: Color32::from_rgb(0x9a, 0x9d, 0xa7),
            text_3: Color32::from_rgb(0x5f, 0x62, 0x6d),
            text_disabled: Color32::from_rgb(0x44, 0x47, 0x4f),
            accent: Color32::from_rgb(0x2d, 0xd4, 0xbf),
            accent_ink: Color32::from_rgb(0x04, 0x24, 0x1f),
            accent_soft: Color32::from_rgba_unmultiplied(0x2d, 0xd4, 0xbf, 0x24),
            accent_2: Color32::from_rgb(0x6f, 0x73, 0xf0),
            accent_2_soft: Color32::from_rgba_unmultiplied(0x6f, 0x73, 0xf0, 0x29),
            danger: Color32::from_rgb(0xef, 0x5c, 0x68),
            warning: Color32::from_rgb(0xe0, 0xa4, 0x3f),
            success: Color32::from_rgb(0x3e, 0xcf, 0x8e),
            radius_xs: 4.0,
            radius_sm: 7.0,
            radius_md: 12.0,
            radius_lg: 16.0,
            space_1: 4.0,
            space_2: 8.0,
            space_3: 12.0,
            space_4: 16.0,
            space_5: 24.0,
            // Schematic palette — dark variant. Cool teal-ink schematic
            // against a deep slate sheet, warm orange selection ring.
            canvas_bg: Color32::from_rgb(0x14, 0x16, 0x1c),
            canvas_grid_minor: Color32::from_rgba_unmultiplied(0x60, 0x64, 0x70, 0x1c),
            canvas_grid_major: Color32::from_rgba_unmultiplied(0x7a, 0x80, 0x8c, 0x34),
            canvas_frame: Color32::from_rgb(0x4a, 0x4f, 0x5a),
            sym_ink: Color32::from_rgb(0xe6, 0xe8, 0xec),
            sym_body_fill: Color32::from_rgb(0x22, 0x24, 0x2a),
            sym_ink_hover: Color32::from_rgb(0x2d, 0xd4, 0xbf),
            sym_ink_selected: Color32::from_rgb(0xff, 0xff, 0xff),
            sym_outline: Color32::from_rgb(0x2a, 0x2c, 0x32),
            sym_sel_ring: Color32::from_rgb(0xe0, 0x78, 0x20),
            wire: Color32::from_rgb(0x9d, 0xc7, 0xff),
            wire_highlight: Color32::from_rgb(0x2d, 0xd4, 0xbf),
            wire_selected: Color32::from_rgb(0xe0, 0x78, 0x20),
            label_ink: Color32::from_rgb(0xb9, 0xc7, 0xdc),
            refdes_ink: Color32::from_rgb(0x9a, 0x9d, 0xa7),
            pin_ink: Color32::from_rgb(0xa8, 0xb0, 0xbe),
            pin_hot: Color32::from_rgb(0xe0, 0x78, 0x20),
            selection: Color32::from_rgba_unmultiplied(0xe0, 0x78, 0x20, 0x33),
            preview_bg: Color32::from_rgb(0x1a, 0x1c, 0x22),
            // Chat palette — dark. Bubble one step lighter than `bg`; assistant
            // avatar tinted with the accent, user avatar a muted neutral disc.
            chat_bubble_bg: Color32::from_rgb(0x15, 0x18, 0x1d),
            chat_bubble_bg_user: Color32::from_rgb(0x1c, 0x20, 0x27),
            chat_avatar_bg: Color32::from_rgb(0x12, 0x34, 0x30),
            chat_avatar_bg_user: Color32::from_rgb(0x24, 0x27, 0x2e),
        }
    }

    /// Light theme.
    pub fn light() -> Self {
        Self {
            dark: false,
            bg: Color32::from_rgb(0xf3, 0xf5, 0xf8),
            bg_chrome: Color32::from_rgb(0xff, 0xff, 0xff),
            card: Color32::from_rgb(0xff, 0xff, 0xff),
            card_hover: Color32::from_rgb(0xfb, 0xfc, 0xfe),
            border: Color32::from_rgb(0xe2, 0xe6, 0xec),
            border_soft: Color32::from_rgb(0xeb, 0xee, 0xf2),
            border_strong: Color32::from_rgb(0xc7, 0xcd, 0xd7),
            text: Color32::from_rgb(0x14, 0x17, 0x1d),
            text_2: Color32::from_rgb(0x57, 0x60, 0x6f),
            text_3: Color32::from_rgb(0x8b, 0x93, 0xa1),
            text_disabled: Color32::from_rgb(0xaa, 0xb2, 0xbd),
            accent: Color32::from_rgb(0x11, 0x96, 0x83),
            accent_ink: Color32::from_rgb(0xff, 0xff, 0xff),
            accent_soft: Color32::from_rgba_unmultiplied(0x11, 0x96, 0x83, 0x1f),
            accent_2: Color32::from_rgb(0x5b, 0x5f, 0xe0),
            accent_2_soft: Color32::from_rgba_unmultiplied(0x5b, 0x5f, 0xe0, 0x1f),
            danger: Color32::from_rgb(0xcf, 0x43, 0x4c),
            warning: Color32::from_rgb(0xb8, 0x7a, 0x18),
            success: Color32::from_rgb(0x1a, 0x9d, 0x6a),
            radius_xs: 4.0,
            radius_sm: 7.0,
            radius_md: 12.0,
            radius_lg: 16.0,
            space_1: 4.0,
            space_2: 8.0,
            space_3: 12.0,
            space_4: 16.0,
            space_5: 24.0,
            // Schematic palette — light variant. Schematic-ink (near-black)
            // on a soft slate sheet; same warm orange selection.
            canvas_bg: Color32::from_rgb(0xf4, 0xf7, 0xfa),
            canvas_grid_minor: Color32::from_rgba_unmultiplied(0x8c, 0x94, 0x9e, 0x1c),
            canvas_grid_major: Color32::from_rgba_unmultiplied(0x78, 0x80, 0x8c, 0x34),
            canvas_frame: Color32::from_rgb(0xa8, 0xae, 0xb8),
            sym_ink: Color32::from_rgb(0x1c, 0x20, 0x26),
            sym_body_fill: Color32::from_rgb(0xff, 0xfb, 0xde),
            sym_ink_hover: Color32::from_rgb(0x14, 0x34, 0x5c),
            sym_ink_selected: Color32::from_rgb(0x10, 0x14, 0x1a),
            sym_outline: Color32::from_rgb(0xfa, 0xfb, 0xfc),
            sym_sel_ring: Color32::from_rgb(0xe0, 0x78, 0x20),
            wire: Color32::from_rgb(0x30, 0x5e, 0x84),
            wire_highlight: Color32::from_rgb(0x14, 0x84, 0x76),
            wire_selected: Color32::from_rgb(0xe0, 0x78, 0x20),
            label_ink: Color32::from_rgb(0x28, 0x48, 0x6c),
            refdes_ink: Color32::from_rgb(0x30, 0x36, 0x3e),
            pin_ink: Color32::from_rgb(0x48, 0x58, 0x6c),
            pin_hot: Color32::from_rgb(0xe0, 0x78, 0x20),
            selection: Color32::from_rgba_unmultiplied(0xe0, 0x78, 0x20, 0x33),
            preview_bg: Color32::from_rgb(0xf4, 0xf5, 0xf7),
            // Chat palette — light. Bubble matches `card`; assistant avatar
            // a pale teal wash, user avatar a soft neutral.
            chat_bubble_bg: Color32::from_rgb(0xff, 0xff, 0xff),
            chat_bubble_bg_user: Color32::from_rgb(0xf0, 0xfa, 0xf7),
            chat_avatar_bg: Color32::from_rgb(0xd8, 0xef, 0xea),
            chat_avatar_bg_user: Color32::from_rgb(0xe6, 0xea, 0xef),
        }
    }

    /// Resolve a theme name: `"light"` → light, anything else → dark.
    pub fn from_name(name: &str) -> Self {
        if name.eq_ignore_ascii_case("light") {
            Self::light()
        } else {
            Self::dark()
        }
    }

    /// `radius_md` as an [`egui::Rounding`].
    pub fn rounding_md(&self) -> egui::Rounding {
        egui::Rounding::same(self.radius_md)
    }

    /// `radius_sm` as an [`egui::Rounding`].
    pub fn rounding_sm(&self) -> egui::Rounding {
        egui::Rounding::same(self.radius_sm)
    }

    /// `radius_xs` as an [`egui::Rounding`].
    pub fn rounding_xs(&self) -> egui::Rounding {
        egui::Rounding::same(self.radius_xs)
    }
}
