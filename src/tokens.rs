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
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,

    /// Spacing scale.
    pub space_1: f32,
    pub space_2: f32,
    pub space_3: f32,
    pub space_4: f32,
    pub space_5: f32,
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
            radius_sm: 7.0,
            radius_md: 12.0,
            radius_lg: 16.0,
            space_1: 4.0,
            space_2: 8.0,
            space_3: 12.0,
            space_4: 16.0,
            space_5: 24.0,
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
            radius_sm: 7.0,
            radius_md: 12.0,
            radius_lg: 16.0,
            space_1: 4.0,
            space_2: 8.0,
            space_3: 12.0,
            space_4: 16.0,
            space_5: 24.0,
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
}
