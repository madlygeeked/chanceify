# Re-applying our changes after a rebase

`CHANCEIFY.md` explains what each feature does. This file is the mechanical
half: the exact hunks, so a feature can be put back by hand when an upstream
rebase drops it.

Each patch lists the file, the anchor to find, and what to change. They are
written so they can be applied by reading, not by `git apply` — upstream moves
code, and a context diff fails where a description still works.

## 1. `build.rs` — emit the build stamp

Find `fn main() {` in `build.rs`. Add the call as the second statement, after
the i18n catalog line:

```rust
fn main() {
    fastframe_i18n::build::compile_catalogs("assets/i18n");
    build_stamp();
```

Then add the whole `build_stamp` function above `fn main`. It shells out to
`git rev-parse --short HEAD` and `git status --porcelain`, appends `+dirty`
when the tree is unclean, and prints:

```
cargo:rustc-env=SPOTIFAST_REVISION=<revision>
cargo:rustc-env=SPOTIFAST_BUILD_EPOCH=<seconds>
```

It prefers `SOURCE_DATE_EPOCH` over the wall clock so release builds stay
reproducible, and adds
`println!("cargo:rerun-if-changed=.git/HEAD")` so switching commits
regenerates the stamp.

**Breakage to expect:** if `src/build_info.rs` is present but `build.rs` lost
this call, the crate fails to compile with `environment variable
SPOTIFAST_REVISION not found`. That is deliberate — a missing stamp should
stop the build, not ship silently.

## 2. `src/lib.rs` — declare the module

In the module list, alphabetically between `bidi` and `credentials`:

```rust
pub mod bidi;
pub mod build_info;
pub mod credentials;
```

## 3. `src/build_info.rs` — new file

Copy the file from this branch. It has no dependencies beyond `env!`, so it
drops in cleanly against any spotifast version.

```sh
git checkout origin/main -- /dev/null 2>/dev/null   # no-op, for clarity
git show chanceify:src/build_info.rs > src/build_info.rs
```

After a rebase that kept the branch, the file is already there. This command
is for recovering it after a reset.

## 4. `src/ui/topbar.rs` — the pill

Two edits.

**Add the function** above `fn nav_button`. It early-returns unless
`crate::build_info::REVISION != "unknown"`, then shows
`0.11.2 (d201d05*)` with `*` for a dirty tree.

The width guard is the part that matters:

```rust
const PADDING: f32 = 20.0;
const SLACK: f32 = RIGHT_CONTROLS_WIDTH + 240.0 + ITEM_SPACING;
if width < SEARCH_IDEAL + PADDING + SLACK {
    return;
}
```

It shows only when the bar is wide enough that the search field keeps its
ideal width whatever the badges ask for. Do **not** replace this with a
leftover-space check: the badges lay out after the pill, so their width is
unknown at that point, and the pill will overlap the search field on narrow
windows. `demo::tests::the_top_bar_badges_never_cover_the_search_field` fails
immediately if you do.

**Call it** in `pub fn show`, right after the forward-arrow button is handled
and before the comment `// The badges sit at the right end but grow with their
text`:

```rust
build_stamp(ui, &palette, width);
```

`width` is `ui.available_width()`, already in scope at the top of `show`.

**Verify:** the pill appears above ~1000px wide and the badge test passes.

## 5. `src/ui/settings.rs` — the About row

Two call sites, both `format!("Spotifast {}", env!("CARGO_PKG_VERSION"))`.
Replace the `env!` with `crate::build_info::stamp()` in each:

- the `RowText::new(...)` in `about_rows`
- the `theme::text(...)` under the logo

**Verify:** Settings → About reads `Spotifast 0.11.2 (d201d05+dirty built ...)`.

## 6. `src/ui/sidebar.rs` — the one library menu and the three layouts

Upstream has `sort_menu()`, a text button that opens only the orders, plus a
row of `theme::soft_button` pills for the shelves, drawn in `contents()`.
Delete `sort_menu()` and the `ui.horizontal_wrapped` pill block; `sort_menu`'s
popup body becomes `library_menu()`:

```rust
fn library_menu(
    app: &mut App,
    ui: &mut egui::Ui,
    shelf: Filter,
    selected: LibrarySort,
    anchor: &egui::Response,
) { /* shelves, rule, orders, rule, view modes, own names, size */ }
```

It opens on an `egui::Popup::menu(anchor)`. The caller makes the anchor:

```rust
let mut library_button = None;
// inside the header's right_to_left block, before the other buttons:
library_button = Some(theme::icon_button(
    ui, Icon::ListMusic, 16.0, palette.secondary, palette.text,
    &gettext(locale, "Library view and order"),
));
```

