//! The Views and panels window: a small panel that floats above everything.
//!
//! Right-click the disc in the sidebar (or the disc shown in the corner of
//! the full-screen views) to open it. It has no backdrop, does not close when
//! you click elsewhere, and can be dragged and resized.

use egui::{Color32, Context, CornerRadius, Frame, Id, Margin, Rect, Stroke};

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

/// The Views disc: a small disc the mouse can drag anywhere and click to open
/// the Views panel, on every screen. Until it has been moved, it stays out of
/// the way where the sidebar has its own disc.
pub fn corner_disc(app: &mut App, ctx: &Context) {
    let full = app.fullscreen_vis || app.lyrics_fullscreen.is_some();
    let moved = app.settings.views_disc;
    let palette = dark_palette(app);
    let screen = ctx.content_rect();
    let size = if app.mini_active { 16.0 } else { 22.0 };
    let home = if app.mini_active {
        egui::pos2(4.0, 4.0)
    } else if full {
        egui::pos2(14.0, 14.0 + theme::titlebar_inset(ctx))
    } else {
        egui::pos2(12.0, screen.bottom() - theme::PLAYER_BAR_HEIGHT - 40.0)
    };
    let at = moved.map_or(home, |[x, y]| egui::pos2(x, y));
    let at = egui::pos2(
        at.x.clamp(screen.left(), (screen.right() - size - 12.0).max(screen.left())),
        at.y.clamp(screen.top(), (screen.bottom() - size - 12.0).max(screen.top())),
    );
    let mut drop_at: Option<egui::Pos2> = None;
    let mut reset = false;
    egui::Area::new(Id::new("views-corner-disc"))
        .order(egui::Order::Foreground)
        .fixed_pos(at)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(size + 12.0), egui::Sense::click_and_drag());
            let hot = response.hovered() || response.dragged();
            // A round dark back, so the disc reads over any picture.
            ui.painter().circle_filled(
                rect.center(),
                (size + 12.0) / 2.0,
                Color32::from_black_alpha(if hot { 190 } else { 140 }),
            );
            ui.painter().circle_stroke(
                rect.center(),
                (size + 12.0) / 2.0,
                Stroke::new(1.0, Color32::from_white_alpha(if hot { 90 } else { 40 })),
            );
            theme::paint_icon(
                ui,
                Icon::Disc,
                rect,
                size,
                if hot { palette.text } else { Color32::from_white_alpha(if app.mini_active { 110 } else { 150 }) },
            );
            response
                .clone()
                .on_hover_text("Views. Click to open, drag to move, double-click to put it back.");
            if hot {
                ui.ctx().set_cursor_icon(if response.dragged() {
                    egui::CursorIcon::Grabbing
                } else {
                    egui::CursorIcon::PointingHand
                });
            }
            if response.dragged() {
                drop_at = Some(at + response.drag_delta());
            }
            if response.double_clicked() {
                reset = true;
            } else if response.clicked() || response.secondary_clicked() {
                app.actions.push(Action::ToggleViewsPanel);
            }
        });
    if reset {
        app.actions.push(Action::MoveViewsDisc(None));
    } else if let Some(place) = drop_at {
        app.actions.push(Action::MoveViewsDisc(Some([place.x, place.y])));
    }
}

/// Where the pop-out controls first appear: bottom centre, above the bar.
pub(super) fn default_float(ctx: &Context) -> [f32; 3] {
    let screen = ctx.content_rect();
    let width = 380.0;
    [
        (screen.center().x - width / 2.0).max(screen.left()),
        (screen.bottom() - theme::PLAYER_BAR_HEIGHT - 110.0).max(screen.top()),
        width,
    ]
}

