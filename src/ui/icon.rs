//! Application icon, rendered from the vector logo (`logo::paint`), without opening any window.
//!
//! Writes `assets/icon.ico` (the executable), `assets/icon.png` (the window
//! icon) and `assets/icon.svg` (the logo as a vector image). Run with `make icon` after a change
//! to the logo, and commit the three files.

use std::fmt::Write as _;
use std::path::Path;

use eframe::egui::{self, Color32, Shape, Stroke, StrokeKind, epaint};
use egui_kittest::Harness;
use image::codecs::ico::{IcoEncoder, IcoFrame};
use image::{ExtendedColorType, RgbaImage};

use super::logo;

/// Sizes in pixels of the images in the `.ico` file: the sizes that Windows uses for the
/// window, the taskbar, the Start menu, Explorer and the shortcuts, at the usual scales.
const SIZES: [u32; 9] = [16, 20, 24, 32, 40, 48, 64, 128, 256];

/// Renders the logo as a square image with a transparent background.
fn render(size: u32) -> RgbaImage {
    let mut harness = Harness::builder()
        .with_size(egui::Vec2::splat(logo::SIZE))
        .with_pixels_per_point(size as f32 / logo::SIZE)
        .wgpu()
        // The harness puts the UI in a panel: without a fill, it does not paint a background.
        .with_context_setup(|ctx| {
            ctx.all_styles_mut(|style| style.visuals.panel_fill = egui::Color32::TRANSPARENT);
        })
        .build_ui(|ui| {
            // The panel has a margin: an area at the origin puts the logo in the whole image.
            egui::Area::new(egui::Id::unique("logo"))
                .fixed_pos(egui::Pos2::ZERO)
                .fade_in(false)
                .show(ui.ctx(), logo::paint);
        });
    harness.run_steps(2);
    let image = harness.render().expect("cannot render the logo");
    assert_eq!(image.dimensions(), (size, size), "unexpected icon size");
    image
}

/// Paints the logo and returns the shapes, in points, with the logo at the origin.
fn shapes() -> Vec<Shape> {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::splat(logo::SIZE),
        )),
        ..Default::default()
    };
    // An area is not painted on its first frame (egui measures it): paint two frames.
    let mut output = egui::FullOutput::default();
    for _ in 0..2 {
        output = ctx.run_ui(input.clone(), |ui| {
            egui::Area::new(egui::Id::unique("logo"))
                .fixed_pos(egui::Pos2::ZERO)
                .fade_in(false)
                .show(ui.ctx(), logo::paint);
        });
        // The font texture is not necessary for the SVG.
        output.textures_delta.clear();
    }
    output
        .shapes
        .into_iter()
        .map(|clipped| clipped.shape)
        .collect()
}

/// Writes the logo as an SVG image, with the same shapes that egui paints.
fn svg() -> String {
    let mut body = String::new();
    for shape in shapes() {
        write_shape(&mut body, shape);
    }
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{s}\" height=\"{s}\" \
         viewBox=\"0 0 {s} {s}\">\n{body}</svg>\n",
        s = n(logo::SIZE)
    )
}

fn write_shape(out: &mut String, shape: Shape) {
    match shape {
        Shape::Noop => {}
        Shape::Vec(shapes) => shapes.into_iter().for_each(|s| write_shape(out, s)),
        Shape::Rect(r) => {
            // The SVG stroke is on the middle of the outline: move the outline to put the
            // stroke on the side that egui uses.
            let offset = match r.stroke_kind {
                StrokeKind::Inside => -r.stroke.width / 2.0,
                StrokeKind::Middle => 0.0,
                StrokeKind::Outside => r.stroke.width / 2.0,
            };
            let paint = |out: &mut String, rect: egui::Rect, radius: f32, attrs: String| {
                let _ = writeln!(
                    out,
                    "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" {attrs}/>",
                    n(rect.min.x),
                    n(rect.min.y),
                    n(rect.width()),
                    n(rect.height()),
                    n(radius.max(0.0)),
                );
            };
            // egui uses the same radius for the four corners in the logo.
            let radius = f32::from(r.corner_radius.nw);
            if r.fill != Color32::TRANSPARENT {
                paint(out, r.rect, radius, fill(r.fill));
            }
            if !r.stroke.is_empty() {
                let rect = r.rect.expand(offset);
                paint(out, rect, radius + offset, stroke(r.stroke));
            }
        }
        Shape::Circle(c) => {
            let center = format!(
                "cx=\"{}\" cy=\"{}\" r=\"{}\"",
                n(c.center.x),
                n(c.center.y),
                n(c.radius)
            );
            if c.fill != Color32::TRANSPARENT {
                let _ = writeln!(out, "  <circle {center} {}/>", fill(c.fill));
            }
            if !c.stroke.is_empty() {
                let _ = writeln!(out, "  <circle {center} {}/>", stroke(c.stroke));
            }
        }
        Shape::Path(p) => {
            assert!(
                !p.closed && p.fill == Color32::TRANSPARENT,
                "unsupported path"
            );
            let epaint::ColorMode::Solid(color) = p.stroke.color else {
                panic!("unsupported path color");
            };
            let points: Vec<String> = p
                .points
                .iter()
                .map(|pt| format!("{},{}", n(pt.x), n(pt.y)))
                .collect();
            let _ = writeln!(
                out,
                "  <polyline points=\"{}\" {} stroke-linejoin=\"round\"/>",
                points.join(" "),
                stroke(Stroke::new(p.stroke.width, color)),
            );
        }
        other => panic!("shape not supported in the SVG: {other:?}"),
    }
}

fn fill(color: Color32) -> String {
    format!("fill=\"{}\"", hex(color))
}

fn stroke(stroke: Stroke) -> String {
    format!(
        "fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"",
        hex(stroke.color),
        n(stroke.width)
    )
}

fn hex(color: Color32) -> String {
    assert!(color.is_opaque(), "the logo uses opaque colors only");
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

/// Number with at most 3 decimals and no trailing zeros.
fn n(value: f32) -> String {
    let s = format!("{value:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

#[test]
#[ignore = "generates the icon: make icon"]
fn icon() {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let images: Vec<RgbaImage> = SIZES.iter().map(|&s| render(s)).collect();

    // The corners are outside the rounded logo: they must be transparent.
    let largest = images.last().unwrap();
    assert_eq!(
        largest.get_pixel(0, 0).0[3],
        0,
        "the background is not transparent"
    );

    let frames: Vec<IcoFrame> = images
        .iter()
        .map(|img| {
            IcoFrame::as_png(
                img.as_raw(),
                img.width(),
                img.height(),
                ExtendedColorType::Rgba8,
            )
            .unwrap()
        })
        .collect();
    let ico = std::fs::File::create(assets.join("icon.ico")).unwrap();
    IcoEncoder::new(ico).encode_images(&frames).unwrap();
    largest.save(assets.join("icon.png")).unwrap();
    std::fs::write(assets.join("icon.svg"), svg()).unwrap();

    println!("{}", assets.join("icon.ico").display());
    println!("{}", assets.join("icon.png").display());
    println!("{}", assets.join("icon.svg").display());
}