`right_to_left` lays these out backwards, so the new button has to be added
*first* to sit leftmost of the group.

In `contents()`:

- `let filter = app.settings.library_shelf;` replaces the `sidebar-filter`
  temp read, and the `insert_temp(filter_id, ...)` line goes with it.
- `let sort = selected_sort(app, filter);` moves **above** the header row,
  because the menu needs it.
- `library_menu(app, ui, filter, sort, button)` replaces the pill row and
  the old `sort_menu` call.
- `grid_layout()` gains a `text_height` argument;
  `grid_text_height(app)` returns `0.0` when names are off, which is what
  makes the covers-only grid square.

Row and card text reads `app.settings.library_names_entry(entry.owned)`.

**Verify:** the header has three icon buttons, the menu switches shelf and
layout, and both survive a restart.

## 6b. `src/ui/sidebar.rs` — the cleaned-out header (supersedes parts of 6)

Four things in `contents()`:

1. Delete the `Icon::Search` nav row under Home entirely.
2. Delete the `Icon::Search` icon button in the Library header, and the
   grid/list button (`Icon::LayoutGrid` / `Icon::LayoutList`) — the menu
   already carries the view choice.
3. The `Library` heading becomes the search toggle. Allocate it with
   `Sense::click`, sized **only to the measured text width** (a full-width
   rect reaches under the buttons and eats their clicks), set
   `response.widget_info(...)` and `on_hover_text`, and draw a 1.5px
   underline on hover. The click toggles the same `show_search` temp the
   old button used, and clears `app.library.filter` when closing.
4. The field is drawn **inside the heading's own layout**, so it sits
   beside it. `show_search && room - width - 8.0 >= INLINE_SEARCH_MIN`
   keeps both; otherwise the heading is not drawn and the field takes the
   row (`INLINE_SEARCH_MIN` 90, `INLINE_SEARCH_MAX` 220).

Request focus by widget id after the row, not on the field's response:

```rust
if focus_search {
    ui.ctx().memory_mut(|memory| memory.request_focus(field_id));
}
```

The click is read after the row is drawn, so the field itself only exists
on the next frame; asking its response would be one frame too late.

Delete `Bigger names` from `library_menu()`, and with it
`Action::CycleLibraryNameSize` (`model.rs`), its handler (`app.rs`),
`Settings::NAME_SCALES`, `library_name_scale`, `library_name_scale()` and
`next_library_name_scale()`. Replace every
`theme::medium(base * app.settings.library_name_scale())` with:

```rust
theme::medium(fitted_name_size(ui, &entry.name, base, color, width))
```

`fitted_name_size()` measures with `layout_no_wrap` and scales down to fit,
never below `NAME_FIT_FLOOR` (0.78). `grid_text_height()` drops the scale
and returns `GRID_TEXT_HEIGHT` outright.

**Verify:** clicking "Library" opens the field beside it; the header has
three buttons; no Search row; long names shrink.

## 7. `src/ui/sidebar.rs` — the nav/Library rule

Find the `ui.painter().hline(...)` between the `Search` nav row and the
Library header. Replace those three statements with `section_rule(ui,
&palette);` and add the helper plus `PANEL_MARGIN_LEFT` /
`PANEL_MARGIN_RIGHT` (12 and 8 — keep them matching the panel `Frame`'s
`inner_margin`) and `rule_color()`.

The helper must widen the clip rect as well as the span. The panel's frame
margin trims anything painted at `ui.max_rect()`'s edges, which is why the
original rule stopped short of both sides.

**Verify:** the rule touches both panel edges and is visible on the dark
theme.

## 8. `src/backend.rs` and `src/app.rs` — the slow first playlist load

Two edits, both one-line-ish, both worth making together.

In `ApiRequest::background()`, delete `| Self::MyPlaylists { .. }` from the
`matches!`. The playlist list is the one request the sidebar is visibly empty
without, and it was queueing behind polls that repeat on a timer anyway.

In `walk_next_playlist()`, add after the `let Loadable::Loaded(playlists)`
line:

```rust
if self.library.playlists_next.is_some() || self.library.playlists_asked.is_some() {
    return;
}
```

Then find the two `self.walk_next_playlist();` call sites in
`handle_backend_events`. Leave the one in the `MembershipPlaylist` arm alone
— that is what chains the walk playlist to playlist. In the `MyPlaylists`
arm, guard the call:

```rust
if self.library.playlists_next.is_none() {
    self.walk_next_playlist();
}
```

**Verify:** first open fills faster, and the membership icons still arrive
afterwards without a reload.

## 9. `src/api/client.rs`, `src/backend.rs`, `src/app.rs` — the stuck spinner

