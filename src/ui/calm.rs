//! Calm mode: one slow, quiet screen for when the rest is too much.
//!
//! A gentle ocean drifts in the background, the song's name sits in the
//! middle, and there is one small play button. Nothing reacts to the music,
//! nothing flashes, nothing moves fast. Home (or Esc) comes back.

use egui::{Color32, Frame, Rect, pos2, vec2};

use crate::app::App;
use crate::model::Action;
use crate::theme::{self, Icon};

/// Paints the ocean: a soft grid of deep blues and sea greens that drift
/// into each other over about a minute.
fn ocean(painter: &egui::Painter, rect: Rect, time: f64) {
    let cols = 36_usize;
    let rows = 22_usize;
    let t = time as f32;
    let deep = [6.0_f32, 18.0, 34.0];
    let mid = [20.0_f32, 64.0, 88.0];
    let glow = [38.0_f32, 104.0, 124.0];
    let mut mesh = egui::Mesh::default();
    for j in 0..=rows {
        for i in 0..=cols {
            let u = i as f32 / cols as f32;
            let v = j as f32 / rows as f32;
            let a = 0.5 + 0.25 * (u * 3.1 + t * 0.07 + (v * 2.3 + t * 0.05).sin() * 1.3).sin();
            let b = 0.25 * (v * 4.0 - t * 0.06 + (u * 2.1 - t * 0.04).sin() * 1.1).sin();
            let field = (a + b).clamp(0.0, 1.0);
            let lift = ((field - 0.7) / 0.3).clamp(0.0, 1.0) * 0.5;
            let mut channel = [0.0_f32; 3];
            for k in 0..3 {
                let base = deep[k] + (mid[k] - deep[k]) * field;
                channel[k] = base + (glow[k] - base) * lift;
            }
            mesh.vertices.push(egui::epaint::Vertex {
                pos: pos2(rect.left() + u * rect.width(), rect.top() + v * rect.height()),
                uv: egui::epaint::WHITE_UV,
                color: Color32::from_rgb(channel[0] as u8, channel[1] as u8, channel[2] as u8),
            });
        }
    }
    let stride = (cols + 1) as u32;
    for j in 0..rows as u32 {
        for i in 0..cols as u32 {
            let a = j * stride + i;
            mesh.indices
                .extend_from_slice(&[a, a + 1, a + stride, a + 1, a + stride + 1, a + stride]);
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        app.calm_mode = false;
        return;
    }
    let now = app.now_playing();
    egui::CentralPanel::default()
        .frame(Frame::new().fill(Color32::from_rgb(6, 18, 34)))
        .show(ui, |ui| {
            let rect = ui.max_rect();
            ocean(&ui.painter().with_clip_rect(rect), rect, ctx.input(|input| input.time));
            let quiet = Color32::from_gray(185);
            let softer = Color32::from_gray(140);
            let middle = rect.center();
            match &now {
                Some(now) => {
                    ui.painter().text(
                        pos2(middle.x, middle.y - 18.0),
                        egui::Align2::CENTER_CENTER,
                        &now.title,
                        theme::semibold(26.0),
                        quiet,
                    );
                    ui.painter().text(
                        pos2(middle.x, middle.y + 16.0),
                        egui::Align2::CENTER_CENTER,
                        &now.subtitle,
                        theme::regular(15.0),
                        softer,
                    );
                    let button = Rect::from_center_size(pos2(middle.x, middle.y + 70.0), vec2(44.0, 44.0));
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(button)
                            .layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight)),
                    );
                    let icon = if now.playing { Icon::PauseFilled } else { Icon::PlayFilled };
                    if theme::icon_button(
                        &mut child,
                        icon,
                        24.0,
                        Color32::from_gray(160),
                        Color32::from_gray(220),
                        if now.playing { "Pause" } else { "Play" },
                    )
                    .clicked()
                    {
                        app.actions.push(Action::TogglePlay);
                    }
                }
                None => {
                    ui.painter().text(
                        middle,
                        egui::Align2::CENTER_CENTER,
                        "Nothing is playing",
                        theme::regular(18.0),
                        softer,
                    );
                }
            }
            ui.painter().text(
                pos2(middle.x, rect.bottom() - 22.0),
                egui::Align2::CENTER_CENTER,
                "Calm mode. Press Home to come back.",
                theme::regular(12.0),
                Color32::from_gray(95),
            );
        });
    // Slow on purpose: a gentle frame rate is all the drift needs.
    ctx.request_repaint_after(std::time::Duration::from_millis(66));
}
