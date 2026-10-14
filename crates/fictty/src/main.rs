use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::json;
use std::path::PathBuf;
use fictty::{App, runtime};

/// A UI that is one serializable value: run a screen, then push whole UIs into it.
#[derive(Parser)]
#[command(name = "fictty")]
struct Cli {
    /// The control socket (default: $FICTTY_SOCKET, else fictty.sock in the temp dir).
    #[arg(long, global = true)]
    socket: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run a screen in this terminal (optionally starting from a UI file).
    Run {
        file: Option<PathBuf>,
        /// Reload the file whenever it changes.
        #[arg(long)]
        watch: bool,
        /// Start from a saved state (what `upgrade` hands the new binary).
        #[arg(long)]
        restore: Option<PathBuf>,
        /// Keep the store (the app's data) in this JSON file: read at the start, written after
        /// every change.
        #[arg(long)]
        store: Option<PathBuf>,
        /// Load themes from this JSON file (theme specs by name); the first is chosen.
        #[arg(long)]
        themes: Option<PathBuf>,
        /// In a native window instead of this terminal (macOS; early: a subset of the components).
        #[arg(long)]
        window: bool,
    },
    /// Replace the running screen's binary with this one (or --binary), keeping its whole state.
    Upgrade {
        #[arg(long)]
        binary: Option<PathBuf>,
    },
    /// Replace the running screen's whole UI with a file's (`-` reads stdin).
    Push { file: PathBuf },
    /// Replace one node, by id, with a node from a file (`-` reads stdin).
    Patch { id: String, file: PathBuf },
    /// Write a store entry: a JSON value from a file (`-` reads stdin). A string replaces an
    /// editor's text as one edit. With --if-rev, refused if someone wrote since that revision.
    Put {
        key: String,
        file: PathBuf,
        #[arg(long)]
        if_rev: Option<u64>,
        /// Who's writing, for the store's `by` (default `agent`).
        #[arg(long)]
        by: Option<String>,
        /// The write lasts this long (ms), then the entry goes again.
        #[arg(long)]
        for_ms: Option<u64>,
    },
    /// Merge a JSON object into a store entry's value (RFC 7396: `null` removes a key).
    Merge {
        key: String,
        file: PathBuf,
        #[arg(long)]
        if_rev: Option<u64>,
        #[arg(long)]
        by: Option<String>,
        /// The keys it sets last this long (ms), then go again.
        #[arg(long)]
        for_ms: Option<u64>,
    },
    /// Edit a text entry: a JSON array of `[from, to, "text"]` ranges in chars of its current
    /// text, in order. The person's carets move with it, and one undo takes it back.
    Edit {
        key: String,
        file: PathBuf,
        #[arg(long)]
        if_rev: Option<u64>,
        #[arg(long)]
        by: Option<String>,
    },
    /// Read a store entry: its revision, who wrote it last, and its text or value.
    Read { key: String },
    /// Choose the running screen's theme (a name, `next` or `prev`), or load themes from a file.
    Theme {
        name: Option<String>,
        #[arg(long)]
        load: Option<PathBuf>,
    },
    /// Move one node's view state, from a JSON object in a file (`-` reads stdin):
    /// `{"selected": 2}` for a list or table, `{"tab": 1}` for tabs.
    Set { id: String, file: PathBuf },
    /// Send keys to the running screen, as if typed.
    Key { keys: Vec<String> },
    /// Print the running screen's UI (or, with --state, everything: focus, selections, data).
    Get {
        #[arg(long)]
        state: bool,
        /// Only the frame and message counters.
        #[arg(long, conflicts_with = "state")]
        stats: bool,
    },
    /// The running screen's current frame, as text (or, with --nodes, the shown nodes as an
    /// outline: kind, id, focus, selection, rectangle and the text each shows).
    Screen {
        #[arg(long)]
        nodes: bool,
        /// The nodes as JSON.
        #[arg(long, requires = "nodes")]
        json: bool,
        /// Each node's style under the active theme too: its part and the states that hold, its
        /// ink, border, title, edge and backdrop, with tokens filled in.
        #[arg(long, requires = "nodes")]
        style: bool,
    },
    /// Move, click or scroll the mouse on the running screen: `move|click|up|down X Y`;
    /// `s-click` and `d-click` click with ⇧ or ⌘ held (picking rows of a `multi` list).
    Mouse { kind: String, x: u16, y: u16 },
    /// Set a terminal up for fictty: `wezterm` installs a module that sends ⌘ keys (⌘Z, ⌘A, ⌘←)
    /// to fictty screens, which WezTerm doesn't do by itself. Ghostty and kitty need nothing.
    Setup {
        terminal: String,
        /// Print the module instead of installing it.
        #[arg(long)]
        print: bool,
    },
    /// Show a message in the running screen's corner for a while.
    Toast {
        text: String,
        /// A theme token or colour for its edge: `pos`, `warn`, `neg`.
        #[arg(long)]
        tone: Option<String>,
        /// How long it shows, in milliseconds (default 3000).
        #[arg(long)]
        ms: Option<u64>,
    },
    /// Replace the running screen's layers with a JSON array from a file (`-` reads stdin).
    Layers { file: PathBuf },
    /// Render a UI file to text without a terminal.
    Render {
        file: PathBuf,
        #[arg(long, default_value = "100x30")]
        size: String,
        /// Keys to replay first, comma-separated (`j,j,tab`).
        #[arg(long, value_delimiter = ',')]
        keys: Vec<String>,
        /// Colours and attributes as ANSI (truecolour) escapes.
        #[arg(long)]
        ansi: bool,
        /// Start from this store file.
        #[arg(long)]
        store: Option<PathBuf>,
        /// Load themes from this file (the first is chosen, or --theme).
        #[arg(long)]
        themes: Option<PathBuf>,
        #[arg(long)]
        theme: Option<String>,
        /// The shown nodes as an outline instead of the frame (as `screen --nodes`).
        #[arg(long)]
        nodes: bool,
        /// With --nodes, each node's resolved style too (as `screen --nodes --style`).
        #[arg(long, requires = "nodes")]
        style: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let socket = cli.socket.unwrap_or_else(runtime::default_socket);
    let reply = match cli.cmd {
        Cmd::Run { file, watch, restore, store, themes, window } => {
            let mut app = match file {
                Some(f) => App::file(f),
                None => App::empty(),
            };
            app = app.watch(watch).socket(socket).restore(restore);
            if let Some(p) = store {
                app = app.store(p);
            }
            if let Some(p) = themes {
                app = app.themes_file(&p)?;
            }
            if window {
                return run_window(app);
            }
            return app.run();
        }
        Cmd::Upgrade { binary } => {
            let bin = binary.or_else(|| std::env::current_exe().ok()).map(|p| p.display().to_string());
            runtime::send(&socket, json!({"op": "upgrade", "binary": bin}))?
        }
        Cmd::Render { file, size, keys, ansi, store, themes, theme, nodes, style } => {
            let (w, h) = size.split_once('x').ok_or_else(|| anyhow::anyhow!("--size is WxH"))?;
            let mut app = App::new(runtime::load(&file)?);
            if let Some(p) = store {
                app = app.store(p);
            }
            if let Some(p) = themes {
                app = app.themes_file(&p)?;
            }
            if let Some(t) = theme {
                app = app.theme(t);
            }
            let mut screen = app.headless(w.parse()?, h.parse()?)?;
            let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
            screen.keys(&keys);
            print!("{}", match (nodes, style) {
                (true, true) => fictty::readback::outline_styled(&screen.readback()),
                (true, false) => fictty::readback::outline(&screen.readback()),
                _ if ansi => screen.ansi(),
                _ => screen.screen(),
            });
            return Ok(());
        }
        Cmd::Push { file } => runtime::send(&socket, json!({"op": "push", "ui": runtime::load(&file)?}))?,
        Cmd::Patch { id, file } => {
            let raw = if file.as_os_str() == "-" {
                std::io::read_to_string(std::io::stdin())?
            } else {
                std::fs::read_to_string(&file)?
            };
            let node: serde_json::Value = serde_json::from_str(&raw)?;
            runtime::send(&socket, json!({"op": "patch", "id": id, "node": node}))?
        }
        Cmd::Put { key, file, if_rev, by, for_ms } => write(&socket, "put", key, "value", &file, if_rev, by, for_ms)?,
        Cmd::Merge { key, file, if_rev, by, for_ms } => write(&socket, "merge", key, "value", &file, if_rev, by, for_ms)?,
        Cmd::Edit { key, file, if_rev, by } => write(&socket, "edit", key, "changes", &file, if_rev, by, None)?,
        Cmd::Read { key } => runtime::send(&socket, json!({"op": "read", "key": key}))?,
        Cmd::Theme { name, load } => {
            let add = match load {
                Some(p) => serde_json::to_value(runtime::load_themes(&p)?)?,
                None => json!({}),
            };
            runtime::send(&socket, json!({"op": "theme", "add": add, "choose": name}))?
        }
        Cmd::Set { id, file } => {
            let raw = if file.as_os_str() == "-" { std::io::read_to_string(std::io::stdin())? } else { std::fs::read_to_string(&file)? };
            let mut req: serde_json::Value = serde_json::from_str(&raw)?;
            req["op"] = "set".into();
            req["id"] = id.into();
            runtime::send(&socket, req)?
        }
        Cmd::Key { keys } => {
            let mut last = json!(null);
            for key in keys {
                last = runtime::send(&socket, json!({"op": "key", "key": key}))?;
            }
            last
        }
        Cmd::Screen { nodes: false, .. } => {
            let r = runtime::send(&socket, json!({"op": "screen"}))?;
            print!("{}", r["screen"].as_str().unwrap_or_default());
            return Ok(());
        }
        Cmd::Screen { nodes: true, json, style } => {
            let mut r = runtime::send(&socket, json!({"op": "readback"}))?;
            if json {
                if !style {
                    for n in r["nodes"].as_array_mut().into_iter().flatten() {
                        n.as_object_mut().map(|o| o.remove("style"));
                    }
                }
                println!("{}", serde_json::to_string_pretty(&r["nodes"])?);
            } else {
                let nodes: Vec<fictty::readback::Shown> = serde_json::from_value(r["nodes"].clone())?;
                print!("{}", if style { fictty::readback::outline_styled(&nodes) } else { fictty::readback::outline(&nodes) });
            }
            return Ok(());
        }
        Cmd::Setup { terminal, print } => {
            match (terminal.as_str(), print) {
                ("wezterm", true) => print!("{}", fictty::setup::wezterm_lua()),
                ("wezterm", false) => println!("{}", fictty::setup::wezterm()?),
                (other, _) => anyhow::bail!("nothing to set up for {other:?}: wezterm needs it; Ghostty and kitty send ⌘ keys themselves"),
            }
            return Ok(());
        }
        Cmd::Toast { text, tone, ms } => runtime::send(&socket, json!({"op": "toast", "text": text, "tone": tone, "ms": ms}))?,
        Cmd::Mouse { kind, x, y } => runtime::send(&socket, json!({"op": "mouse", "kind": kind, "x": x, "y": y}))?,
        Cmd::Layers { file } => {
            let raw = if file.as_os_str() == "-" { std::io::read_to_string(std::io::stdin())? } else { std::fs::read_to_string(&file)? };
            let layers: serde_json::Value = serde_json::from_str(&raw)?;
            runtime::send(&socket, json!({"op": "layers", "layers": layers}))?
        }
        Cmd::Get { state, stats } => {
            let op = if stats { "stats" } else if state { "state" } else { "get" };
            runtime::send(&socket, json!({"op": op}))?
        }
    };
    println!("{}", serde_json::to_string_pretty(&reply)?);
    if reply.get("ok") == Some(&json!(false)) {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn run_window(app: App) -> Result<()> {
    fictty::window::run(app)
}

#[cfg(not(target_os = "macos"))]
fn run_window(_: App) -> Result<()> {
    anyhow::bail!("the native window runs on macOS for now")
}

/// A store write: the JSON in `file` (`-` reads stdin) as the request's `field`.
#[allow(clippy::too_many_arguments)]
fn write(socket: &std::path::Path, kind: &str, key: String, field: &str, file: &PathBuf, if_rev: Option<u64>, by: Option<String>, for_ms: Option<u64>) -> Result<serde_json::Value> {
    let raw = if file.as_os_str() == "-" { std::io::read_to_string(std::io::stdin())? } else { std::fs::read_to_string(file)? };
    let body: serde_json::Value = serde_json::from_str(&raw)?;
    let mut req = json!({"op": "write", "write": kind, "key": key, field: body, "if_rev": if_rev, "for_ms": for_ms});
    if let Some(by) = by {
        req["by"] = by.into();
    }
    runtime::send(socket, req)
}