Read the log at `%LOCALAPPDATA%\paolino\spotifast\data\spotifast.log` before
touching this. The symptom is a "Waiting for Spotify" spinner that never
clears; the cause is Spotify answering 429 with a 24-28 second `Retry-After`,
repeatedly.

`ApiRequest::MembershipPlaylist` is the offender. Make three changes.

**`src/api/client.rs`.** Make the cooldown readable:

```rust
pub async fn cooldown_remaining(&self) -> Duration {
    self.cooldown_until
        .lock()
        .await
        .saturating_duration_since(Instant::now())
}
```

Add `ApiError::is_rate_limited()`, true for `RateLimited` and for a
`Status { status: 429 }`. And add `path` to the rate-limit warning — without
it the log cannot tell you which endpoint is being throttled:

```rust
log::warn!("Spotify rate limit source={} path={} wait={wait:?}", self.source, path);
```

**`src/backend.rs`.** At the top of the `ApiRequest::MembershipPlaylist`
arm, before any request is sent:

```rust
if let Ok(client) = &selected
    && client.cooldown_remaining().await > Duration::ZERO
{
    return (
        ApiResponse::MembershipPlaylist {
            playlist,
            result: Err(ApiError::RateLimited),
        },
        None,
    );
}
```

**`src/app.rs`.** Add `MEMBERSHIP_RETRY` (120s) and a
`membership_paused_until: Option<Instant>` field on `Library`. In the
`ApiResponse::MembershipPlaylist` arm, match the result instead of
`if let Ok`, pausing on a rate limit. In `walk_next_playlist()`, return early
while the pause holds.

**Why:** a request sleeping inside `wait_for_cooldown()` still counts as
in-flight, because `activity.begin()` runs before the retry loop. A walk
that never stops asking therefore keeps `in_flight` above zero and the
spinner on, indefinitely. This is not a repaint bug and must not be fixed as
one.

**Verify:** with the log open, a throttled session logs `membership walk
paused` once rather than a rate limit every 28 seconds.

## 10. `src/ui/sidebar.rs` — the sidebar width cap

In `show()`, replace `.size_range(210.0..=600.0)` with
`.size_range(SIDEBAR_MIN_WIDTH..=sidebar_max_width(ui))`, hoist the `0.45`
in `expanded_art_side()` into `EXPANDED_ART_WIDTH_FRACTION`, and add:

```rust
fn sidebar_max_width(ui: &egui::Ui) -> f32 {
    let height = ui.available_height().max(1.0);
    let art = (height * EXPANDED_ART_WIDTH_FRACTION).max(80.0);
    (art + PANEL_MARGIN_LEFT + PANEL_MARGIN_RIGHT).min(SIDEBAR_ABSOLUTE_MAX)
}
```

**Verify:** drag the handle to its limit; the cover is at its largest and no
dead space sits to its right.

## 11. `src/api/gateway.rs` — route the library to the reader's own app

In `plan()`, add one arm **above** the catch-all:

```rust
PlaylistLibrary if personal_ready => ApiSource::Personal,
```

Add `shared_client()` and `served_personal()` to `ApiGateway`. `plan()` alone
is not enough: a personal app without the library scope answers a refusal,
and a refusal there costs the reader their whole library. So in
`src/backend.rs`, the `MyPlaylists` arm and the membership walk both retry
against `api.shared_client()` when the result is `ApiError::is_refused()`.

The membership walk's paging loop has to be callable by either client, so
lift it out of `handle()` into a free `async fn walk_playlist(client,
playlist)`.

**Verify:** `grep -c "source=personal" spotifast.log` is above zero.

## 12. `src/ui/player_bar.rs` — bar options

Add `VisColor` plus four settings fields and clamped accessors (see
CHANCEIFY.md for the field list), four `Action`s and their handlers, then in
`player_bar.rs` add `vis_menu()` attached to the visualizer's own
interaction response, and make `spectrum()` take the gap.

`merge_bands()` cuts groups **by proportion**, not by a fixed chunk size.
Fixed chunks drop the remainder — 75 bands into 12 bars yields 11 bars with
the last one empty — and a reader who asked for 12 must get 12.

**Note:** `egui::ColorPicker` is not available in this build. Use a swatch
plus a `TextEdit` for the hex, and only act on a colour `from_hex` accepts,
so a half-typed `#8b5` is not stored.

## Features 13-15 (sort menu, sidebar, visualizer, log filter)

Three independent patches. Each is a clean apply on top of the last.

