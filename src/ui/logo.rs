//! Application logo: a tape drive from the 70s-80s.

use std::f32::consts::FRAC_PI_2;

use eframe::egui::{self, Color32, CornerRadius, Pos2, Sense, Shape, Stroke, StrokeKind, Vec2};

use super::theme::RADIUS;

/// Side of the square logo, in points.
pub const SIZE: f32 = 44.0;

/// Paints the logo. The tape goes from the full reel, around two guide rollers, to the empty
/// reel.
pub fn paint(ui: &mut egui::Ui) {
    // The logo uses grays only.
    /// Background, and the holes in the hubs and rollers.
    const BACK: Color32 = Color32::from_gray(0x24);
    /// Border of the background.
    const EDGE: Color32 = Color32::from_gray(0x3C);
    /// Reel flanges, around the wound tape.
    const FLANGE: Color32 = Color32::from_gray(0x2E);
    /// Tape, wound on the reels and between them.
    const TAPE: Color32 = Color32::from_gray(0x74);
    /// Reel rims, hubs and rollers.
    const METAL: Color32 = Color32::from_gray(0xC8);
    /// Width of the reel rims and of the tape.
    const LINE: f32 = 1.5;
    /// Radius of the reel rims (to the center of the line).
    const REEL: f32 = 7.25;
    const HUB: f32 = 2.4;
    const ROLLER: f32 = 1.9;

    let (rect, _) = ui.allocate_exact_size(Vec2::splat(SIZE), Sense::hover());
    let p = ui.painter();
    let radius = CornerRadius::same(RADIUS + 2);
    p.rect_filled(rect, radius, BACK);
    p.rect_stroke(rect, radius, Stroke::new(1.0, EDGE), StrokeKind::Inside);

    let o = rect.left_top();
    // Center of each reel and radius of its wound tape: the left reel is almost full and the
    // right reel is almost empty.
    let reels = [
        (o + Vec2::new(12.5, 17.5), 5.5),
        (o + Vec2::new(31.5, 17.5), 3.7),
    ];
    let rollers = [o + Vec2::new(17.0, 32.0), o + Vec2::new(27.0, 32.0)];

    // Tape: it leaves the outer side of each wound tape, wraps around the rollers and goes
    // across between them. The line is on the inner side of the wound tape and on the outer
    // side of the rollers (with a small overlap, so that no gap shows between them).
    let wrap = ROLLER + LINE / 2.0 - 0.3;
    let [(left, left_pack), (right, right_pack)] = reels;
    let (left_start, left_wrap) = tangent(left, left_pack - LINE / 2.0, rollers[0], wrap, -Vec2::X);
    let (right_end, right_wrap) =
        tangent(right, right_pack - LINE / 2.0, rollers[1], wrap, Vec2::X);
    let mut tape = vec![left_start];
    tape.extend(arc(
        rollers[0],
        wrap,
        (left_wrap - rollers[0]).angle(),
        FRAC_PI_2,
    ));
    tape.extend(arc(
        rollers[1],
        wrap,
        FRAC_PI_2,
        (right_wrap - rollers[1]).angle(),
    ));
    tape.push(right_end);

    // The tape goes in front of the flanges and behind the rims.
    for (c, _) in reels {
        p.circle_filled(c, REEL, FLANGE);
    }
    p.add(Shape::line(tape, Stroke::new(LINE, TAPE)));
    for (c, pack) in reels {
        p.circle_filled(c, pack, TAPE);
        p.circle_stroke(c, REEL, Stroke::new(LINE, METAL));
        // Hub, with the hole for the drive spindle.
        p.circle_filled(c, HUB, METAL);
        p.circle_filled(c, 1.0, BACK);
    }
    for r in rollers {
        p.circle_filled(r, ROLLER, METAL);
        p.circle_filled(r, 0.7, BACK);
    }
}

/// Line that touches two circles on the same side, the side that `side` points to. Returns the
/// point where the line touches each circle.
fn tangent(c1: Pos2, r1: f32, c2: Pos2, r2: f32, side: Vec2) -> (Pos2, Pos2) {
    let d = c2 - c1;
    let along = d.normalized();
    let mut across = along.rot90();
    if across.dot(side) < 0.0 {
        across = -across;
    }
    let cos = (r1 - r2) / d.length();
    let normal = cos * along + (1.0 - cos * cos).sqrt() * across;
    (c1 + r1 * normal, c2 + r2 * normal)
}

/// Points of a circular arc, from the angle `from` to the angle `to` (in radians).
fn arc(center: Pos2, radius: f32, from: f32, to: f32) -> impl Iterator<Item = Pos2> {
    const STEPS: usize = 8;
    (0..=STEPS).map(move |i| {
        let a = from + (to - from) * i as f32 / STEPS as f32;
        center + radius * Vec2::angled(a)
    })
}
