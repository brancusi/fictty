use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::json;
use std::path::PathBuf;
use fictty::{App, runtime};

/// A terminal UI that is one serializable value: run a screen, then push whole UIs into it.
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
    },
    /// Merge a JSON object into a store entry's value (RFC 7396: `null` removes a key).
    Merge {
        key: String,
        file: PathBuf,
        #[arg(long)]
        if_rev: Option<u64>,
        #[arg(long)]
        by: Option<String>,
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
    /// The running screen's current frame, as text.
    Screen,
    /// Move, click or scroll the mouse on the running screen: `move|click|up|down X Y`.
    Mouse { kind: String, x: u16, y: u16 },
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
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let socket = cli.socket.unwrap_or_else(runtime::default_socket);
    let reply = match cli.cmd {
        Cmd::Run { file, watch, restore, store, themes } => {
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
            return app.run();
        }
        Cmd::Upgrade { binary } => {
            let bin = binary.or_else(|| std::env::current_exe().ok()).map(|p| p.display().to_string());
            runtime::send(&socket, json!({"op": "upgrade", "binary": bin}))?
        }
        Cmd::Render { file, size, keys, ansi, store, themes, theme } => {
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
            print!("{}", if ansi { screen.ansi() } else { screen.screen() });
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
        Cmd::Put { key, file, if_rev, by } => write(&socket, "put", key, "value", &file, if_rev, by)?,
        Cmd::Merge { key, file, if_rev, by } => write(&socket, "merge", key, "value", &file, if_rev, by)?,
        Cmd::Edit { key, file, if_rev, by } => write(&socket, "edit", key, "changes", &file, if_rev, by)?,
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
        Cmd::Screen => {
            let r = runtime::send(&socket, json!({"op": "screen"}))?;
            print!("{}", r["screen"].as_str().unwrap_or_default());
            return Ok(());
        }
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

/// A store write: the JSON in `file` (`-` reads stdin) as the request's `field`.
fn write(socket: &std::path::Path, kind: &str, key: String, field: &str, file: &PathBuf, if_rev: Option<u64>, by: Option<String>) -> Result<serde_json::Value> {
    let raw = if file.as_os_str() == "-" { std::io::read_to_string(std::io::stdin())? } else { std::fs::read_to_string(file)? };
    let body: serde_json::Value = serde_json::from_str(&raw)?;
    let mut req = json!({"op": "write", "write": kind, "key": key, field: body, "if_rev": if_rev});
    if let Some(by) = by {
        req["by"] = by.into();
    }
    runtime::send(socket, req)
}