**13 — Multi-key sort + resizable columns.** `TableSort` becomes
`{ keys: Vec<SortKey> }` and gains `single()`, `primary()`, `uses()`, `rank()`,
`promoted()`, `without()`, `reversed()`; it needs a **hand-written
`Deserialize`** so a `settings.json` written by the last version still reads.
Add `SortColumn::Playlists` and `Settings::TrackColumns { album, added_by,
added }`. In `collection.rs`, `sort_visible()` builds one `SortValue` per row
(no repeated `sort_by_key` closures), `compare_values()` handles the
playlist-less-last rule, and `playlist_key()` picks the first playlist
alphabetically while excluding `Page::Playlist(id)`. In `widgets.rs`,
`table_header()` **changes signature**: it now takes `Option<&TableSort>` and
`&TrackColumns` and **returns `HeaderAction`**, and the menu is opened with
`egui::Popup::context_menu(&response)` — `Popup::menu` only opens on a *left*
click. Drag handles are `column_divider()`, which returns `Option<Response>`
so a divider can anchor a right-click. If a rebase conflicts here, take
`collection.rs` and `widgets.rs` wholesale from the branch and re-run the
sort tests; they are the specification.

**14 — Shortcuts, visualizer steppers, player-bar text.** In `keys.rs` move
the bare `Key::B` binding from `ToggleSaved` to `ToggleSidebar` and add
`Key::F1` with `Modifiers::NONE` → `Action::ToggleCustomTitlebar`. In
`model.rs`/`app.rs` that is one new `Action` plus a handler calling
`SetCustomTitlebar(!settings.custom_titlebar)`. The visualizer menu replaces
its two cycling rows with `stepper_row()`; the `Cycle*` actions become
`StepVis*(i32)`. `visualizer()` must call
`App::now_playing_cover_colour()` — **not** `now_playing_tint()`, which
returns `None` unless `accent_from_art` is set, and that is the bug being
fixed. `now_playing_block()` gates the small cover on
`!art_expanded || !sidebar_visible` and takes its sizes from
`theme::medium(16.0)` / `theme::regular(13.0)`.

**15 — Log filter.** One string, in `entrypoint.rs`. Append
`librespot_core::dealer=error,librespot_connect::state=error` to **both**
branches of `default_filter` — the verbose one too, or `--verbose` brings the
warning straight back. Nothing else changes; do not add a filter for a
librespot module that has not actually complained.

## Features 16-17 (visualizer panel, flow, overlay card)

**16 — The panel that stays open.** The single most important line is
`.close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)` on
`vis_menu()`'s popup. egui's default `CloseOnClick` closes the panel on
*any* click inside it, which is why every stepper threw the reader back
out. If a rebase drops this line the panel reverts to one-shot.

The sliders replace `stepper_row()`, which is deleted along with
`Action::StepVisBars`/`StepVisGap` and `Settings::{VIS_BAR_COUNTS,
VIS_GAPS, next_player_bar_vis_bars}`. `player_bar_vis_bars` **widened from
`u8` to `u16`** — 256 bars do not fit in a byte, and forgetting this is a
loud compile error rather than a quiet one. `Action::SetVisBars(u16)` /
`SetVisGap(f32)` clamp in the handler *and* in the accessor.

`spread_bands()` is the mirror of `merge_bands()`: one merges the
analyser's bands down to fewer bars, the other spreads them up to more
points, interpolating between and holding the ends. They must stay
opposites — if a rebase makes `merge_bands()` interpolate, dense bars start
inventing peaks between the bands.

The flow sheets are one mesh built back to front. Note the winding: crest
points then foot points, both left to right, joined as quads. Drawing the
foot points in reverse looks plausible and fills nothing.

**17 — Names and the overlay card.** The menu item's label is load-bearing:
it used to read as a command ("Name my own playlists") while being a
checkbox, so pressing it turned the names off. `library_names_entry()` is
unchanged; the wording is the fix. If upstream renames the label, check the
demo test, which drives the menu by that exact string.

`now_playing_overlay()` lives in `player_bar.rs` and is `pub`, called from
`sidebar.rs`'s `paint_expanded_art()`. It is in `player_bar.rs` rather than
`sidebar.rs` because it needs the same link/heart/context-menu handling as
the bar's own block, and duplicating that is how the two drift apart.
`now_playing_block()` **returns early** when the big art is showing — do not
"fix" that as a dead branch.

## Feature 18 (bar cap, flow tuning, Details toggle, grouped menu)

**The bar cap.** `visualizer()` must branch on direction: below the
analyser's band count the bands are **merged** down, above it they are
**spread** out. Merging above the count was the bug — `merge_bands()`
clamps to its input length, so a slider set to 256 drew 75 and looked
broken. If a rebase "simplifies" that branch back to a single `merge_bands`
call, re-read this first.

