//! Visual style of the application: colors, spacing, fonts and the egui style.

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke,
    TextStyle, Vec2,
};

pub const BG: Color32 = Color32::from_gray(0x0F);
pub const CARD: Color32 = Color32::from_gray(0x18);
/// Background for rows and text fields, darker than the sections.
pub const INSET: Color32 = Color32::from_gray(0x12);
pub const BORDER: Color32 = Color32::from_gray(0x2B);
pub const BORDER_STRONG: Color32 = Color32::from_gray(0x3D);
pub const TEXT: Color32 = Color32::from_gray(0xEB);
pub const TEXT_WEAK: Color32 = Color32::from_gray(0x90);
/// Light green for icons, progress and highlighted text.
pub const ACCENT: Color32 = Color32::from_rgb(0x4A, 0xDE, 0x80);
/// Strong green for the primary button background.
pub const ACCENT_STRONG: Color32 = Color32::from_rgb(0x16, 0xA3, 0x4A);
/// Very dark green for backgrounds with a hint of color.
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0x12, 0x2E, 0x1F);
pub const ACCENT_EDGE: Color32 = Color32::from_rgb(0x1E, 0x4D, 0x33);
pub const CONTROL: Color32 = Color32::from_gray(0x23);

// Spacing scale, in points. All margins and gaps of the UI use these values.
/// Gap between lines of the same group (a title and its description, a name and its path).
pub const TIGHT: f32 = 2.0;
/// Gap between the items inside a box.
pub const SPACE: f32 = 8.0;
/// Gap between an icon and its text, and between a box header and its contents.
pub const GAP: f32 = 12.0;
/// Inner margin of boxes and panels, and gap between boxes.
pub const PAD: f32 = 16.0;
/// Side margin of the window.
pub const GUTTER: f32 = 24.0;

/// Height of buttons and text fields.
pub const CONTROL_HEIGHT: f32 = 32.0;
/// Height of the large buttons (primary action, add an item).
pub const LARGE_HEIGHT: f32 = 40.0;

pub fn margin(value: f32) -> Margin {
    Margin::same(value as i8)
}

/// Lines of the same group, separated by `TIGHT`.
pub fn lines<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = TIGHT;
        add_contents(ui)
    })
    .inner
}

/// Corner radius for sections and large buttons.
pub const RADIUS: u8 = 4;
/// Corner radius for small elements.
pub const RADIUS_SMALL: u8 = 3;

/// Semibold font family, for titles and buttons.
pub fn semibold() -> FontFamily {
    FontFamily::Name("semibold".into())
}

fn fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "inter".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../assets/fonts/Inter-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "inter-semibold".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../assets/fonts/Inter-SemiBold.ttf"
        ))),
    );
    // The default fonts remain as a fallback for characters that Inter does not include.
    let proportional = fonts.families.entry(FontFamily::Proportional).or_default();
    let mut semibold_fonts = vec!["inter-semibold".to_owned()];
    semibold_fonts.extend(proportional.iter().cloned());
    proportional.insert(0, "inter".into());
    fonts.families.insert(semibold(), semibold_fonts);
    fonts
}

pub fn apply(ctx: &egui::Context) {
    ctx.set_fonts(fonts());
    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(|style| {
        use FontFamily::Proportional;
        style.text_styles = [
            (TextStyle::Heading, FontId::new(20.0, semibold())),
            (TextStyle::Body, FontId::new(14.0, Proportional)),
            (TextStyle::Button, FontId::new(13.5, Proportional)),
            (TextStyle::Small, FontId::new(12.0, Proportional)),
            (
                TextStyle::Monospace,
                FontId::new(13.0, FontFamily::Monospace),
            ),
        ]
        .into();

        let s = &mut style.spacing;
        s.item_spacing = Vec2::splat(SPACE);
        s.button_padding = Vec2::new(GAP, 5.0);
        s.interact_size.y = CONTROL_HEIGHT;

        style.visuals = egui::Visuals::dark();
        let v = &mut style.visuals;
        v.panel_fill = BG;
        v.window_fill = CARD;
        v.extreme_bg_color = INSET;
        v.text_edit_bg_color = Some(INSET);
        v.faint_bg_color = BG;
        v.weak_text_color = Some(TEXT_WEAK);
        v.hyperlink_color = ACCENT;
        v.selection.bg_fill = Color32::from_rgb(0x16, 0x4A, 0x2C);
        v.selection.stroke = Stroke::new(1.5, ACCENT);
        v.text_cursor.stroke = Stroke::new(2.0, ACCENT);
        v.warn_fg_color = Tone::Warn.fg();
        v.error_fg_color = Tone::Error.fg();

        let radius = CornerRadius::same(RADIUS_SMALL);
        let w = &mut v.widgets;
        w.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
        w.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
        w.noninteractive.corner_radius = radius;

        w.inactive.weak_bg_fill = CONTROL;
        w.inactive.bg_fill = CONTROL;
        w.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        w.inactive.fg_stroke = Stroke::new(1.0, TEXT);
        w.inactive.corner_radius = radius;

        w.hovered.weak_bg_fill = Color32::from_gray(0x2D);
        w.hovered.bg_fill = Color32::from_gray(0x2D);
        w.hovered.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
        w.hovered.fg_stroke = Stroke::new(1.5, TEXT);
        w.hovered.corner_radius = radius;
        w.hovered.expansion = 0.0;

        w.active.weak_bg_fill = Color32::from_gray(0x36);
        w.active.bg_fill = Color32::from_gray(0x36);
        w.active.bg_stroke = Stroke::new(1.0, ACCENT);
        w.active.fg_stroke = Stroke::new(1.5, TEXT);
        w.active.corner_radius = radius;
        w.active.expansion = 0.0;

        w.open.corner_radius = radius;
    });
}

/// Notice kind: defines the box colors and icon.
#[derive(Clone, Copy)]
pub enum Tone {
    Info,
    Success,
    Warn,
    Error,
}

impl Tone {
    pub fn fg(self) -> Color32 {
        match self {
            Tone::Info => Color32::from_gray(0xB4),
            Tone::Success => ACCENT,
            Tone::Warn => Color32::from_rgb(0xFB, 0xBF, 0x24),
            Tone::Error => Color32::from_rgb(0xF8, 0x71, 0x71),
        }
    }

    pub fn bg(self) -> Color32 {
        match self {
            Tone::Info => Color32::from_gray(0x1A),
            Tone::Success => Color32::from_gray(0x1A),
            Tone::Warn => Color32::from_rgb(0x26, 0x1F, 0x0F),
            Tone::Error => Color32::from_rgb(0x29, 0x13, 0x13),
        }
    }

    pub fn border(self) -> Color32 {
        match self {
            Tone::Info => Color32::from_gray(0x33),
            Tone::Success => Color32::from_gray(0x33),
            Tone::Warn => Color32::from_rgb(0x52, 0x40, 0x12),
            Tone::Error => Color32::from_rgb(0x55, 0x21, 0x21),
        }
    }
}
