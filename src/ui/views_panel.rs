//! The Views and panels window: a small panel that floats above everything.
//!
//! Right-click the disc in the sidebar (or the disc shown in the corner of
//! the full-screen views) to open it. It has no backdrop, does not close when
//! you click elsewhere, and can be dragged and resized.

use egui::{Color32, Context, CornerRadius, Frame, Id, Margin, Stroke};

use crate::app::App;
use crate::model::Action;
use crate::theme::{self, Icon, Palette};

use super::widgets;

/// The panel is always dark, so it reads over any theme or visualizer.
fn dark_palette(app: &App) -> Palette {
    let mut palette = Palette::dark();
    palette.accent = app.palette.accent;
    palette.accent_hover = app.palette.accent_hover;
    palette.on_accent = app.palette.on_accent;
    palette
}

/// One line of the zoom control: smaller, the size, bigger.
pub fn zoom_row(ui: &mut egui::Ui, app: &mut App, palette: &Palette) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        theme::text(ui, "Zoom", theme::medium(13.5), palette.text);
        if theme::soft_button(ui, palette, None, "-", false).clicked() {
            app.actions.push(Action::AdjustZoom(-1));
        }
        if theme::soft_button(ui, palette, None, &format!("{:.0}%", app.settings.zoom * 100.0), false)
            .on_hover_text("Back to 100%")
            .clicked()
        {
            app.actions.push(Action::AdjustZoom(0));
        }
        if theme::soft_button(ui, palette, None, "+", false).clicked() {
            app.actions.push(Action::AdjustZoom(1));
        }
    });
}

/// A small disc in the corner of the full-screen views, so the panel can be
/// reached from there too.
pub fn corner_disc(app: &mut App, ctx: &Context) {
    let full = app.fullscreen_vis || app.lyrics_fullscreen.is_some();
    // With the sidebar hidden its disc is hidden too, so one sits by the
    // player bar instead.
    if !full && !app.mini_active && app.settings.sidebar_visible {
        return;
    }
    let palette = dark_palette(app);
    egui::Area::new(Id::new("views-corner-disc"))
        .order(egui::Order::Foreground)
        .fixed_pos(if app.mini_active {
            egui::pos2(4.0, 4.0)
        } else if full {
            egui::pos2(14.0, 14.0 + theme::titlebar_inset(ctx))
        } else {
            egui::pos2(12.0, ctx.content_rect().bottom() - theme::PLAYER_BAR_HEIGHT - 40.0)
        })
        .show(ctx, |ui| {
            let button = theme::icon_button(
                ui,
                Icon::Disc,
                if app.mini_active { 16.0 } else { 22.0 },
                Color32::from_white_alpha(if app.mini_active { 110 } else { 150 }),
                palette.text,
                "Views",
            );
            if button.clicked() || button.secondary_clicked() {
                app.actions.push(Action::ToggleViewsPanel);
            }
        });
}

pub fn show(app: &mut App, ctx: &Context) {
    corner_disc(app, ctx);
    if !app.views_panel {
        return;
    }
    let palette = dark_palette(app);
    let frame = Frame::new()
        .fill(Color32::from_rgb(0x14, 0x16, 0x1a))
        .stroke(Stroke::new(1.0, palette.outline))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(14))
        .shadow(egui::epaint::Shadow {
            offset: [0, 8],
            blur: 28,
            spread: 0,
            color: Color32::from_black_alpha(160),
        });
    let mut close = false;
    egui::Window::new("Views")
        .id(Id::new("views-panel"))
        .title_bar(false)
        .order(egui::Order::Foreground)
        .frame(frame)
        .default_pos(egui::pos2(90.0, 90.0))
        .default_width(320.0)
        .min_width(240.0)
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(palette.text);
            egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
                // A bare disc where a title would be: press it to put the panel away.
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    if theme::icon_button(ui, Icon::Disc, 20.0, palette.accent, palette.text, "Close").clicked() {
                        close = true;
                    }
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                        let presets = [
                            ("Mini player", app.mini_active, Action::ToggleMiniPlayer),
                            ("Visualizer", app.settings.vis_shapes_value() != 0, Action::ToggleVisShapes),
                            ("Full screen visualizer", app.fullscreen_vis, Action::ToggleFullscreenVis),
                            ("Library only", false, Action::LibraryOnlyView),
                            ("Default view", false, Action::NormalView),
                        ];
                        for (label, on, action) in presets {
                            if theme::soft_button(ui, &palette, None, label, on).clicked() {
                                app.actions.push(action);
                            }
                        }
                    });
                });
                ui.add_space(8.0);
                let rows: [(&str, bool, Action); 9] = [
                    ("Library", app.settings.sidebar_visible, Action::ToggleSidebar),
                    ("Queue", app.show_queue_panel, Action::ToggleQueuePanel),
                    ("Side lyrics", app.show_lyrics_panel, Action::ToggleLyricsPanel),
                    ("Full screen lyrics", app.lyrics_fullscreen.is_some(), Action::ToggleLyricsFullscreen),
                    ("Full screen visualizer", app.fullscreen_vis, Action::ToggleFullscreenVis),
                    ("Visualizer settings", app.vis_panel, Action::ToggleVisPanel),
                    ("Visualizer", app.settings.vis_shapes_value() != 0, Action::ToggleVisShapes),
                    ("Big album art", app.settings.art_expanded, Action::ToggleArtExpanded),
                    ("Mini player", app.mini_active, Action::ToggleMiniPlayer),
                ];
                for (label, on, action) in rows {
                    let mut value = on;
                    ui.horizontal(|ui| {
                        theme::text(ui, label, theme::regular(13.5), palette.text);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if widgets::switch(ui, &palette, label, &mut value).changed() {
                                app.actions.push(action);
                            }
                        });
                    });
                    ui.add_space(2.0);
                }
                ui.add_space(8.0);
                zoom_row(ui, app, &palette);
                ui.add_space(10.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                    if theme::soft_button(ui, &palette, None, "Default controls", false).clicked() {
                        app.actions.push(Action::ResetBlockNudge);
                    }
                    if theme::soft_button(ui, &palette, Some(Icon::Info), "Shortcuts", false).clicked() {
                        app.actions.push(Action::ShowDialog(crate::model::Dialog::Shortcuts));
                    }
                });
            });
        });
    if close {
        app.actions.push(Action::ToggleViewsPanel);
    }
}