**Flow tuning.** `FlowStyle` is a small struct of what the flow needs, not
the whole settings; the drift speed is applied at the call site, not in the
struct. The three fields (`player_bar_vis_flow_sheets/_depth/_speed`) each
have a `RangeInclusive` constant that is the single source of truth for both
the slider and the handler's clamp. Keep them as `RangeInclusive` rather
than loose MIN/MAX pairs — they are read as `.start()`/`.end()` in three
places and duplicating them is how a slider and its handler drift apart.

**Details.** `library_own_playlist_names` was **renamed**, not reused. The
old setting meant "hide my own playlists' names"; the new one means "show
the detail line", which is the opposite. Reusing the field would have
silently inverted everyone's saved preference on upgrade. A file with the
old key and no `library_details` reads as `true` because the struct is
`#[serde(default)]`. `Action::SetOwnPlaylistNames` became
`SetLibraryDetails` for the same reason — do not rename it back.

**Grouped menu.** `widgets::menu_group()` is a dim caption, not a
separator: three captions beat two rules and a dozen rows. The shelf, order
and layout sections are each a `menu_group()` followed by its items.

## Feature 19 (membership walk reads the playlist cache)

The membership walk — feature 4's playlist icons — pages through every
song in every playlist the reader owns, 50 at a time, on every launch. That
is 60–100 requests against a Development Mode quota that runs out almost
immediately, and it was the cause of the `QUOTA_EXCEEDED` error rather
than anything Spotify did.

`walk_playlist()` now reads `cache/playlists/<account>/<id>.json` **first**
and returns without sending anything when the cache is complete. The
completeness test is the important part:

```rust
let complete = cached.next_offset.is_none()
    && cached.total.is_none_or(|total| total as usize == uris.len());
```

`next_offset` being `Some` means the cache is a *prefix* — a partial read
from an earlier session. Treating that as the whole playlist would have
the index claiming songs are in a playlist it never read, which is worse
than the request it saves. Do not simplify this to
`cached.next_offset.is_none()` alone.

`ApiRequest::MembershipPlaylist` gained a `cache: PathBuf` field. The path
comes from `App::playlist_cache_path()`, which uses `self.user.id` — the
same account string the worker's gateway reports, already known to the
interface from `/me`, so no new plumbing through `Backend` was needed.

**There is no legitimate way around the quota.** It is a server-side limit
on the app's client id. Do not add token swapping, shared-credential tricks
or request batching to "get around" it; reduce the requests instead.

## Feature 20 (header menu anchors, dividers, Swirl, monochrome, envelope)

**Header anchors.** `table_header()` collects every interactive element's
`egui::Response` into an `anchors` vec and draws the menu for whichever one
reports `secondary_clicked()`. Two borrow-checker constraints shaped this
and will shape it again: the `heading` closure takes `anchors` as a
*parameter* (capturing it makes the later use of the same vec a second
mutable borrow), and anything with a hover hint must `.clone()` the
response before `on_hover_text`, which consumes it.

**Divider direction.** The optional columns are laid out **from the right
inwards** — `cx` starts at `rect.right()` and walks left — so the edge of a
column is its *left* boundary. Dragging that edge right narrows the column:
`width - delta`, not `+ delta`. This was wrong in the first version and is
the kind of thing a rebase will reintroduce if the layout changes.

**Monochrome.** `vis_colours()` branches on `MONOCHROME_SATURATION`. Both
ends are placed *below* the brightness ceiling on purpose: `within_brightness`
scales anything above it down to the ceiling per colour, so two ends both
above it converge to the same grey and the bar goes flat. Do not "fix" the
grey case by raising the values.

**Envelope.** `vis::scope_line()` replaced the raw every-seventh-sample
read with the extreme of a run that widens with the point count. The test
asserts a rising ramp stays non-decreasing and that a lone loud sample is
still caught — an envelope that averaged instead of taking the extreme
would pass the first and fail the second.

## Feature 21 (sticky header menu, stuck dividers, bar-wide Swirl, queue trim)

**A popup must be drawn every frame it is open.** This is the single most
easily reintroduced mistake in the set. Feature 20 drew the header sort menu
only for an anchor that reported `secondary_clicked()` *that* frame; egui
sees a popup that is not drawn as a close, so the menu flashed and vanished.
`table_header()` now draws `egui::Popup::context_menu` for **every** anchor on
**every** frame. `now_playing_overlay()` and the vis panel already worked this
way. Panel popups additionally need
`.close_behavior(PopupCloseBehavior::CloseOnClickOutside)`.

