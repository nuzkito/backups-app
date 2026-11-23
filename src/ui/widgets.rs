//! Reusable components: sections, notices, buttons, chips and icons.

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Margin, Pos2, Rect, RichText, Sense, Shape,
    Stroke, Vec2,
};

use super::theme::*;

/// Numbered section: a step number, a title, a description and an optional counter on the
/// right.
pub fn section<R>(
    ui: &mut egui::Ui,
    step: u8,
    title: &str,
    subtitle: &str,
    count: Option<usize>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(RADIUS))
        .inner_margin(margin(PAD))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = GAP;
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = GAP;
                step_badge(ui, step);
                lines(ui, |ui| {
                    // Same height as the badge, so that the title is centered on it.
                    ui.spacing_mut().interact_size.y = BADGE;
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(title)
                                .family(semibold())
                                .size(15.0)
                                .color(TEXT),
                        );
                        if let Some(n) = count {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    pill(ui, &n.to_string());
                                },
                            );
                        }
                    });
                    ui.add(
                        egui::Label::new(RichText::new(subtitle).size(12.5).color(TEXT_WEAK))
                            .wrap(),
                    );
                });
            });
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = SPACE;
                add_contents(ui)
            })
            .inner
        })
        .inner
}

/// Diameter of the step badge of a section.
const BADGE: f32 = 24.0;

/// Circle with a section's step number.
fn step_badge(ui: &mut egui::Ui, step: u8) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(BADGE), Sense::hover());
    let p = ui.painter();
    p.circle_filled(rect.center(), BADGE / 2.0, ACCENT_SOFT);
    p.circle_stroke(
        rect.center(),
        BADGE / 2.0 - 0.5,
        Stroke::new(1.0, ACCENT_EDGE),
    );
    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        step.to_string(),
        FontId::new(12.0, semibold()),
        ACCENT,
    );
}

/// Small rounded badge with short text, such as a counter.
fn pill(ui: &mut egui::Ui, text: &str) {
    egui::Frame::new()
        .fill(CONTROL)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(8, 1))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(12.0).color(TEXT_WEAK));
        });
}

/// Draws a notice icon: a circle with a symbol inside.
fn tone_icon(ui: &mut egui::Ui, tone: Tone) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
    let p = ui.painter();
    let c = rect.center();
    p.circle_filled(c, 9.0, tone.fg());
    let stroke = Stroke::new(1.8, tone.bg());
    match tone {
        Tone::Success => {
            p.line(
                vec![
                    c + Vec2::new(-4.0, 0.2),
                    c + Vec2::new(-1.2, 3.0),
                    c + Vec2::new(4.0, -2.8),
                ],
                stroke,
            );
        }
        Tone::Warn | Tone::Error => {
            p.line_segment([c + Vec2::new(0.0, -4.5), c + Vec2::new(0.0, 1.0)], stroke);
            p.circle_filled(c + Vec2::new(0.0, 4.0), 1.1, tone.bg());
        }
        Tone::Info => {
            p.line_segment([c + Vec2::new(-4.0, 0.0), c + Vec2::new(4.0, 0.0)], stroke);
        }
    }
}

/// Notice box with an icon, a title and optional content below the title.
pub fn banner<R>(
    ui: &mut egui::Ui,
    tone: Tone,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::new()
        .fill(tone.bg())
        .stroke(Stroke::new(1.0, tone.border()))
        .corner_radius(CornerRadius::same(RADIUS))
        .inner_margin(margin(PAD))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = GAP;
                tone_icon(ui, tone);
                lines(ui, |ui| {
                    ui.add(
                        egui::Label::new(RichText::new(title).family(semibold()).color(tone.fg()))
                            .wrap(),
                    );
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = SPACE;
                        add_contents(ui)
                    })
                    .inner
                })
            })
            .inner
        })
        .inner
}

/// Green primary button with the given width.
pub fn primary_button(ui: &mut egui::Ui, text: &str, width: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, LARGE_HEIGHT), Sense::click());
    let fill = if response.is_pointer_button_down_on() {
        Color32::from_rgb(0x15, 0x80, 0x3D)
    } else if response.hovered() {
        Color32::from_rgb(0x22, 0xC5, 0x5E)
    } else {
        ACCENT_STRONG
    };
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(RADIUS), fill);
    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        FontId::new(14.5, semibold()),
        Color32::WHITE,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Secondary button with the same height as the primary one, for actions such as "Cancel".
pub fn secondary_button(ui: &mut egui::Ui, text: &str, width: f32) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).size(14.0).color(TEXT))
            .corner_radius(CornerRadius::same(RADIUS))
            .min_size(Vec2::new(width, LARGE_HEIGHT)),
    )
}

