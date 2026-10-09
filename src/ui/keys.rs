//! Keyboard shortcuts.

use std::borrow::Cow;

use egui::{Key, Modifiers};

use crate::app::App;
use crate::i18n::{Locale, gettext};
use crate::model::{Action, Dialog, Page};

pub(super) const fn platform_shortcut<'a>(ctrl: &'a str, cmd: &'a str) -> &'a str {
    if cfg!(target_os = "macos") { cmd } else { ctrl }
}

pub(super) const QUIT_SHORTCUT: &str = platform_shortcut("Ctrl+Q", "Cmd+Q");
pub(super) const WINAMP_SHORTCUT: &str = platform_shortcut("Ctrl+M", "Cmd+Shift+M");
pub(super) const MILKDROP_SHORTCUT: &str = platform_shortcut("Ctrl+Shift+K", "Cmd+Shift+K");

/// A shortcut the user may move to another plain key.
pub struct Bindable {
    pub id: &'static str,
    pub label: &'static str,
    pub default: Key,
    pub action: fn() -> Action,
}

/// Keys that cannot be moved, shown in the group they belong to.
pub fn fixed_for(group: &str) -> Vec<(&'static str, &'static str)> {
    let ctrl = |ctrl: &'static str, cmd: &'static str| platform_shortcut(ctrl, cmd);
    match group {
        "Views" => vec![("Esc", "Leave full-screen lyrics")],
        "Song lists" => vec![
            ("Delete", "Remove from this playlist"),
            (ctrl("Ctrl+A", "Cmd+A"), "Select all songs"),
            (ctrl("Ctrl+C", "Cmd+C"), "Copy the songs' links"),
            (ctrl("Ctrl+X", "Cmd+X"), "Cut the selected songs"),
            (ctrl("Ctrl+V", "Cmd+V"), "Add the pasted links"),
            ("Shift+↑/↓", "Extend or shrink the selection"),
        ],
        "Window" => vec![
            (QUIT_SHORTCUT, "Quit"),
        ],
        _ => Vec::new(),
    }
}

/// "No key yet": the stand-in default for shortcuts that start unassigned.
pub const UNSET: Key = Key::F35;

/// Whether a shortcut has a key right now.
pub fn is_set(app: &App, id: &str) -> bool {
    chord(app, id).is_some()
}