**Handle overlap.** `column_divider()` takes `placed: &mut Vec<f32>` and
returns `None` for any handle within `DIVIDER_HANDLE_WIDTH` (14px) of one
already placed this frame. Without it, two overlapping grab rects leave the
topmost taking every click and the one beneath permanently ungrabbable.

**A drag stops at the minimum.** `columns.set(column, width.max(TrackColumns::MIN))`.
A drag *below* `MIN` used to hide the column, which removed its own handle
with it — the "stuck forever" report. `TrackColumns::set` still maps
sub-minimum widths to 0, which is correct for the *menu*; the clamp belongs at
the drag site, not in `set`.

**Swirl geometry.** `SWIRL_RINGS` went 7 → 9, `reach` is
`rect.height() * 0.62` (was a square-ish `side`), and the chain is
`rect.width() + reach * 2.0` wide with each ring's centre at
`rect.left() - reach + along * span`, so the end rings bleed off both edges.
Each ring also drifts vertically on `sin(turn * 0.7 + along * 2.3)`. The core
glow became a `shaded()` wash along the foot. The ring maths is unchanged:
closed curve, radius sampled from the spectrum around the circle.

**Queue trim is a menu, not a button.** `queue_compact` lives on a
right-click of the queue header row (`ui.interact(ui.min_rect(), ...)` +
`egui::Popup::context_menu`). As a third `icon_button` in the `Sides` row it
wrapped the Queue/Recent chips onto two lines at minimum panel width and
broke `the_narrowest_panels_keep_their_headers_on_one_row`. Keep it a menu.

**Grid name scale is relative, not absolute.** `grid_name_size(card_width) =
(card_width * 0.125).clamp(12.0, 26.0)` and the room under a card is
`grid_name_size(card_width) * GRID_TEXT_LINES` (3.2). Quoting the room in
pixels (the old `GRID_TEXT_HEIGHT = 44.0`) broke the moment the name scaled,
because the text then overflowed the card and the double-click test on a grid
card failed. `grid_card_width()` duplicates `grid_layout()`'s column maths
deliberately, so the two must be changed together.

## Feature 22 (BPM column, fetched once and cached for good)

**Do not reach for Spotify's audio-features endpoint.** It is gone.
`GET /v1/audio-features/{id}` returned 403 for this build's own stored
credential when probed (see `examples/audio_features_probe.rs`, which is kept
in the tree for exactly this). Spotify deprecated it on 27 November 2024 for
every app without a quota extension from before then. It carried both `tempo`
and `key`; neither is reachable now, and Spotify's client shows its key and
tempo from data it already holds rather than from any API.

**The join is the ISRC, not the title.** Deezer's
`/track/isrc:{isrc}` is an exact lookup and needs no account. Spotify already
returns `external_ids.isrc` on every track, and `Track::recording_key()`
already existed before this feature — reuse it rather than matching on title
and artist. The older "a Spotify track id is also a Deezer track id" trick is
**dead**: Deezer ids are decimal integers, not the hex Spotify uses.

**Cache the absences, or the feature re-fetches forever.** `bpm::Store` is a
`HashMap<String, Option<f32>>` where `None` means *asked, and there is none*.
Deezer reports `0` for an unmeasured recording and an error object for an
unknown ISRC; both parse to `None` and both are stored. Without this an
ambient or spoken-word playlist re-requests every row on every launch.

**In-flight is not the same as answered.** `Worker::bpm_pending` is a separate
`HashSet`. An earlier version marked the recording as known when the request
*started*, which turned a network blip into a permanent "this song has no
tempo". Only a real answer reaches `Event::Bpm`, and only `Event::Bpm` writes
the cache.

**Write the cache on a pause, not per row.** `BPM_SAVE_AFTER` (10s) plus a
flush in `save_bpm()` on the next dispatch and at `Command::Shutdown`. That
flush is also where `bpm_pending` is cleared — by then every request before
it has either landed or failed, and forgetting a failed one is exactly right.

**Sorting needs a re-sort when an answer lands.** `App::bpms_revision` bumps
on every `Event::Bpm` and is part of the `TableCache` validity check, or a
list sorted by tempo would stay in whatever order a half-filled column
produced. `SortValue::Optional(Option<i32>)` holds tenths, and
`compare_values` sends `None` last in both directions — a song with no tempo
has no place in a tempo order whichever way it runs.

**Layout.** `table_layout` gained a sixth column and then eight arguments, so
the flags moved into `ColumnsShown`. The tempo column is budgeted **first**, so
it is the last optional column dropped when the window is tight. `RESIZABLE`
and `SORTABLE` gained an entry each.

**Localisation.** "BPM" is deliberately **not** added to `assets/i18n`. It is
the same word in every language, and a message a catalogue does not carry
falls back to its English source — `the_tempo_heading_sorts_and_drags_like_any_other_column`
asserts that so a future change cannot quietly leave the heading blank.

