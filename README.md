# DeepLens

DeepLens is a native Rust desktop command palette for searching local files,
folders, and applications. It is built for people who want a Spotlight,
Alfred, Raycast, or Windows Search style launcher, but with stronger local file
content search and no Electron, Tauri, cloud service, index database, or AI
layer.

DeepLens searches on demand with command-line tools you control:

- file contents with [`ripgrep-all`](https://github.com/phiresky/ripgrep-all)
  (`rga`)
- folder names with [`fd`](https://github.com/sharkdp/fd)
- fast folder scope switching with [`zoxide`](https://github.com/ajeetdsouza/zoxide)
- installed macOS applications by scanning `.app` bundles

## Why Use It

System search tools are good launchers, but they can be opaque, index-dependent,
or weak at searching inside PDFs, documents, archives, and code. DeepLens is
designed for a narrower job: quickly search a chosen local scope, see useful
matches as they stream in, and open the right file, folder, or app.

Use DeepLens when you want:

- a compact keyboard-first launcher for local files and apps
- content search across PDFs, office documents, text files, code, and other
  formats supported by `ripgrep-all`
- visible matched snippets instead of only filenames
- explicit folder scope selection instead of relying on a global index
- fast switching between project folders using `/project` with `zoxide`
- session-local recent searches and no persistent search index
- a native Rust desktop app with a small footprint

DeepLens is not a semantic search engine, OCR indexer, cloud sync tool, or
replacement for a full document management system.

## Capabilities

- Search file contents, folder names, apps, or all three at once.
- Stream results progressively while the search is running.
- Cancel stale searches while typing.
- Group multiple matches from the same file.
- Show `+N more matches` for grouped duplicates.
- Rank results by rough filename, path, content, exact phrase, recency, and
  noisy-path signals.
- Open selected results with the system default application.
- Reveal selected results in Finder on macOS.
- Copy selected paths.
- Open the selected result folder in a configured terminal.
- Use Spotlight-style ghost completion for selected app and path results.
- Hide automatically when focus moves to another app.
- Persist settings in `~/.deeplens/settings.json`.

## Platform Status

DeepLens is currently developed on macOS.

Core file and folder search is based on cross-platform command-line tools, but
some desktop integrations are macOS-oriented today:

- app search scans macOS `.app` bundles
- reveal uses Finder on macOS
- shortcut labels use `Cmd`
- application icons are loaded from macOS app bundles

Linux and Windows support may require adapting app discovery, reveal behavior,
terminal launch behavior, and shortcut labels.

## Requirements

- Rust toolchain
- `rga` from `ripgrep-all`
- `fd`
- `zoxide`
- `numbat` for calculator and unit conversion results

Install dependencies on macOS:

```sh
brew install ripgrep-all fd zoxide
cargo install numbat-cli
```

For a source checkout, these tools must be available on `PATH`. A packaged
release can avoid this requirement by bundling the helper binaries in the app
and resolving them from the app resources directory instead of `PATH`.

## Installation

Clone and install from source:

```sh
git clone <repo-url>
cd deeplens
./install.sh
```

The install script:

- installs missing helper tools with Homebrew: `ripgrep-all`, `fd`, `zoxide`
- builds DeepLens with `cargo build --release`
- installs the `DeepLens` binary to `~/.local/bin`
- creates `DeepLens.app` in `/Applications` when writable, otherwise in
  `~/Applications`

To install to another prefix:

```sh
DEEPLENS_INSTALL_PREFIX=/usr/local ./install.sh
```

To choose the app install directory:

```sh
DEEPLENS_APP_DIR=/Applications ./install.sh
DEEPLENS_APP_DIR="$HOME/Applications" ./install.sh
```

Run without installing:

```sh
cargo run
```

For a local release build:

```sh
cargo build --release
```

The binary is created at:

```sh
target/release/DeepLens
```

macOS may require accessibility/input permissions before global shortcuts are
delivered to apps launched from Terminal.

## Usage

Open DeepLens with the global shortcut, type a query, and use the highlighted
result.

Default scope is `/`. Use the `+` button to choose a folder, or type a
zoxide-powered scope command:

```text
/project invoice
```

This resolves `project` with `zoxide`, switches scope, and searches for
`invoice`.

Search starts after the minimum query length is reached and after the debounce
delay. Both are configurable.

## Search Modes

- `All`: search apps, folders, and file contents
- `Apps`: search installed macOS applications
- `Folders`: search folder names with `fd`
- `Files`: search file contents with `rga`

## Query Syntax

```text
"<exact search>"       exact phrase search
pdf <file>             switch to Files and search PDF files
doc <file>             switch to Files and search document-like files
txt <file>             switch to Files and search text-like files
code <file>            switch to Files and search code-like files
/all <file>            search apps, folders, and file contents
/apps <file>           search installed macOS applications
/folders <folder>      search folder names only
/files <file>          search file contents only
2 + 2                  calculate with Numbat
=30 km/h -> mph        calculate or convert units with Numbat
calc 5 ft + 2 in -> cm calculate explicitly with Numbat
g <query>              search Google
gh <query>             search GitHub
yt <query>             search YouTube
docs <query>           search documentation on the web
/                      search from filesystem root
/<folder> <file>       resolve folder with zoxide and search there
/set <name> <value>    update a setting
```

Web shortcuts are configurable with `web_search_shortcuts`. Entries use
`name=https://example.com/search?q={query}` and are separated by commas. Slash
forms such as `/g <query>` also work, but bare prefixes are the default style.

Calculator results use [`numbat`](https://github.com/sharkdp/numbat). Expression-like
input is detected automatically; set `calculator_requires_prefix` to `true` if
you only want calculations for input starting with `=` or `calc`.

## Shortcuts

Default shortcuts:

| Shortcut | Action |
| --- | --- |
| `Cmd+Shift+Space` | Show or hide DeepLens |
| `Cmd+,` | Open settings |
| `Enter` | Open selected result |
| `Cmd+K` | Open selected result actions |
| `Space` | Preview selected result with Quick Look |
| `Cmd+Enter` | Reveal selected result in Finder |
| `Cmd+C` | Copy selected path |
| `Cmd+T` | Open selected result folder in terminal |
| `Up` / `Down` | Move selection |
| `Tab` / `Right Arrow` | Accept ghost completion |
| `Esc` | Cancel search, clear query, or hide window |

Shortcuts are configurable in Settings or directly in
`~/.deeplens/settings.json`.

Shortcut syntax examples:

```text
cmd+t
cmd+enter
cmd+shift+space
tab,right
```

## Settings

Open settings with `Cmd+,`.

Settings are grouped into:

- `General`: result limits, debounce, minimum query length, window sizes,
  result row height, terminal app, excluded folders, broad-search limits,
  web search shortcuts, and recents/history ranking
- `Style`: colors for the palette, text, chips, selected rows, and highlights
- `Shortcuts`: global and in-app shortcuts

Settings are persisted at:

```text
~/.deeplens/settings.json
```

Recent-result history is stored separately at:

```text
~/.deeplens/history.json
```

You can also update individual settings from the search box:

```text
/set result_row_height 104
/set terminal_app iTerm
/set excluded_folders Library, .Trash, node_modules, target, .git
/set max_search_events_per_tick 160
/set search_event_limit_multiplier 20
/set history_enabled true
/set max_history_items 500
/set history_recency_boost 300
/set web_search_shortcuts g=https://www.google.com/search?q={query}, gh=https://github.com/search?q={query}
/set calculator_enabled true
/set calculator_requires_prefix false
/set preview_enabled true
/set preview_shortcut space
/set actions_enabled true
/set actions_shortcut cmd+k
/set global_shortcut cmd+shift+space
```

## Result Behavior

- Single click selects a result.
- Double click opens a result.
- `Enter` opens the selected result.
- The first visible result is highlighted automatically for each search.
- Multiple matches in the same file are grouped.
- Opened files, folders, and apps are remembered in `~/.deeplens/history.json`
  and boosted in future searches. History is not shown as a separate results
  screen yet.
- The visible result list is capped by `max_displayed_results`.
- Broad searches are bounded by `max_search_events_per_tick` and
  `search_event_limit_multiplier`.
- Invalid JSON lines from `rga` are ignored.

## Performance

DeepLens does not keep a background index. It searches the selected scope on
demand and streams results into the UI.

To keep searches responsive, DeepLens:

- debounces typing
- cancels old searches when the query changes
- kills the full Unix process group on cancel, including child tools
- treats `/` as a safe computer scope: app search stays global, while file and
  folder search use the user's home folder instead of recursively walking the
  literal filesystem root
- skips configured heavy folders such as caches, `.Trash`, `.git`,
  `node_modules`, and `target`
- limits large files passed to `rga`
- processes streamed matches in bounded UI batches, so broad queries do not
  freeze the window
- caps grouped visible results

The skipped folder list is configurable in Settings as `excluded_folders`. It
is a comma-separated list used by both file-content search and folder-name
search.

Default:

```text
Library, Library/Caches, Library/Developer, .Trash, node_modules, target, .git
```

## Packaging Dependencies

DeepLens looks for `rga`, `fd`, and `zoxide` on `PATH`. The installed macOS app
also prepends common tool locations such as `/opt/homebrew/bin` and
`/usr/local/bin`, because apps opened from Finder or the Dock do not inherit
your shell PATH.

For a user-friendly desktop release, there are three practical options:

- Bundle helper binaries inside the app. Put `rga`, `fd`, and optionally
  `zoxide` in the app bundle, then make DeepLens resolve them relative to the
  executable. This gives users a single app download and is the best long-term
  desktop experience.
- Publish a Homebrew formula or cask. Declare `ripgrep-all`, `fd`, and `zoxide`
  as dependencies. This is easiest for developer users and avoids shipping
  third-party binaries yourself.
- Add an in-app dependency installer. On first launch, detect missing tools and
  offer to install them through Homebrew. This is convenient, but less reliable
  than bundling and requires clear user consent.

Bundling has one important caveat: `ripgrep-all` can use additional converters
for some file types. If you want full PDF/document/archive support without any
external installs, bundle and test the exact converter set you want to support.
`zoxide` can be bundled for querying, but users still need a zoxide database
for `/project` scope shortcuts to be useful.

## Troubleshooting

`ripgrep-all (rga) was not found`

Install it and restart DeepLens:

```sh
brew install ripgrep-all
```

Folder search does not work

Install `fd`:

```sh
brew install fd
```

`/project` scope switching does not work

Install and initialize `zoxide`, then make sure the folder has been visited
before:

```sh
brew install zoxide
```

Global shortcut does not work on macOS

Check System Settings permissions for the app or the terminal used to launch
it. If another app owns the same shortcut, change `global_shortcut` in Settings.

Terminal shortcut opens the wrong terminal

Set the terminal app name:

```text
/set terminal_app iTerm
```

## Roadmap

DeepLens already covers the core local-search workflow. The next work should
focus on the small features that make Spotlight, Alfred, and Raycast feel fast
in daily use.

### Top Priority

- [x] **Better app launching**: improve app ranking, aliases, and matching so
  common apps open reliably from a few letters.
- [x] **Fuzzy matching**: tolerate small typos, missing separators, and partial
  word matches without making results noisy.
- [x] **Recent results**: remember frequently opened files, folders, and apps so
  repeated work is faster than a fresh search.
- [ ] **Pinned results**: let users keep important files, folders, and apps at
  the top for matching searches.
- [x] **Quick preview**: preview the selected file with a shortcut before opening
  it, similar to Quick Look.
- [x] **Actions menu**: add a small action picker for selected results, such as
  open, reveal, copy path, copy filename, open terminal here, and show info.
- [ ] **Cleaner mode controls**: replace extra chips with one simple search mode
  control and keep advanced filters in query syntax.
- [ ] **Clipboard history**: search and paste recently copied text.
- [x] **Calculator and unit conversion**: evaluate quick math and conversions
  directly from the search box.
- [x] **Web search shortcuts**: open configured searches such as Google, GitHub,
  YouTube, or documentation with a short prefix.
- [ ] **Custom commands**: run user-defined commands or scripts with arguments.

### Later

- [ ] **Polished packaging**: ship a signed macOS app, Homebrew cask, bundled
  icon, and clearer setup so users do not have to think about helper tools.
- [ ] Optional lightweight indexing for faster repeated searches.
- [ ] Linux and Windows desktop integration.
- [ ] OCR and semantic search as optional features, not part of the default core.

## Development

Validate the project with:

```sh
cargo fmt
cargo check
cargo test
```

The app entry point is `src/main.rs`; most UI behavior lives in `src/app.rs`
and supporting modules under `src/app/`.