/// The controls (buttons and song bar) as a panel of their own that can be
/// dragged anywhere by its top strip, widened from its right edge, snapped
/// onto the library sidebar, or put away from the right-click menu.
pub fn floating_controls(app: &mut App, ctx: &Context) {
    let Some([x, y, width]) = app.settings.float_controls else {
        return;
    };
    if app.fullscreen_vis || app.calm_mode {
        return;
    }
    let screen = ctx.content_rect();
    let width = width.clamp(300.0, 460.0);
    let mut change: Option<Option<[f32; 3]>> = None;
    let mut top_left = egui::pos2(x, y);
    egui::Area::new(Id::new("pop-out-controls"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(0.0, 0.0))
        .interactable(true)
        .show(ctx, |ui| {
            let Some(outer) = super::lyrics::controls_box_inner(app, ui, top_left, width, false) else {
                return;
            };
            let grip = Rect::from_min_size(outer.min, egui::vec2(outer.width(), 12.0));
            let grip_response = ui.interact(grip, Id::new("pop-out-grip"), egui::Sense::click_and_drag());
            let dots = Rect::from_center_size(egui::pos2(grip.center().x, grip.center().y + 1.0), egui::vec2(22.0, 4.0));
            for i in 0..3 {
                ui.painter().circle_filled(
                    egui::pos2(dots.left() + 3.0 + i as f32 * 8.0, dots.center().y),
                    1.6,
                    Color32::from_white_alpha(if grip_response.hovered() { 150 } else { 70 }),
                );
            }
            if grip_response.hovered() || grip_response.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
            }
            if grip_response.dragged() {
                let to = top_left + grip_response.drag_delta();
                top_left = egui::pos2(
                    to.x.clamp(screen.left(), (screen.right() - width).max(screen.left())),
                    to.y.clamp(screen.top(), (screen.bottom() - outer.height()).max(screen.top())),
                );
                change = Some(Some([top_left.x, top_left.y, width]));
            }
            let edge = Rect::from_min_max(
                egui::pos2(outer.right() - 6.0, outer.top() + 14.0),
                egui::pos2(outer.right(), outer.bottom() - 4.0),
            );
            let edge_response = ui.interact(edge, Id::new("pop-out-edge"), egui::Sense::drag());
            if edge_response.hovered() || edge_response.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
            if edge_response.dragged() {
                let wider = (width + edge_response.drag_delta().x).clamp(300.0, 460.0);
                change = Some(Some([top_left.x, top_left.y, wider]));
            }
            grip_response.context_menu(|ui| {
                if ui.button("Snap to the library sidebar").clicked() {
                    let side = app.settings.sidebar_width.clamp(300.0, 460.0);
                    change = Some(Some([
                        8.0,
                        (screen.bottom() - theme::PLAYER_BAR_HEIGHT - outer.height() - 8.0).max(screen.top()),
                        side,
                    ]));
                    ui.close();
                }
                if ui.button("Put back where it started").clicked() {
                    change = Some(Some(default_float(ui.ctx())));
                    ui.close();
                }
                if ui.button("Put the controls away").clicked() {
                    change = Some(None);
                    ui.close();
                }
            });
        });
    if let Some(place) = change {
        app.actions.push(Action::SetFloatControls(place));
    }
}

pub fn show(app: &mut App, ctx: &Context) {
    corner_disc(app, ctx);
    floating_controls(app, ctx);
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
                            ("My view", app.settings.my_view.is_some(), Action::ApplyMyView),
                            ("Save my view", false, Action::SaveMyView),
                            ("Calm mode", false, Action::ToggleCalm),
                            (if app.extra_vis { "Close visualizer window" } else { "New visualizer window" }, app.extra_vis, Action::ToggleExtraWindow),
                        ];
                        for (label, on, action) in presets {
                            if theme::soft_button(ui, &palette, None, label, on).clicked() {
                                app.actions.push(action);
                            }
                        }
                    });
                });
                ui.add_space(8.0);
                let float_on = app.settings.float_controls.is_some();
                let rows: [(&str, bool, Action); 11] = [
                    ("Always on top", app.settings.mini_on_top, Action::ToggleMiniOnTop),
                    ("Library", app.settings.sidebar_visible, Action::ToggleSidebar),
                    ("Queue", app.show_queue_panel, Action::ToggleQueuePanel),
                    ("Side lyrics", app.show_lyrics_panel, Action::ToggleLyricsPanel),
                    ("Full screen lyrics", app.lyrics_fullscreen.is_some(), Action::ToggleLyricsFullscreen),
                    ("Full screen visualizer", app.fullscreen_vis, Action::ToggleFullscreenVis),
                    ("Visualizer settings", app.vis_panel, Action::ToggleVisPanel),
                    ("Visualizer", app.settings.vis_shapes_value() != 0, Action::ToggleVisShapes),
                    ("Big album art", app.settings.art_expanded, Action::ToggleArtExpanded),
                    ("Mini player", app.mini_active, Action::ToggleMiniPlayer),
                    ("Pop-out controls", float_on, Action::SetFloatControls(if float_on { None } else { Some(default_float(ctx)) })),
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
                    if theme::soft_button(ui, &palette, None, "Reset the disc", false)
                        .on_hover_text("Puts the Views disc back where it started")
                        .clicked()
                    {
                        app.actions.push(Action::MoveViewsDisc(None));
                    }
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