## Feature 23 (lined-up columns, a shouting menu, warped artwork, an open card)

**The header and the rows must be drawn in the same order, and the cursor must
step over every column it draws.** `table_header` was missing `cx +=
laid.added`, so BPM landed on DATE ADDED's left edge. Two headings, one
position, every tempo under the wrong name. Keep the comment above the
headings block and the test that renders both and compares.

**An unsorted list is newest first.** In `view_indices`, the `else` branch
sorts by `TableSort::single(SortColumn::Added, false)`. Rows with no date all
tie, so an undated list is unchanged. The menu's way back is one `NEWEST FIRST`
row, and there is no "Sort by" heading — a heading that cannot be clicked is a
button that does nothing. If you add a menu line, uppercase it: the test
filters the labels the menu added and compares them to their own uppercase.

**A marquee needs `reserve`.** `song_name_cell` takes the whole
`available_width` when it has to scroll, which pushed the artists and the
inline date off the end of the row. It now takes a `reserve` argument: the
thin row passes a fraction of the title column, the two-line row passes zero.
Anything drawn after the name on the same line needs its room held back.

**Don't use `animate_value_with_time(...).fract()` for a marquee.** It wraps
on its own schedule and cannot be parked. The progress lives in
`ui.memory_mut(|m| m.data.get_temp::<f32>(id))`, advances on
`input.stable_dt`, and is held at `MARQUEE_HOLD` at each end.

**A polar warp belongs off the UI thread.** `swirl_art::warp` runs in
`ArtLoader::spawn_processed`, which had to be added because `ArtLoader::inner`
is private. Decode through `ImageReader` with `Limits` like
`blurred_background`, cap the download at 24MB, and answer `None` for anything
that is not an image — a bad cover must not take the app down.

**Settings need a range, a default and a clamped accessor.** The flow offset is
`VIS_FLOW_OFFSET = 0.0..=0.5`, default `0.0`, accessor
`player_bar_vis_flow_offset()` that rejects non-finite values, and
`Action::SetFlowOffset` clamped in `App::apply`. Test the accessor, not the
field.

**egui 0.36, again.** `ColorImage::pixels` is a field, not a method;
`ColorImage::width()` is `usize`; `Color32`'s components are private, so
compare colours with `!=` or `to_array()`. A popup must still be drawn every
frame it is open, and `Slider` has no number field — `slider_row` builds one
out of `TextEdit::singleline` plus `lost_focus()`.

## Feature 24 (a tempo that exists, a shorter card, no ADDED BY, a queue without a clock)

**Deezer's ISRC lookup has no `bpm`.** Proved by hand:

```
GET /track/isrc:GBARL9300135 -> {"id":15646529, ... "rank":1620 ...}   # no bpm
GET /track/15646529           -> {..., "bpm":113.3, "gain":-9.2}       # has it
```

The tempo lives on the track's own page. So the worker reads the ISRC body,
takes the numeric `id` out of it, and fetches that. Beware when testing a
regex over Deezer JSON by hand: `.*"id":(\d+)` matches the *last* `id` in the
body (`track_position`, `disk_number`), not the track's, and you will fetch a
track that does not exist.

**A cache written by a broken lookup is worse than no cache.** Version 1 wrote
`null` for every recording because the tempo was never in the answer, and the
store treats `null` as settled. The file is versioned now
(`{"version":2,"tempos":{…}}`) and anything else is discarded on load. Bump
`bpm::VERSION` whenever a change would make what is on disk mean something
other than what it says.

**Keep a column's data and delete its column.** `TrackColumns::added_by` stays
as a field so an old `settings.json` still deserialises, but `width()` returns
0 and `set()` writes 0, and it is out of `SORTABLE`/`RESIZABLE`. Deleting the
field outright turns every old settings file into a parse error.

**Adding a `TrackRow` flag touches thirteen constructors.** `show_time` was
added with a `sed` over every `show_bpm:` line in `TrackRow { … }`, which is
the last field in every one of them.

## Feature 25 (inspector, gentler swirl, three clicks on the art, no waveform)

**A debugging aid is cheaper than a screenshot.** `src/inspect.rs` records
every string `theme::text`/`theme::link` draws into a `Mutex<Vec<(Rect,
String)>>`, but only while armed, and the panel filters those containing the
pointer. A layout bug is almost always "something is drawn where I think
nothing is", and this answers it without guessing. Keep the `armed()` check
in the paint path: an unconditional lock and push per label costs real frame
time on a list of a thousand rows.

