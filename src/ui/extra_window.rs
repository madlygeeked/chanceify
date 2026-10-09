//! One extra window that shows the visualizer, for a second monitor.
//!
//! It draws with the very same settings and the very same audio as the main
//! window, so it needs no settings of its own: it copies what the main
//! window shows. Double-click (or F11) makes it full screen on whichever
//! monitor it sits on; drag it to another monitor first. Esc leaves full
//! screen, and closes the window when it is not full screen. The X closes it
//! too, and so does the End key or the Views panel in the main window, so it
//! can never be stuck on a screen you cannot see.

use egui::{Color32, Context, Frame, ViewportBuilder, ViewportClass, ViewportCommand, ViewportId, pos2};

use crate::app::App;
use crate::theme;

pub fn show(app: &mut App, ctx: &Context) {
    if !app.extra_vis {
        app.extra_vis_since = None;
        return;
    }
    let since = *app
        .extra_vis_since
        .get_or_insert_with(std::time::Instant::now);
    let id = ViewportId::from_hash_of("chanceify-extra-visualizer");
    let builder = ViewportBuilder::default()
        .with_title("chanceify visualizer")
        .with_inner_size([960.0, 540.0])
        .with_min_inner_size([240.0, 160.0]);
    let mut close = false;
    let now = app.now_playing();
    let was_full = app.fullscreen_vis;
    ctx.show_viewport_immediate(id, builder, |ui, class| {
        let ctx = ui.ctx().clone();
        // Where the platform cannot open a second window, egui would draw it
        // as a floating box inside the main one. That is not what this is for.
        if class != ViewportClass::Immediate && class != ViewportClass::Root {
            close = true;
            return;
        }
        if ctx.input(|input| input.viewport().close_requested()) {
            close = true;
            return;
        }
        let full = ctx.input(|input| input.viewport().fullscreen.unwrap_or(false));
        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            if full {
                ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
            } else {
                close = true;
                return;
            }
        }
        if ctx.input(|input| input.key_pressed(egui::Key::F11)) {
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!full));
        }
        egui::CentralPanel::default()
            .frame(Frame::new().fill(Color32::BLACK))
            .show(ui, |ui| {
                let rect = ui.max_rect();
                let response = ui.interact(rect, ui.id().with("extra-vis"), egui::Sense::click());
                if response.double_clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!full));
                }
                // The visualizer paints as it does full screen.
                app.fullscreen_vis = true;
                let moving = super::player_bar::lyrics_backdrop(app, ui, rect, now.as_ref(), 3);
                app.fullscreen_vis = was_full;
                if !moving {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "The visualizer is off. Turn it on in the main window (Views > Visualizer).",
                        theme::regular(15.0),
                        Color32::from_gray(150),
                    );
                }
                let age = since.elapsed().as_secs_f32();
                if age < 7.0 {
                    let fade = (1.0 - (age - 5.0).max(0.0) / 2.0).clamp(0.0, 1.0);
                    ui.painter().text(
                        pos2(rect.center().x, rect.bottom() - 24.0),
                        egui::Align2::CENTER_CENTER,
                        "Double-click for full screen. Esc closes.",
                        theme::regular(13.0),
                        Color32::from_white_alpha((150.0 * fade) as u8),
                    );
                }
            });
        ctx.request_repaint();
    });
    app.fullscreen_vis = was_full;
    if close {
        app.extra_vis = false;
        app.extra_vis_since = None;
    }
}