/// Full-width button with a dashed border, for adding items to a list.
pub fn add_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), LARGE_HEIGHT), sense);
    let hovered = enabled && response.hovered();
    let color = if !enabled {
        BORDER_STRONG
    } else if hovered {
        ACCENT
    } else {
        TEXT_WEAK
    };
    let p = ui.painter();
    if hovered {
        p.rect_filled(rect, CornerRadius::same(RADIUS_SMALL), ACCENT_SOFT);
    }
    let border = if hovered { ACCENT_EDGE } else { BORDER_STRONG };
    let r = rect.shrink(0.5);
    let corners = [
        r.left_top(),
        r.right_top(),
        r.right_bottom(),
        r.left_bottom(),
        r.left_top(),
    ];
    p.extend(Shape::dashed_line(
        &corners,
        Stroke::new(1.0, border),
        5.0,
        4.0,
    ));

    let galley = p.layout_no_wrap(text.to_owned(), FontId::proportional(13.5), color);
    let icon = 10.0;
    let gap = 8.0;
    let left = rect.center().x - (icon + gap + galley.size().x) / 2.0;
    let c = Pos2::new(left + icon / 2.0, rect.center().y);
    let stroke = Stroke::new(1.6, color);
    let h = icon / 2.0;
    p.line_segment([c - Vec2::new(h, 0.0), c + Vec2::new(h, 0.0)], stroke);
    p.line_segment([c - Vec2::new(0.0, h), c + Vec2::new(0.0, h)], stroke);
    let text_pos = Pos2::new(left + icon + gap, rect.center().y - galley.size().y / 2.0);
    p.galley(text_pos, galley, color);

    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

/// Draws a folder icon inside a 32-point square.
pub fn folder_tile(ui: &mut egui::Ui, active: bool) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
    let p = ui.painter();
    let (bg, fg) = if active {
        (ACCENT_SOFT, ACCENT)
    } else {
        (CONTROL, BORDER_STRONG)
    };
    p.rect_filled(rect, CornerRadius::same(RADIUS), bg);
    let icon = Rect::from_center_size(rect.center(), Vec2::new(16.0, 13.0));
    let tab = Rect::from_min_size(icon.left_top(), Vec2::new(7.0, 4.0));
    let body = Rect::from_min_max(icon.left_top() + Vec2::new(0.0, 2.0), icon.right_bottom());
    p.rect_filled(tab, CornerRadius::same(1), fg);
    p.rect_filled(body, CornerRadius::same(2), fg);
}

/// Thin, flat progress bar.
pub fn progress_bar(ui: &mut egui::Ui, fraction: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 6.0), Sense::hover());
    let p = ui.painter();
    let radius = CornerRadius::same(RADIUS_SMALL);
    p.rect_filled(rect, radius, CONTROL);
    if fraction > 0.0 {
        let mut fill = rect;
        fill.set_width((rect.width() * fraction).max(rect.height()));
        p.rect_filled(fill, radius, ACCENT);
    }
}

/// Exclusion pattern chip, with a "×" to remove it. Returns `true` if the "×" is clicked.
pub fn chip(ui: &mut egui::Ui, text: &str, enabled: bool) -> bool {
    egui::Frame::new()
        .fill(CONTROL)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(RADIUS_SMALL))
        .inner_margin(Margin {
            left: 9,
            right: 2,
            top: 1,
            bottom: 1,
        })
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.label(RichText::new(text).monospace().size(12.5).color(TEXT));
            remove_button(ui, enabled, 20.0).clicked()
        })
        .inner
}

/// Small frameless button with a drawn "×", for removing items from a list.
pub fn remove_button(ui: &mut egui::Ui, enabled: bool, size: f32) -> egui::Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), sense);
    let hovered = enabled && response.hovered();
    let p = ui.painter();
    if hovered {
        p.rect_filled(rect, CornerRadius::same(RADIUS_SMALL), Tone::Error.bg());
    }
    let color = if !enabled {
        BORDER_STRONG
    } else if hovered {
        Tone::Error.fg()
    } else {
        TEXT_WEAK
    };
    let c = rect.center();
    let d = (size / 6.0).min(4.0);
    let stroke = Stroke::new(1.5, color);
    p.line_segment([c + Vec2::new(-d, -d), c + Vec2::new(d, d)], stroke);
    p.line_segment([c + Vec2::new(-d, d), c + Vec2::new(d, -d)], stroke);
    if enabled {
        response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text("Remove")
    } else {
        response
    }
}