**A tiled image needs seamless edges.** The swirl is tiled along the bar, so
`warp` cross-fades the outermost `SEAM` pixels of each side into one another
after the polar pass. Without it there is a hard join every tile-width and the
bar reads as a row of repeated blobs rather than one moving picture.

**egui 0.36 API notes for the panel.** `Context::screen_rect` does not exist:
it is `ctx.input(|input| input.raw.screen_rect)` and it is an `Option<Rect>`.
`Frame::popup(&ctx.style())` does not either — build the `Frame` by hand the
way `widgets::menu_frame` does. `Area::order(Order::Tooltip)` draws it over
everything.

**A parent `Sense::hover` interact does not steal clicks** — that part was
already right — but a control drawn in a `Ui::new_child` still needs its own
`interact`, and `ui.interact` on a rect outside the child's `max_rect` does
nothing. Draw the control's own response, not just the frame's.

**Virtualised tables must scroll before they draw.** `RevealSong` sets
`table.row_offset` from inside `table()` **above** `item_index`, which
borrows `table` — assigning to `row_offset` below that closure is a borrow
error, not a warning.

**Deleting a menu row leaves its settings behind.** Waveform is out of the
vis menu but still in `PlayerBarVis`, because the enum is deserialised from
`settings.json`. The drawing path maps it to Swirl rather than deleting the
variant, so an old file still loads.

## Feature 26 (no tray, Shift+Q selector, F fullscreen, one-picture swirl)

**The tray is created by a dependency and only removed on a clean exit.**
`fastframe_tray`'s Windows `Host::drop` posts `WM_QUIT`; a `taskkill //F`
skips it and Explorer keeps the icon. `AppOptions::tray` defaults to `false`
here. Close politely (`taskkill //PID n`, no `/F`) before rebuilding.

**A tiled picture reads as a pattern, not a picture.** The swirl drew a
bar-height tile repeatedly; the repeat was the ugliness. One copy stretched
across the bar, moved with `Painter::image`'s `uv: Rect`, plus a second copy a
width to the left for the wrap, reads as one image travelling. The uv rect
is in texture space (`0..1`), not screen space, so each copy samples one
bar-width further along.

**A full-window mode belongs in `ui::show`.** `fullscreen_vis` draws the
player-bar panel at `screen_rect.height()` and returns before the sidebar,
queue, lyrics and controls — one early return rather than a dozen `if`s
threaded through the layout.

**Keys live in two places.** `keys::handle` binds them; `keys::shortcuts`
lists them for the shortcuts dialog. A new key needs both, and a test that
reads the list back — that is what caught Q and Shift+Q colliding.

## Feature 27 (speed scenes: nightcore, hardtekk, on-the-fly curves)

**There is no rate control to reach for.** The pinned librespot fork has no
`set_speed` and Spotify's API has none, so speed is a resampler in our own
sink chain — `vis::Tapped`, the stage that already owns the equalizer. Add
the shared handle the way `eq` does: `SharedEq` is `Arc<Mutex<EqSettings>>`
passed through `EngineConfig`, and `SharedSpeed` is the same shape.

**The read step is the speed, not its inverse.** `step` is input samples
consumed per output sample: 1.5 means skip 1.5 input samples per output
sample, which is *faster*. The first version stored `1.0 / speed` and made
1.5x slower, and the test that caught it was the one asserting fewer samples
out of a faster run.

**Interpolate across the packet boundary, not inside it.** `carry` keeps two
samples of left context and the read position is renormalised after the
drain; a naive `for i in 0..n { out.push(samples[i * step]) }` clicks on
every packet edge. `a_packet_boundary_does_not_discontinuity` compares a
split run against a whole one and is worth keeping for any change here.

**A lock per sample is an outage.** The shared handle is read once per
`AudioPacket` in `Tapped::write`, then handed to `Resampler::set_speed`.

**Presets are `(speed, Option<eq curve name>)`.** The pairing is the feature:
"hardtekk" is a rate *and* Techno, and the test asserts every named curve
exists in `eq::PRESETS` so a rename upstream cannot silently break a scene.

## Recovering the whole set

```sh
# See what is missing after a rebase.
git diff origin/main --stat -- build.rs src/lib.rs src/build_info.rs src/ui/topbar.rs src/ui/settings.rs

# Take any file wholesale from the branch.
git checkout chanceify -- src/build_info.rs

# Or drop the whole set back on at once, then resolve by hand.
git checkout chanceify -- build.rs src/lib.rs src/ui/topbar.rs src/ui/settings.rs
```

After any of these, run the gates in `CHANCEIFY.md`. Formatting drifts easily
(`cargo fmt --all` before committing) and clippy runs with `-D warnings`, so an
unused import fails the build.