/// A saved choice: a key, with Shift held or not.
fn parse_chord(text: &str) -> Option<(Key, bool)> {
    let (shift, name) = match text.strip_prefix("Shift+") {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let key = Key::from_name(name)?;
    (key != UNSET).then_some((key, shift))
}

fn chord_text(key: Key, shift: bool) -> String {
    if shift {
        format!("Shift+{}", key.name())
    } else {
        key.name().to_string()
    }
}

/// The key (and whether Shift goes with it) a shortcut is on now, if any.
pub fn chord(app: &App, id: &str) -> Option<(Key, bool)> {
    match app.settings.key_bindings.get(id) {
        // A saved choice, which may be "no key" (cleared).
        Some(text) => parse_chord(text),
        None => BINDABLE
            .iter()
            .find(|b| b.id == id)
            .and_then(|b| (b.default != UNSET).then_some((b.default, false))),
    }
}

/// How a shortcut's key is written, such as `K` or `Shift+ArrowLeft`.
pub fn chord_label(app: &App, id: &str) -> Option<String> {
    chord(app, id).map(|(key, shift)| chord_text(key, shift))
}

/// The shortcuts that can be rebound, each one a single key.
pub const BINDABLE: &[Bindable] = &[
    // Panels and views.
    Bindable { id: "sidebar", label: "Show or hide the library sidebar", default: UNSET, action: || Action::ToggleSidebar },
    Bindable { id: "queue", label: "Queue and recently played", default: UNSET, action: || Action::ToggleQueuePanel },
    Bindable { id: "lyrics", label: "Lyrics side panel", default: UNSET, action: || Action::ToggleLyricsPanel },
    Bindable { id: "visualizer", label: "Visualizer settings", default: UNSET, action: || Action::ToggleVisPanel },
    Bindable { id: "art", label: "Big or small album art", default: UNSET, action: || Action::ToggleArtExpanded },
    Bindable { id: "normalview", label: "Normal view (close lyrics, queue, fullscreen)", default: UNSET, action: || Action::NormalView },
    Bindable { id: "views", label: "Views and panels (all the switches)", default: Key::Z, action: || Action::ToggleViewsPanel },
    Bindable { id: "mini", label: "Mini player", default: Key::M, action: || Action::ToggleMiniPlayer },
    // Fullscreen.
    Bindable { id: "fullscreen", label: "Full-screen visualizer", default: UNSET, action: || Action::ToggleFullscreenVis },
    Bindable { id: "lyricsfull", label: "Full-screen lyrics", default: UNSET, action: || Action::ToggleLyricsFullscreen },
    Bindable { id: "visshapes", label: "Visualizer on or off", default: UNSET, action: || Action::ToggleVisShapes },
    // Playback.
    Bindable { id: "playpause", label: "Play or pause", default: UNSET, action: || Action::TogglePlay },
    Bindable { id: "shuffle", label: "Toggle shuffle", default: UNSET, action: || Action::ToggleShuffle },
    Bindable { id: "repeat", label: "Cycle repeat", default: UNSET, action: || Action::CycleRepeat },
    Bindable { id: "mute", label: "Mute or unmute", default: UNSET, action: || Action::ToggleMute },
    Bindable { id: "back10", label: "Seek back 10 seconds", default: UNSET, action: || Action::SeekBy(-10_000) },
    Bindable { id: "forward10", label: "Seek forward 10 seconds", default: UNSET, action: || Action::SeekBy(10_000) },
    Bindable { id: "seekwidth", label: "Short, medium or full seek bar", default: UNSET, action: || Action::CycleSeekWidth },
    Bindable { id: "sharediscord", label: "Copy the song for Discord", default: UNSET, action: || Action::ShareToDiscord },
    Bindable { id: "lastfmlove", label: "Love the song on Last.fm", default: UNSET, action: || Action::LastfmLove },
    Bindable { id: "tap", label: "Tap tempo", default: UNSET, action: || Action::TapTempo },
    Bindable { id: "next", label: "Next song", default: UNSET, action: || Action::Next },
    Bindable { id: "previous", label: "Previous song", default: UNSET, action: || Action::Previous },
    Bindable { id: "volup", label: "Volume up", default: UNSET, action: || Action::VolumeBy(5) },
    Bindable { id: "voldown", label: "Volume down", default: UNSET, action: || Action::VolumeBy(-5) },
    Bindable { id: "speed", label: "Cycle speed scene", default: UNSET, action: || Action::CycleSpeedPreset },
    Bindable { id: "scenes", label: "Scenes panel", default: UNSET, action: || Action::ToggleSceneBrowser },
    // Going places.
    Bindable { id: "search", label: "Search", default: UNSET, action: || Action::FocusSearch },
    Bindable { id: "home", label: "Open Home", default: UNSET, action: || Action::Open(Page::Home) },
    Bindable { id: "liked", label: "Open Liked Songs", default: UNSET, action: || Action::Open(Page::LikedSongs) },
    Bindable { id: "settings", label: "Open Settings", default: UNSET, action: || Action::Open(Page::Settings) },
    Bindable { id: "pageback", label: "Go back a page", default: UNSET, action: || Action::Back },
    Bindable { id: "pageforward", label: "Go forward a page", default: UNSET, action: || Action::Forward },
    Bindable { id: "artistpage", label: "Open current artist", default: UNSET, action: || Action::OpenUri("artist".into()) },
    Bindable { id: "albumpage", label: "Open current album", default: UNSET, action: || Action::OpenUri("album".into()) },
    // Keys that used to be fixed; every key here starts unset.
    Bindable { id: "playspace", label: "Play or pause (second key)", default: UNSET, action: || Action::TogglePlay },
    Bindable { id: "seekback5", label: "Seek back 5 seconds", default: UNSET, action: || Action::SeekBy(-5_000) },
    Bindable { id: "seekfwd5", label: "Seek forward 5 seconds", default: UNSET, action: || Action::SeekBy(5_000) },
    Bindable { id: "volup5", label: "Volume up 5%", default: UNSET, action: || Action::VolumeBy(5) },
    Bindable { id: "voldown5", label: "Volume down 5%", default: UNSET, action: || Action::VolumeBy(-5) },
    Bindable { id: "tenth0", label: "Jump to 0% of the song", default: UNSET, action: || Action::SeekToPercent(0) },
    Bindable { id: "tenth1", label: "Jump to 10% of the song", default: UNSET, action: || Action::SeekToPercent(10) },
    Bindable { id: "tenth2", label: "Jump to 20% of the song", default: UNSET, action: || Action::SeekToPercent(20) },
    Bindable { id: "tenth3", label: "Jump to 30% of the song", default: UNSET, action: || Action::SeekToPercent(30) },
    Bindable { id: "tenth4", label: "Jump to 40% of the song", default: UNSET, action: || Action::SeekToPercent(40) },
    Bindable { id: "tenth5", label: "Jump to 50% of the song", default: UNSET, action: || Action::SeekToPercent(50) },
    Bindable { id: "tenth6", label: "Jump to 60% of the song", default: UNSET, action: || Action::SeekToPercent(60) },
    Bindable { id: "tenth7", label: "Jump to 70% of the song", default: UNSET, action: || Action::SeekToPercent(70) },
    Bindable { id: "tenth8", label: "Jump to 80% of the song", default: UNSET, action: || Action::SeekToPercent(80) },
    Bindable { id: "tenth9", label: "Jump to 90% of the song", default: UNSET, action: || Action::SeekToPercent(90) },
    // Window.
    Bindable { id: "closewindow", label: "Close the window", default: UNSET, action: || Action::CloseWindow },
    Bindable { id: "undo", label: "Undo a removal", default: UNSET, action: || Action::UndoRemoval },
    // Other.
    Bindable { id: "tutorial", label: "Keyboard shortcuts (this list)", default: Key::T, action: || Action::ShowDialog(Dialog::Shortcuts) },
];

/// Ready-made sets of keys to try. Each lists the shortcuts it moves; every
/// other shortcut goes back to its starting key (mostly none).
pub const PRESETS: &[(&str, &[(&str, &str)])] = &[
    (
        "Spotify-like",
        &[
            ("playspace", "Space"),
            ("next", "Shift+ArrowRight"),
            ("previous", "Shift+ArrowLeft"),
            ("seekback5", "ArrowLeft"),
            ("seekfwd5", "ArrowRight"),
            ("volup", "ArrowUp"),
            ("voldown", "ArrowDown"),
            ("shuffle", "S"),
            ("repeat", "R"),
            ("mute", "Shift+M"),
            ("search", "Slash"),
            ("queue", "Q"),
            ("lyrics", "L"),
            ("sidebar", "B"),
            ("home", "H"),
            ("settings", "Comma"),
            ("lastfmlove", "K"),
            ("fullscreen", "F"),
        ],
    ),
    (
        "Winamp-style",
        &[
            ("previous", "Z"),
            ("playpause", "X"),
            ("next", "B"),
            ("views", "Shift+Z"),
            ("seekback5", "ArrowLeft"),
            ("seekfwd5", "ArrowRight"),
            ("volup", "ArrowUp"),
            ("voldown", "ArrowDown"),
            ("shuffle", "S"),
            ("repeat", "R"),
            ("mute", "N"),
            ("visualizer", "V"),
            ("fullscreen", "F"),
            ("lyrics", "L"),
            ("queue", "Q"),
            ("sidebar", "E"),
            ("search", "Slash"),
        ],
    ),
    (
        "Vim-like",
        &[
            ("playspace", "Space"),
            ("seekback5", "H"),
            ("seekfwd5", "L"),
            ("voldown", "J"),
            ("volup", "K"),
            ("previous", "Shift+H"),
            ("next", "Shift+L"),
            ("search", "Slash"),
            ("queue", "Q"),
            ("lyrics", "Y"),
            ("shuffle", "S"),
            ("repeat", "R"),
            ("mute", "X"),
            ("home", "G"),
            ("settings", "Comma"),
            ("sidebar", "B"),
            ("fullscreen", "F"),
            ("lastfmlove", "Shift+K"),
        ],
    ),
    (
        "Just the basics",
        &[
            ("playspace", "Space"),
            ("next", "ArrowRight"),
            ("previous", "ArrowLeft"),
            ("volup", "ArrowUp"),
            ("voldown", "ArrowDown"),
            ("mute", "N"),
        ],
    ),
];

/// Puts a preset's keys in place of whatever was set before. A starting key
/// that a preset key now needs is cleared, so no two shortcuts share a key.
pub fn apply_preset(app: &mut App, which: u8) {
    let Some((_, entries)) = PRESETS.get(usize::from(which)) else {
        return;
    };
    app.settings.key_bindings.clear();
    let mut used: Vec<(Key, bool)> = Vec::new();
    for (id, text) in entries.iter() {
        if let Some(chord) = parse_chord(text) {
            used.push(chord);
        }
        app.settings
            .key_bindings
            .insert((*id).to_string(), (*text).to_string());
    }
    for bindable in BINDABLE {
        if app.settings.key_bindings.contains_key(bindable.id) || bindable.default == UNSET {
            continue;
        }
        if used.contains(&(bindable.default, false)) {
            app.settings
                .key_bindings
                .insert(bindable.id.to_string(), UNSET.name().to_string());
        }
    }
}

/// The groups the shortcuts list is shown in, in order, and which shortcut
/// belongs in each.
pub const CATEGORIES: &[(&str, &[&str])] = &[
    ("Views", &["views", "normalview", "mini", "fullscreen", "lyricsfull", "visshapes", "art"]),
    ("Panels", &["sidebar", "queue", "lyrics", "visualizer", "scenes"]),
    ("Playback", &["playpause", "playspace", "next", "previous", "shuffle", "repeat", "speed", "tap", "lastfmlove", "sharediscord"]),
    ("Volume", &["mute", "volup", "voldown", "volup5", "voldown5"]),
    ("Jump in the song", &["back10", "forward10", "seekback5", "seekfwd5", "seekwidth", "tenth0", "tenth1", "tenth2", "tenth3", "tenth4", "tenth5", "tenth6", "tenth7", "tenth8", "tenth9"]),
    ("Going places", &["search", "home", "liked", "settings", "pageback", "pageforward", "artistpage", "albumpage", "tutorial"]),
    ("Song lists", &[]),
    ("Window", &["closewindow", "undo"]),
];

/// The id of the shortcut waiting for its new key, if any.
pub fn rebinding(ctx: &egui::Context) -> Option<String> {
    ctx.data(|data| data.get_temp::<String>(egui::Id::new("rebinding-shortcut")))
        .filter(|id| !id.is_empty())
}

pub fn start_rebinding(ctx: &egui::Context, id: &str) {
    ctx.data_mut(|data| data.insert_temp(egui::Id::new("rebinding-shortcut"), id.to_string()));
}

pub fn stop_rebinding(ctx: &egui::Context) {
    ctx.data_mut(|data| data.insert_temp(egui::Id::new("rebinding-shortcut"), String::new()));
}

/// While a shortcut waits for its key, the next key pressed becomes it,
/// with Shift if Shift is held. Escape cancels. Returns true while
/// capturing, so nothing else reacts.
pub fn capture_rebind(app: &mut App, ctx: &egui::Context) -> bool {
    let Some(id) = rebinding(ctx) else {
        return false;
    };
    let pressed = ctx.input(|input| {
        input.events.iter().find_map(|event| match event {
            egui::Event::Key {
                key,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } => Some((*key, modifiers.shift, modifiers.command || modifiers.alt)),
            _ => None,
        })
    });
    if let Some((key, shift, other_modifier)) = pressed {
        if other_modifier && key != Key::Escape {
            app.toast("Only a plain key, or Shift with a key, can be used here.");
        } else if is_reserved(key) {
            app.toast("That key is reserved (see Keys that never change). Pick another.");
        } else if key != Key::Escape {
            let text = chord_text(key, shift);
            // A key already in use is taken from the shortcut that had it.
            let taken: Vec<&'static str> = BINDABLE
                .iter()
                .filter(|b| b.id != id && chord(app, b.id) == Some((key, shift)))
                .map(|b| b.id)
                .collect();
            for other in taken {
                if let Some(entry) = BINDABLE.iter().find(|b| b.id == other) {
                    app.toast(format!("{} was on that key, so it is now unset.", entry.label));
                }
                app.settings
                    .key_bindings
                    .insert(other.to_string(), UNSET.name().to_string());
            }
            app.settings.key_bindings.insert(id.clone(), text);
            app.actions.push(Action::SettingsChanged);
        }
        stop_rebinding(ctx);
        ctx.input_mut(|input| input.events.clear());
    }
    true
}

/// Keys that already mean something fixed, so a shortcut cannot be put on
/// them without breaking something else.
fn is_reserved(key: Key) -> bool {
    matches!(
        key,
        Key::Delete | Key::Backspace | Key::Enter | Key::Tab | Key::F2
    )
}

pub fn handle(app: &mut App, ctx: &egui::Context) {
    if capture_rebind(app, ctx) {
        return;
    }
    // The default keys were rearranged in 0.22 (W lyrics, F and D the two
    // full screens). Old saved choices would hide that, so they are dropped
    // once; keys changed after this stay.
    if app.settings.keymap_version < 2 {
        app.settings.key_bindings.clear();
        app.settings.keymap_version = 2;
        app.actions.push(Action::SettingsChanged);
    }
    // 0.30: the full screens moved next to Z on the keyboard (X visualizer,
    // C lyrics). Only the two old defaults are dropped; any other choice stays.
    if app.settings.keymap_version < 3 {
        for (id, old) in [("fullscreen", "D"), ("lyricsfull", "F")] {
            if app
                .settings
                .key_bindings
                .get(id)
                .is_some_and(|name| name.eq_ignore_ascii_case(old))
            {
                app.settings.key_bindings.remove(id);
            }
        }
        app.settings.keymap_version = 3;
        app.actions.push(Action::SettingsChanged);
    }
    // 0.48: M opens the mini player and mute moved to N. A mute key the
    // reader left on M is dropped so the two never share a key; any other
    // choice stays.
    if app.settings.keymap_version < 4 {
        if app
            .settings
            .key_bindings
            .get("mute")
            .is_some_and(|name| name.eq_ignore_ascii_case("M"))
        {
            app.settings.key_bindings.remove("mute");
        }
        app.settings.keymap_version = 4;
        app.actions.push(Action::SettingsChanged);
    }
    // 0.52: every shortcut starts unset and the reader binds what they
    // want. Choices from the old layout are dropped once.
    if app.settings.keymap_version < 5 {
        app.settings.key_bindings.clear();
        app.settings.keymap_version = 5;
        app.actions.push(Action::SettingsChanged);
    }
    // 0.63: the Genre column starts switched on, once, so songs get their
    // genres from MusicBrainz without hunting for the setting.
    if app.settings.keymap_version < 6 {
        if !app.settings.track_columns.shown(crate::model::SortColumn::Genre) {
            app.settings.track_columns.toggle(crate::model::SortColumn::Genre);
        }
        app.settings.keymap_version = 6;
        app.actions.push(Action::SettingsChanged);
    }
    // The keys the reader has chosen, read once, before the input is
    // borrowed. Shift versions are read first: egui ignores an extra Shift
    // when it matches, so the plain key would otherwise take them.
    let mut chords: Vec<(Key, bool, fn() -> Action)> = BINDABLE
        .iter()
        .filter_map(|b| chord(app, b.id).map(|(key, shift)| (key, shift, b.action)))
        .collect();
    chords.sort_by_key(|(_, shift, _)| !*shift);
    // A focused song row still takes Ctrl+arrow to change songs; a text
    // field uses those keys to move its caret.
    let editing_text = ctx.text_edit_focused();
    let mut actions = Vec::new();
    ctx.input_mut(|input| {
        let mut key = |modifiers: Modifiers, key: Key, action: Action| {
            if input.consume_key(modifiers, key) {
                actions.push(action);
            }
        };
        // The only key that is not the reader's to move. Everything else,
        // even Ctrl+W, is a shortcut they bind in the keys list.
        key(Modifiers::COMMAND, Key::Q, Action::Quit);
    });
    // The shortcuts the reader chose. Shift ones come first in the list.
    // A text field keeps its letters; anything else, even a focused song
    // row or button, lets them through.
    if !editing_text {
        for (assigned, shift, make) in &chords {
            let modifiers = if *shift { Modifiers::SHIFT } else { Modifiers::NONE };
            if ctx.input_mut(|input| input.consume_key(modifiers, *assigned)) {
                actions.push(make());
            }
        }
    }
    // Resolve the "open current artist/album" placeholders.
    for action in actions {
        match action {
            Action::OpenUri(kind) if kind == "artist" => {
                if let Some(id) = app
                    .now_playing()
                    .and_then(|now| now.artists.first().and_then(|artist| artist.id.clone()))
                {
                    app.actions.push(Action::Open(Page::Artist(id)));
                }
            }
            Action::OpenUri(kind) if kind == "album" => {
                if let Some(now) = app.now_playing() {
                    if let Some(id) = now.album_id {
                        app.actions.push(Action::Open(Page::Album(id)));
                    } else if let Some(id) = now.show_id {
                        app.actions.push(Action::Open(Page::Show(id)));
                    }
                }
            }
            other => app.actions.push(other),
        }
    }
    // Map mouse back and forward buttons to navigation.
    let (back, forward) = ctx.input(|input| {
        (
            input.pointer.button_pressed(egui::PointerButton::Extra1),
            input.pointer.button_pressed(egui::PointerButton::Extra2),
        )
    });
    if back {
        app.actions.push(Action::Back);
    }
    if forward {
        app.actions.push(Action::Forward);
    }
    if ctx.input(|input| input.key_pressed(Key::Escape)) {
        if app.dialog.is_some() {
            app.actions.push(Action::CloseDialog);
        } else if app.show_devices {
            app.show_devices = false;
        } else if app.lyrics_fullscreen.is_some() {
            app.actions.push(Action::SetLyricsFullscreen(false));
        }
    }
}

/// The rows of the keyboard shortcuts dialog: the keys, then what they do.
/// Key names stay as the keyboard prints them; words around them, and
/// every description, are translated.
pub fn shortcuts(locale: Locale) -> Vec<(Cow<'static, str>, Cow<'static, str>)> {
    let keys = |text: &'static str| Cow::Borrowed(text);
    vec![
        (
            keys(platform_shortcut("Ctrl+←  /  Ctrl+→", "Cmd+←  /  Cmd+→")),
            gettext(locale, "Previous or next"),
        ),
        (
            keys(platform_shortcut("Ctrl+↑  /  Ctrl+↓", "Cmd+↑  /  Cmd+↓")),
            gettext(locale, "Volume up or down"),
        ),
        (
            keys(platform_shortcut("Ctrl+Shift+L", "Cmd+Shift+L")),
            gettext(locale, "Report: freeze what is under the pointer"),
        ),
        (keys("Esc"), gettext(locale, "Lyrics: leave full screen")),
        (
            keys("Shift+↑  /  Shift+↓"),
            gettext(locale, "Song list: extend or shrink the selection"),
        ),
        (keys("Delete"), gettext(locale, "Remove from this playlist")),
        (
            keys(platform_shortcut("Ctrl+A", "Cmd+A")),
            gettext(locale, "Song list: select all"),
        ),
        (
            keys(platform_shortcut("Ctrl+C", "Cmd+C")),
            gettext(locale, "Song list: copy the selected songs' links"),
        ),
        (
            keys(platform_shortcut("Ctrl+X", "Cmd+X")),
            gettext(locale, "Playlist: cut the selected songs"),
        ),
        (
            keys(platform_shortcut("Ctrl+V", "Cmd+V")),
            gettext(locale, "Playlist: add the pasted song links"),
        ),
        (
            keys(platform_shortcut("Ctrl+F", "Cmd+F")),
            gettext(locale, "Search"),
        ),
        (keys("Alt+←  /  Alt+→"), gettext(locale, "Back or forward")),
        (
            keys(platform_shortcut("Ctrl+H", "Cmd+Shift+H")),
            gettext(locale, "Home"),
        ),
        (
            keys(platform_shortcut("Ctrl+L", "Cmd+L")),
            gettext(locale, "Liked Songs"),
        ),
        (
            keys(platform_shortcut("Ctrl+Shift+A", "Cmd+Shift+A")),
            gettext(locale, "Go to the playing artist"),
        ),
        (
            keys(platform_shortcut("Ctrl+Shift+B", "Cmd+Shift+B")),
            gettext(locale, "Go to the playing album"),
        ),
        (
            keys(platform_shortcut("Ctrl+,", "Cmd+,")),
            gettext(locale, "Settings"),
        ),
        (
            keys(platform_shortcut("Ctrl+/", "Cmd+/")),
            gettext(locale, "Keyboard shortcuts"),
        ),
        (
            keys(platform_shortcut("Ctrl+W", "Cmd+W")),
            gettext(locale, "Close the window"),
        ),
        (keys(QUIT_SHORTCUT), gettext(locale, "Quit")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppOptions;
    use crate::paths::AppDirs;
    use crate::settings::Settings;

    /// A text field edits with the arrow keys: Ctrl or Alt moves a word,
    /// Cmd moves to the end of the line, and Ctrl or Cmd with Up or Down
    /// to either end of the text. While a field has focus those keys are
    /// the field's, and the shortcuts on them wait until it lets go.
    #[test]
    fn a_focused_text_field_keeps_the_arrow_keys_it_edits_with() {
        let root =
            std::env::temp_dir().join(format!("spotifast-text-arrows-test-{}", std::process::id()));
        let dirs = AppDirs {
            config: root.join("config"),
            state: root.join("state"),
            cache: root.join("cache"),
        };
        let mut app = App::new(
            &crate::backend::Waker::default(),
            dirs,
            Settings::default(),
            AppOptions {
                media_controls: false,
                restore_sign_in: false,
                tray: false,
            },
        );
        crate::demo::populate(&mut app);

        // The modifiers as the platform reports its command key.
        let command = if cfg!(target_os = "macos") {
            Modifiers::MAC_CMD | Modifiers::COMMAND
        } else {
            Modifiers::CTRL | Modifiers::COMMAND
        };
        let ctx = egui::Context::default();
        let field = egui::Id::new("keys-test-field");
        let row = egui::Id::new("keys-test-row");
        let mut text = String::from("find this song");
        let end = text.chars().count();
        // Shortcuts first, then the page, as `ui::show` draws a frame.
        let frame = |app: &mut App, text: &mut String, events: Vec<egui::Event>| {
            app.actions.clear();
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    handle(app, ui.ctx());
                    ui.add(egui::TextEdit::singleline(text).id(field));
                    if ui
                        .interact(
                            egui::Rect::from_min_size(
                                egui::pos2(0.0, 100.0),
                                egui::vec2(200.0, 28.0),
                            ),
                            row,
                            egui::Sense::click(),
                        )
                        .clicked()
                    {
                        app.actions.push(Action::Open(Page::Home));
                    }
                },
            );
            output.textures_delta.clear();
            format!("{:?}", app.actions)
        };
        let press = |key: Key, modifiers: Modifiers| {
            vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }]
        };
        let caret_at = |index: usize| {
            let mut state = egui::TextEdit::load_state(&ctx, field).expect("the field's state");
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(
                    egui::text::CCursor::new(index),
                )));
            state.store(&ctx, field);
        };
        let caret = || {
            egui::TextEdit::load_state(&ctx, field)
                .and_then(|state| state.cursor.char_range())
                .map(|range| range.primary.index)
        };

        frame(&mut app, &mut text, vec![]);
        ctx.memory_mut(|memory| memory.request_focus(field));
        frame(&mut app, &mut text, vec![]);
        assert!(ctx.memory(|memory| memory.has_focus(field)));

        // Cmd+Left goes to the start of the line; Ctrl+Left back a word.
        let command_left = if cfg!(target_os = "macos") { 0 } else { 10 };
        for (keys, modifiers, lands, what) in [
            (
                Key::ArrowLeft,
                command,
                command_left,
                "the previous-song shortcut",
            ),
            (Key::ArrowLeft, Modifiers::ALT, 10, "the back shortcut"),
            (Key::ArrowUp, command, 0, "the volume shortcut"),
        ] {
            caret_at(end);
            assert_eq!(
                frame(&mut app, &mut text, press(keys, modifiers)),
                "[]",
                "{what} took {keys:?} from the focused field"
            );
            assert_eq!(
                caret(),
                Some(egui::text::CCursor::new(lands).index),
                "{keys:?} with {modifiers:?} moves the caret"
            );
        }
        assert_eq!(text, "find this song");

        // Space belongs to text editing while the field has focus.
        caret_at(end);
        let mut space = press(Key::Space, Modifiers::NONE);
        space.push(egui::Event::Text(" ".into()));
        assert_eq!(frame(&mut app, &mut text, space), "[]");
        assert_eq!(text, "find this song ");

        // With nothing being typed into, the shortcuts are the app's again.
        ctx.memory_mut(|memory| memory.surrender_focus(field));
        frame(&mut app, &mut text, vec![]);
        assert_eq!(
            frame(&mut app, &mut text, press(Key::ArrowRight, command)),
            format!("{:?}", [Action::Next])
        );
        assert_eq!(
            frame(&mut app, &mut text, press(Key::ArrowLeft, Modifiers::ALT)),
            format!("{:?}", [Action::Back])
        );
        // A focused non-text row still receives player/navigation shortcuts.
        ctx.memory_mut(|memory| memory.request_focus(row));
        frame(&mut app, &mut text, vec![]);
        assert!(ctx.memory(|memory| memory.has_focus(row)));
        assert!(!ctx.text_edit_focused());
        assert_eq!(
            frame(&mut app, &mut text, press(Key::Space, Modifiers::NONE)),
            format!("{:?}", [Action::TogglePlay]),
            "a focused row must not steal Space from play/pause"
        );
        assert_eq!(
            frame(&mut app, &mut text, press(Key::Enter, Modifiers::NONE)),
            format!("{:?}", [Action::Open(Page::Home)]),
            "Enter still activates the focused row"
        );
        assert_eq!(
            frame(&mut app, &mut text, press(Key::ArrowRight, command)),
            format!("{:?}", [Action::Next])
        );
        app.backend.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn shortcut_constants_name_the_platform_modifier() {
        let expected = if cfg!(target_os = "macos") {
            ["Cmd+Q", "Cmd+Shift+M", "Cmd+Shift+K"]
        } else {
            ["Ctrl+Q", "Ctrl+M", "Ctrl+Shift+K"]
        };
        assert_eq!(
            [
                QUIT_SHORTCUT,
                WINAMP_SHORTCUT,
                MILKDROP_SHORTCUT,
            ],
            expected
        );
    }

    #[test]
    fn shortcut_dialog_never_names_the_other_command_modifier() {
        let other = if cfg!(target_os = "macos") {
            "Ctrl+"
        } else {
            "Cmd+"
        };
        for (keys, _) in shortcuts(Locale::English) {
            assert!(!keys.contains(other), "wrong modifier in {keys}");
        }
    }

    #[test]
    fn shortcut_dialog_names_platform_reserved_alternatives() {
        let label = |description: &str| {
            shortcuts(Locale::English)
                .into_iter()
                .find(|(_, candidate)| candidate == description)
                .map(|(keys, _)| keys)
                .unwrap()
        };
        if cfg!(target_os = "macos") {
            assert_eq!(label("Home"), "Cmd+Shift+H");
        } else {
            assert_eq!(label("Home"), "Ctrl+H");
        }
    }

    /// Nothing is on a key until the reader puts it there, and a saved
    /// choice may carry Shift.
    #[test]
    fn every_shortcut_starts_unset_and_a_choice_may_carry_shift() {
        // Only the mini player starts on a key: M.
        assert!(BINDABLE.iter().all(|b| b.default == UNSET || b.id == "mini"));
        assert_eq!(parse_chord("K"), Some((Key::K, false)));
        assert_eq!(parse_chord("Shift+ArrowLeft"), Some((Key::ArrowLeft, true)));
        assert_eq!(parse_chord(UNSET.name()), None);
        assert_eq!(parse_chord("nonsense"), None);
        assert_eq!(chord_text(Key::K, true), "Shift+K");
    }
}
