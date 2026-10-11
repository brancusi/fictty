//! The screen's whole state, the messages that change it and a pure `update`. The runtime owns
//! the terminal, the socket and child processes; it feeds messages in and carries effects out.
//! `State` serializes, so `fictty get --state` shows exactly what is drawn.

use crate::model::{Action, Kind, LayerSpec, Node, ThemeSpec, Ui, lookup};
use crate::store::{Body, Entry, Write};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub ui: Ui,
    /// Per-node view state, by node id. Survives a push for every id still in the new tree.
    #[serde(default)]
    pub local: BTreeMap<String, Local>,
    #[serde(default)]
    pub focus: Option<String>,
    /// The last value fetched for each source.
    #[serde(default)]
    pub data: BTreeMap<String, Slot>,
    /// The app's data: written by the person and by agents, never by a push (see `store`).
    #[serde(default)]
    pub store: BTreeMap<String, Entry>,
    /// Writes to the store so far, so the runtime knows when to save it.
    #[serde(default)]
    pub store_writes: u64,
    /// The themes the person or project loaded (`--themes`), by name. A push doesn't change them.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub themes: BTreeMap<String, ThemeSpec>,
    /// The active theme's name: the person's choice, like a selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// What's wrong with the UI against the store and the theme, in sentences an agent can act
    /// on: a bound value of the wrong shape, an unknown variant or tone. Checked after every
    /// message; the status line shows the first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub problems: Vec<String>,
    /// One line at the bottom: the last error or message.
    #[serde(default)]
    pub status: Option<String>,
    /// How many UIs have been pushed (the first load counts).
    #[serde(default)]
    pub version: u64,
    /// The runtime's measurements, also bindable as the `$stats` source.
    #[serde(default)]
    pub stats: Stats,
    /// What the mouse is over.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hover: Option<Hover>,
    /// The logical clock (ms), moved only by `Msg::Tick`; animations read it.
    #[serde(default)]
    pub now_ms: u64,
    /// What the terminal can draw (pictures, the cell size in pixels, larger text).
    #[serde(default)]
    pub caps: crate::gfx::Caps,
    /// Whether larger text (OSC 66) is on: the terminal's to say, or `FICTTY_TEXT_SIZE=1`.
    #[serde(default)]
    pub text_sizing: bool,
    /// The rows a node shows when they aren't its bound array as it is: a tree's open rows in
    /// order, a filtered list's matches. Worked out from the rest after each message (so never
    /// saved); `items` reads them.
    #[serde(skip)]
    pub rows: BTreeMap<String, Vec<Value>>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Hover {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row: Option<usize>,
}

/// Frames and messages over the last measuring window (the runtime reports twice a second).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    /// Frames drawn per second.
    pub fps: f64,
    /// Mean time to build and draw one frame, in milliseconds.
    pub draw_ms: f64,
    /// Slowest frame in the window, in milliseconds.
    pub max_ms: f64,
    /// Messages applied per second (keys, data, pushes, patches).
    pub msgs: f64,
    /// Frames drawn since start.
    pub frames: u64,
}

/// Sources whose names start with `$` are the runtime's own (`$stats`); a push keeps them.
pub fn builtin(name: &str) -> bool {
    name.starts_with('$')
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Local {
    pub selected: usize,
    /// The shown child of a tabs node.
    #[serde(default)]
    pub tab: usize,
    /// An editor's or input's views of the text entries it has shown, by entry: carets,
    /// scroll, size. The text is in the store.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub editors: BTreeMap<String, caretline::View>,
    /// A tree's rows (by key) folded or opened against its default.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub toggled: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum Slot {
    Loading,
    Ready { value: Value },
    Error { message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "msg", rename_all = "snake_case")]
pub enum Msg {
    /// Replace the whole UI.
    Push { ui: Ui },
    /// Replace one node by id.
    Patch { id: String, node: Node },
    /// A key, by name (`j`, `tab`, `s-tab`, `c-c`, `enter`).
    Key { key: String },
    /// A source finished fetching.
    Data { name: String, result: Result<Value, String> },
    /// A `run` finished.
    Ran { argv: Vec<String>, result: Result<(), String> },
    /// The host says its source `host` changed: the sources that read it fetch again.
    Changed { host: String },
    /// A host action finished, at the local clock time (`HH:MM:SS`, from the runtime).
    Acted { name: String, result: Result<Value, String>, at: String },
    /// The runtime's measurements.
    Stats { stats: Stats },
    /// A click on a node (a row of it, or a tab title).
    Click { id: String, row: Option<usize>, tab: Option<usize> },
    /// The mouse moved over a node (a row of it), or off everything.
    Hover { hover: Option<Hover> },
    /// The wheel over a node: rows to move (negative is up).
    Scroll { id: String, by: isize },
    /// A click inside an editor's or input's text, at a cell relative to it: focus it and put the
    /// caret there.
    Point { id: String, col: u16, row: u16 },
    /// Animation time.
    Tick { now_ms: u64 },
    /// Move a node's view state from outside: a list's or table's `selected` row, a tabs
    /// node's `tab`.
    Set {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        selected: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tab: Option<usize>,
    },
    /// Write one store entry. With `if_rev`, refused unless the entry is still at that
    /// revision (someone else wrote in between). The answer comes back as `Effect::Reply`.
    Write {
        key: String,
        #[serde(flatten)]
        write: Write,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        if_rev: Option<u64>,
        by: String,
    },
    /// Add or replace themes (by name), and choose one: a name, or `next` / `prev` to cycle.
    Theme {
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        add: BTreeMap<String, ThemeSpec>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        choose: Option<String>,
    },
    /// Something an agent did over the socket, for the `$log` source (newest first), with the
    /// local clock time it happened at (`HH:MM:SS`, from the runtime).
    Activity { op: String, what: String, at: String },
    /// The cells an editor node got in the last frame (the view reports them; the runtime sends
    /// this when they differ from what the editor has).
    Size { id: String, width: u16, height: u16 },
    /// Replace the layers only.
    Layers { layers: Vec<LayerSpec> },
    /// What the terminal can draw.
    Caps { caps: crate::gfx::Caps, text_sizing: bool },
    /// Where this screen runs: its binary, its socket, the folder its UI came from. Bindable as
    /// `$self`, with `cli` (a command line that reaches this screen from any terminal), `env` (a
    /// line that sets a terminal up so `fictty`, and the scripts beside the UI, reach it) and `dir`.
    Host { bin: String, socket: String, dir: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    /// Fetch a source (the runtime reads its definition from `ui.data`).
    Fetch(String),
    Run(Vec<String>),
    Load(String),
    Quit,
    /// Put this text on the system clipboard.
    Copy(String),
    /// The answer to the request that sent the message (a write: its new revision, or why not).
    Reply(Value),
    /// A host action, its args filled.
    Host { name: String, args: Value },
}

pub fn update(st: &mut State, msg: Msg) -> Vec<Effect> {
    let checks = matches!(msg, Msg::Push { .. } | Msg::Patch { .. } | Msg::Data { .. } | Msg::Write { .. } | Msg::Theme { .. } | Msg::Key { .. });
    let writes = st.store_writes;
    let quiet = matches!(msg, Msg::Tick { .. } | Msg::Hover { .. } | Msg::Stats { .. } | Msg::Activity { .. } | Msg::Caps { .. } | Msg::Size { .. });
    let fx = apply(st, msg);
    if !quiet {
        derive_rows(st);
        settle(st);
    }
    // A selection, a write or a new row can point an editor at an entry it hasn't shown yet.
    fields(st);
    if checks {
        st.problems = problems(st);
    }
    if st.store_writes != writes || !st.data.contains_key("$store") {
        let value = store_summary(st);
        st.data.insert("$store".into(), Slot::Ready { value });
    }
    fx
}

fn apply(st: &mut State, msg: Msg) -> Vec<Effect> {
    match msg {
        Msg::Push { ui } => {
            st.ui = ui;
            st.version += 1;
            st.status = None;
            // Seeds start entries that don't exist yet; the store is never cleared by a push.
            for (k, v) in &st.ui.store {
                if !st.store.contains_key(k) {
                    st.store.insert(k.clone(), Entry::value(v.clone(), "seed"));
                    st.store_writes += 1;
                }
            }
            settle(st);
            // Sources can change with the UI; fetch them all again.
            st.data.retain(|k, _| builtin(k) || st.ui.data.contains_key(k));
            st.ui.data.keys().map(|k| Effect::Fetch(k.clone())).collect()
        }
        Msg::Patch { id, node } => {
            if st.ui.root.replace(&id, &node) {
                st.status = None;
                settle(st);
            } else {
                st.status = Some(format!("patch: no node with id {id:?}"));
            }
            vec![]
        }
        Msg::Data { name, result } => {
            if st.ui.data.contains_key(&name) {
                let slot = match result {
                    Ok(value) => Slot::Ready { value },
                    Err(message) => Slot::Error { message },
                };
                st.data.insert(name, slot);
                settle(st);
            }
            vec![]
        }
        Msg::Stats { stats } => {
            // Rounded for showing: `$stats` is for templates.
            let r = |x: f64, d: i32| (x * 10f64.powi(d)).round() / 10f64.powi(d);
            let value = serde_json::json!({"fps": r(stats.fps, 1), "draw_ms": r(stats.draw_ms, 2), "max_ms": r(stats.max_ms, 1),
                                           "msgs": r(stats.msgs, 0), "frames": stats.frames});
            st.data.insert("$stats".into(), Slot::Ready { value });
            st.stats = stats;
            vec![]
        }
        Msg::Ran { argv, result } => {
            st.status = result.err().map(|e| format!("{}: {e}", argv.join(" ")));
            refresh(st)
        }
        Msg::Changed { host } => st.ui.data.iter().filter(|(_, s)| s.host.as_deref() == Some(&host)).map(|(k, _)| Effect::Fetch(k.clone())).collect(),
        Msg::Acted { name, result, at } => {
            match result {
                Ok(_) => log(st, "host".into(), name, at),
                Err(e) => {
                    st.status = Some(format!("{name}: {e}"));
                    log(st, "refused".into(), format!("{name} ({e})"), at);
                }
            }
            // What the host did may have changed what its sources say.
            st.ui.data.iter().filter(|(_, s)| s.host.is_some()).map(|(k, _)| Effect::Fetch(k.clone())).collect()
        }
        Msg::Click { id, row, tab } => {
            let Some(n) = st.ui.root.find(&id) else { return vec![] };
            let click = n.click.clone();
            if n.focusable() {
                st.focus = Some(id.clone());
            }
            let local = st.local.entry(id).or_default();
            if let Some(r) = row {
                local.selected = r;
            }
            if let Some(t) = tab {
                local.tab = t;
                settle(st);
            }
            match click {
                Some(a) => act(st, a),
                None => vec![],
            }
        }
        Msg::Hover { hover } => {
            st.hover = hover;
            vec![]
        }
        Msg::Point { id, col, row } => {
            if st.ui.root.find(&id).is_some_and(|n| n.field().is_some()) {
                st.focus = Some(id.clone());
                on_doc(st, &id, caretline::Msg::Click { col, row, extend: false }, "person");
            }
            vec![]
        }
        Msg::Scroll { id, by } if st.ui.root.find(&id).is_some_and(|n| n.field().is_some()) => {
            on_doc(st, &id, caretline::Msg::Scroll { rows: by as i32 * 3 }, "person");
            vec![]
        }
        Msg::Scroll { id, by } => {
            if st.ui.root.find(&id).is_some_and(Node::focusable) {
                st.focus = Some(id);
                step(st, |sel, len| sel.saturating_add_signed(by).min(len.saturating_sub(1)))
            } else {
                vec![]
            }
        }
        Msg::Tick { now_ms } => {
            st.now_ms = now_ms;
            vec![]
        }
        Msg::Set { id, selected, tab } => {
            if st.ui.root.find(&id).is_none() {
                st.status = Some(format!("set: no node with id {id:?}"));
                return vec![];
            }
            let local = st.local.entry(id).or_default();
            if let Some(r) = selected {
                local.selected = r;
            }
            if let Some(t) = tab {
                local.tab = t;
            }
            settle(st);
            vec![]
        }
        Msg::Write { key, write, if_rev, by } => {
            let reply = match write_entry(st, &key, write, if_rev, &by) {
                Ok(()) => serde_json::json!({"ok": true, "key": key, "rev": st.store[&key].rev}),
                Err(status) => {
                    let mut r = serde_json::json!({"ok": false, "status": status});
                    if let Some(e) = st.store.get(&key) {
                        r["current"] = e.read();
                    }
                    r
                }
            };
            settle(st);
            vec![Effect::Reply(reply)]
        }
        Msg::Theme { add, choose } => {
            let first = add.keys().next().cloned();
            st.themes.extend(add);
            if let Some(c) = choose.or(if st.theme.is_none() { first } else { None }) {
                choose_theme(st, &c);
            }
            vec![]
        }
        Msg::Activity { op, what, at } => {
            log(st, op, what, at);
            vec![]
        }
        Msg::Size { id, width, height } => {
            on_doc(st, &id, caretline::Msg::resize(width.max(1), height.max(1)), "person");
            vec![]
        }
        Msg::Layers { layers } => {
            st.ui.layers = layers;
            vec![]
        }
        Msg::Caps { caps, text_sizing } => {
            st.caps = caps;
            st.text_sizing = text_sizing;
            // Bindable as `$caps`, so a UI can say what it's drawing with.
            let mut value = serde_json::to_value(caps).unwrap_or_default();
            value["text_sizing"] = text_sizing.into();
            value["cell_w"] = (caps.cell_w.round() as i64).into();
            value["cell_h"] = (caps.cell_h.round() as i64).into();
            st.data.insert("$caps".into(), Slot::Ready { value });
            vec![]
        }
        Msg::Host { bin, socket, dir } => {
            let q = |s: &str| format!("'{}'", s.replace('\'', r"'\''"));
            let bin_dir = std::path::Path::new(&bin).parent().map(|p| p.display().to_string()).unwrap_or_default();
            let value = serde_json::json!({
                "bin": bin, "socket": socket, "dir": q(&dir),
                "cli": format!("FICTTY_SOCKET={} {}", q(&socket), q(&bin)),
                "env": format!("export PATH={}:{}:\"$PATH\" FICTTY_SOCKET={}", q(&bin_dir), q(&dir), q(&socket)),
            });
            st.data.insert("$self".into(), Slot::Ready { value });
            vec![]
        }
        Msg::Key { key } => {
            if let Some(why) = not_a_number(st, &key) {
                st.status = Some(why);
                return vec![];
            }
            if let Some(id) = editing(st, &key) {
                type_into(st, &id, &key);
                return vec![];
            }
            match resolve(st, &key) {
                Some(a) => act(st, a),
                None => vec![],
            }
        }
    }
}

/// What a store action's templates read: the selected row's fields, `store` (every entry: a
/// value, or a text's contents) and `$id`, an id no write has used (the sum of all revisions only
/// goes up, and survives a restart with the store).
fn action_context(st: &State) -> Value {
    let mut ctx = match selected_item(st) {
        Some(Value::Object(m)) => Value::Object(m),
        _ => Value::Object(Default::default()),
    };
    let store: serde_json::Map<String, Value> = st
        .store
        .iter()
        .map(|(k, e)| {
            let v = match &e.body {
                Body::Value { value } => value.clone(),
                Body::Text { doc } => doc.text.to_string().into(),
            };
            (k.clone(), v)
        })
        .collect();
    ctx["store"] = Value::Object(store);
    let next: u64 = st.store.values().map(|e| e.rev).sum::<u64>() + 1;
    ctx["$id"] = format!("{next:x}").into();
    if let Some(Slot::Ready { value }) = st.data.get("$self") {
        ctx["$self"] = value.clone();
    }
    ctx
}

/// A store action's `value` filled from `ctx`: strings are templates. A string that is one
/// placeholder takes the value itself, so a number stays a number and a list a list; text is
/// never guessed into a number (components that add up read numbers from strings).
fn fill_value(v: &Value, ctx: &Value) -> Value {
    match v {
        Value::String(t) if t.contains('{') => {
            let one = t.strip_prefix('{').and_then(|r| r.strip_suffix('}')).filter(|p| !p.contains(['{', '}', '|']));
            if let Some(found) = one.and_then(|p| lookup(ctx, p)) {
                return found.clone();
            }
            Value::String(crate::model::fill(t, ctx))
        }
        Value::Array(a) => Value::Array(a.iter().map(|x| fill_value(x, ctx)).collect()),
        Value::Object(m) => Value::Object(m.iter().map(|(k, x)| (k.clone(), fill_value(x, ctx))).collect()),
        other => other.clone(),
    }
}

/// Make a theme active: a name, or `next` / `prev` through the loaded ones in name order.
fn choose_theme(st: &mut State, choice: &str) {
    let names: Vec<&String> = st.themes.keys().collect();
    let at = st.theme.as_ref().and_then(|t| names.iter().position(|n| *n == t));
    let pick = match choice {
        "next" | "prev" if names.is_empty() => None,
        "next" => Some(names[at.map_or(0, |i| (i + 1) % names.len())].clone()),
        "prev" => Some(names[at.map_or(0, |i| (i + names.len() - 1) % names.len())].clone()),
        name if st.themes.contains_key(name) => Some(name.to_string()),
        name => {
            st.status = Some(format!("theme: no theme named {name:?} (loaded: {})", names.iter().map(|n| n.as_str()).collect::<Vec<_>>().join(", ")));
            None
        }
    };
    if pick.is_some() {
        st.theme = pick;
    }
    // Bindable as `$theme`: which one is on, and what else there is.
    let value = serde_json::json!({"name": st.theme, "names": st.themes.keys().collect::<Vec<_>>()});
    st.data.insert("$theme".into(), Slot::Ready { value });
}

/// How many entries `$log` keeps.
const LOG_LEN: usize = 40;

/// Adds to `$log`: `{at, op, what}`, newest first, `at` as minutes and seconds on the clock.
/// Keys in a row join one entry, so an agent typing a sentence reads as the sentence.
fn log(st: &mut State, op: String, what: String, at: String) {
    let mut entries = match st.data.remove("$log") {
        Some(Slot::Ready { value: Value::Array(a) }) => a,
        _ => Vec::new(),
    };
    let typed = |k: &str| match k {
        "space" => " ".to_string(),
        "enter" => "⏎".to_string(),
        k if k.chars().count() == 1 => k.to_string(),
        k => format!("‹{k}›"),
    };
    match entries.first_mut() {
        Some(last) if op == "key" && last["op"] == "key" => {
            let joined = format!("{}{}", last["what"].as_str().unwrap_or(""), typed(&what));
            let n = joined.chars().count();
            last["what"] = joined.chars().skip(n.saturating_sub(48)).collect::<String>().into();
            last["at"] = at.into();
        }
        _ => {
            let what = if op == "key" { typed(&what) } else { what };
            entries.insert(0, serde_json::json!({"at": at, "op": op, "what": what}));
            entries.truncate(LOG_LEN);
        }
    }
    st.data.insert("$log".into(), Slot::Ready { value: Value::Array(entries) });
}

/// Why the focused number field's own `key` can't run yet: its text isn't a number.
fn not_a_number(st: &State, key: &str) -> Option<String> {
    let n = st.ui.root.find(st.focus.as_ref()?)?;
    if !n.field()?.number || !n.keys.contains_key(key) {
        return None;
    }
    let (doc_key, _) = field_view(st, n.id.as_ref()?)?;
    let text = match &st.store.get(&doc_key)?.body {
        Body::Text { doc } => doc.text.to_string(),
        Body::Value { .. } => return None,
    };
    let name = n.title.as_deref().or(n.id.as_deref()).unwrap_or("input");
    (!crate::model::number_done(&text)).then(|| format!("{name}: {text:?} isn't a number"))
}

/// The focused editor or input that takes this key, if any: everything but Tab, Shift-Tab, Esc
/// and the node's own `keys`. Ctrl-C copies there; quitting is from outside.
fn editing(st: &State, key: &str) -> Option<String> {
    let id = st.focus.as_ref()?;
    let n = st.ui.root.find(id)?;
    let taken = n.field().is_some() && !n.keys.contains_key(key) && !matches!(key, "tab" | "s-tab" | "esc");
    taken.then(|| id.clone())
}

/// A key into an editor: the clock first (typing groups into undo steps by time), then the key
/// through caretline's keymap. Its effects (save, quit, the system clipboard) are the host's
/// and fictty has none to give, so they're dropped; copy and paste work inside the editor.
fn type_into(st: &mut State, id: &str, key: &str) {
    let Some(msg) = editor_key(key).and_then(|k| caretline::keymap(&k)) else { return };
    let now_ms = st.now_ms;
    on_doc(st, id, caretline::Msg::Tick { now_ms }, "person");
    on_doc(st, id, msg, "person");
}

/// The store entry an editor or input shows: its `doc` template over the bound value, else its
/// `bind`, else its id. None while a `doc` has nothing to fill from (no row selected).
pub fn doc_key(st: &State, n: &Node) -> Option<String> {
    let f = n.field()?;
    match (f.doc, n.bind.as_deref()) {
        (Some(doc), Some(bind)) => Some(crate::model::fill(doc, value_of(st, bind)?)),
        (Some(doc), None) => Some(doc.to_string()),
        (None, Some(bind)) if !bind.starts_with('@') => Some(bind.to_string()),
        _ => n.id.clone(),
    }
}

/// The view editor `id` has of the entry it shows now.
pub fn field_view<'a>(st: &'a State, id: &str) -> Option<(String, &'a caretline::View)> {
    let key = doc_key(st, st.ui.root.find(id)?)?;
    let v = st.local.get(id)?.editors.get(&key)?;
    Some((key, v))
}

/// Every editor showing `key`, with its view of it taken out of the state (put back by
/// `put_views`).
fn take_views(st: &mut State, key: &str) -> Vec<(String, caretline::View)> {
    let mut all = Vec::new();
    st.ui.root.walk(&mut all);
    let ids: Vec<String> =
        all.iter().filter(|n| n.field().is_some() && doc_key(st, n).as_deref() == Some(key)).filter_map(|n| n.id.clone()).collect();
    ids.into_iter().filter_map(|id| Some((id.clone(), st.local.get_mut(&id)?.editors.remove(key)?))).collect()
}

fn put_views(st: &mut State, key: &str, views: Vec<(String, caretline::View)>) {
    for (id, v) in views {
        st.local.entry(id).or_default().editors.insert(key.to_string(), v);
    }
}

/// A caretline message through editor `id`'s view of its text. Every other editor on the same
/// text follows the change. A change of text is a store write, by `by`.
fn on_doc(st: &mut State, id: &str, msg: caretline::Msg, by: &str) {
    let Some(node) = st.ui.root.find(id) else { return };
    let number = node.field().is_some_and(|f| f.number);
    let Some(key) = doc_key(st, node) else { return };
    let mut views = take_views(st, &key);
    let acting = views.iter().position(|(v, _)| v == id);
    if let (Some(acting), Some(Entry { body: Body::Text { doc }, .. })) = (acting, st.store.get_mut(&key)) {
        let before = doc.rev;
        let mut vs: Vec<caretline::View> = views.iter().map(|(_, v)| v.clone()).collect();
        // A number field keeps what it had when an edit would make it something else.
        let undo = number.then(|| (doc.clone(), vs.clone()));
        caretline::views::update_doc(doc, &mut vs, acting, msg);
        if let Some((d, v)) = undo
            && !crate::model::number_so_far(&doc.text.to_string())
        {
            *doc = d;
            vs = v;
        }
        for ((_, v), new) in views.iter_mut().zip(vs) {
            *v = new;
        }
        if doc.rev != before {
            wrote(st, &key, by);
        }
    }
    put_views(st, &key, views);
}

/// Count a write to `key`: its revision goes up, and it records who.
fn wrote(st: &mut State, key: &str, by: &str) {
    if let Some(e) = st.store.get_mut(key) {
        e.rev += 1;
        e.by = by.into();
    }
    st.store_writes += 1;
}

/// A write from outside (an agent, over the socket). Text is edited, never swapped out: the
/// person's carets and undo history carry on through it, and one undo takes it back.
fn write_entry(st: &mut State, key: &str, write: Write, if_rev: Option<u64>, by: &str) -> Result<(), String> {
    if let (Some(want), Some(e)) = (if_rev, st.store.get(key))
        && e.rev != want
    {
        return Err(format!("stale: {key} is at rev {}, not {want}; read it and try again", e.rev));
    }
    if if_rev.is_some_and(|r| r > 0) && !st.store.contains_key(key) {
        return Err(format!("stale: {key} doesn't exist"));
    }
    let is_text = matches!(st.store.get(key), Some(Entry { body: Body::Text { .. }, .. }));
    match write {
        Write::Put { value } if !is_text => {
            match st.store.get_mut(key) {
                Some(e) => e.body = Body::Value { value },
                None => {
                    st.store.insert(key.into(), Entry { rev: 0, by: by.into(), body: Body::Value { value } });
                }
            }
            wrote(st, key, by);
        }
        Write::Merge { value } if !is_text => {
            let e = st.store.entry(key.into()).or_insert_with(|| Entry { rev: 0, by: by.into(), body: Body::Value { value: Value::Null } });
            if let Body::Value { value: v } = &mut e.body {
                crate::store::merge(v, &value);
            }
            wrote(st, key, by);
        }
        Write::Put { value: Value::String(text) } => {
            let len = match &st.store[key].body {
                Body::Text { doc } => doc.text.len_chars(),
                Body::Value { .. } => 0,
            };
            text_edit(st, key, vec![(0, len, text)], by)?;
        }
        Write::Edit { changes } if is_text => text_edit(st, key, changes, by)?,
        Write::Put { .. } => return Err(format!("{key} is text: put a string, or send an edit")),
        Write::Merge { .. } => return Err(format!("{key} is text: merge is for values")),
        Write::Edit { .. } => return Err(format!("{key} isn't text: put or merge a value")),
    }
    Ok(())
}

/// An agent's edit of a text entry: one undo step, made through a view of its own so every
/// editor's carets map through it and none of them jumps.
fn text_edit(st: &mut State, key: &str, changes: Vec<(usize, usize, String)>, by: &str) -> Result<(), String> {
    let Some(Entry { body: Body::Text { doc }, .. }) = st.store.get(key) else { return Err(format!("{key} isn't text")) };
    let len = doc.text.len_chars();
    let mut at = 0;
    for (from, to, _) in &changes {
        if from < &at || from > to || *to > len {
            return Err(format!("edit: ranges must be in order, apart and within the text (0..{len})"));
        }
        at = *to;
    }
    let now_ms = st.now_ms;
    let mut views = take_views(st, key);
    if let Some(Entry { body: Body::Text { doc }, .. }) = st.store.get_mut(key) {
        let mut vs: Vec<caretline::View> = views.iter().map(|(_, v)| v.clone()).collect();
        vs.push(caretline::View::new(caretline::Viewport { width: 80, height: 24 }));
        let agent = vs.len() - 1;
        caretline::views::update_doc(doc, &mut vs, agent, caretline::Msg::Tick { now_ms });
        caretline::views::update_doc(doc, &mut vs, agent, caretline::Msg::Edit { changes, join: false });
        vs.pop();
        for ((_, v), new) in views.iter_mut().zip(vs) {
            *v = new;
        }
    }
    put_views(st, key, views);
    wrote(st, key, by);
    Ok(())
}

/// fictty's key names as caretline keys: `a`, `space`, `enter`, `bs`, `del`, arrows, `home`,
/// `end`, `pgup`, `pgdn`, `esc`, with `c-` (Ctrl), `a-` (Alt) and `s-` (Shift) in front.
fn editor_key(name: &str) -> Option<caretline::Key> {
    use caretline::KeyCode as C;
    let mut mods = caretline::Mods::default();
    let mut rest = name;
    while let Some((m, r)) = rest.split_once('-').filter(|(m, r)| m.len() == 1 && !r.is_empty()) {
        match m {
            "c" => mods.ctrl = true,
            "a" => mods.alt = true,
            "s" => mods.shift = true,
            _ => return None,
        }
        rest = r;
    }
    let code = match rest {
        "space" => C::Char(' '),
        "enter" => C::Enter,
        "bs" => C::Backspace,
        "del" => C::Delete,
        "left" => C::Left,
        "right" => C::Right,
        "up" => C::Up,
        "down" => C::Down,
        "home" => C::Home,
        "end" => C::End,
        "pgup" => C::PageUp,
        "pgdn" => C::PageDown,
        "esc" => C::Esc,
        _ => {
            let mut cs = rest.chars();
            match (cs.next(), cs.next()) {
                (Some(c), None) => C::Char(c),
                _ => return None,
            }
        }
    };
    Some(caretline::Key { code, mods })
}

/// The action for a key: the focused node's own, then the UI's, then the defaults.
fn resolve(st: &State, key: &str) -> Option<Action> {
    let focused = st.focus.as_deref().and_then(|id| st.ui.root.find(id));
    if let Some(a) = focused.and_then(|n| n.keys.get(key)) {
        return Some(a.clone());
    }
    if let Some(a) = st.ui.keys.get(key) {
        return Some(a.clone());
    }
    // A tree folds where a list would switch tabs.
    if focused.is_some_and(|n| matches!(n.kind, Kind::Tree { .. })) {
        match key {
            "h" | "left" => return Some(Action::Named("collapse".into())),
            "l" | "right" => return Some(Action::Named("expand".into())),
            "space" => return Some(Action::Named("fold".into())),
            _ => {}
        }
    }
    let name = match key {
        "q" | "c-c" => "quit",
        "j" | "down" => "down",
        "k" | "up" => "up",
        "g" | "home" => "top",
        "G" | "end" => "bottom",
        "tab" => "focus_next",
        "s-tab" => "focus_prev",
        "h" | "left" => "tab_prev",
        "l" | "right" => "tab_next",
        "r" => "refresh",
        _ => return None,
    };
    Some(Action::Named(name.into()))
}

fn act(st: &mut State, a: Action) -> Vec<Effect> {
    match a {
        Action::Run { run } => {
            let sel = selected_item(st).unwrap_or(Value::Null);
            vec![Effect::Run(run.iter().map(|s| crate::model::fill(s, &sel)).collect())]
        }
        Action::Load { load } => {
            // `{file}` reads the selected row, so a menu of screens is a list.
            let sel = selected_item(st).unwrap_or(Value::Null);
            vec![Effect::Load(crate::model::fill(&load, &sel))]
        }
        Action::Focus { focus } => {
            if st.ui.root.find(&focus).is_some_and(Node::focusable) {
                st.focus = Some(focus);
            }
            vec![]
        }
        Action::Theme { theme } => {
            choose_theme(st, &theme);
            vec![]
        }
        Action::Append { append, value } => {
            let v = fill_value(&value, &action_context(st));
            match st.store.get_mut(&append).map(|e| &mut e.body) {
                Some(Body::Value { value: Value::Array(a) }) => a.push(v),
                None => {
                    st.store.insert(append.clone(), Entry { rev: 0, by: "person".into(), body: Body::Value { value: Value::Array(vec![v]) } });
                }
                Some(_) => {
                    st.status = Some(format!("append: {append} isn't a list"));
                    return vec![];
                }
            }
            wrote(st, &append, "person");
            settle(st);
            vec![]
        }
        Action::Put { put, value } => {
            let value = fill_value(&value, &action_context(st));
            if let Err(e) = write_entry(st, &put, Write::Put { value }, None, "person") {
                st.status = Some(format!("put: {e}"));
            }
            settle(st);
            vec![]
        }
        Action::Copy { copy } => {
            let text = crate::model::fill(&copy, &action_context(st));
            vec![Effect::Copy(text)]
        }
        Action::Host { host, args } => {
            let args = fill_value(&args, &action_context(st));
            vec![Effect::Host { name: host, args }]
        }
        Action::Remove { remove } => {
            let target = row_target(st).filter(|(key, _)| *key == remove);
            let Some((key, ptr)) = target else {
                st.status = Some(format!("remove: the focused list doesn't show a row of {remove}"));
                return vec![];
            };
            let (parent, last) = ptr.rsplit_once('/').unwrap_or(("", ""));
            let removed = match st.store.get_mut(&key).map(|e| &mut e.body) {
                Some(Body::Value { value }) => match (value.pointer_mut(parent), last.parse::<usize>()) {
                    (Some(Value::Array(a)), Ok(i)) if i < a.len() => {
                        a.remove(i);
                        true
                    }
                    _ => false,
                },
                _ => false,
            };
            if !removed {
                st.status = Some(format!("remove: no row at {ptr} in {key}"));
                return vec![];
            }
            wrote(st, &key, "person");
            settle(st);
            vec![]
        }
        Action::Toggle { toggle } => {
            let Some((key, ptr)) = row_target(st) else {
                st.status = Some(format!("toggle: the focused list doesn't show a store entry, so {toggle} can't change"));
                return vec![];
            };
            let field = format!("{ptr}/{}", toggle.replace('.', "/"));
            let flipped = match st.store.get_mut(&key).map(|e| &mut e.body) {
                Some(Body::Value { value }) => match value.pointer_mut(&field) {
                    Some(Value::Bool(b)) => {
                        *b = !*b;
                        true
                    }
                    Some(v @ Value::Null) => {
                        *v = Value::Bool(true);
                        true
                    }
                    Some(_) => false,
                    None => match value.pointer_mut(&ptr) {
                        Some(Value::Object(m)) => {
                            m.insert(toggle.clone(), Value::Bool(true));
                            true
                        }
                        _ => false,
                    },
                },
                _ => false,
            };
            if !flipped {
                st.status = Some(format!("toggle: {toggle} of the selected row isn't true or false"));
                return vec![];
            }
            wrote(st, &key, "person");
            vec![]
        }
        Action::Do { steps } => steps.into_iter().flat_map(|a| act(st, a)).collect(),
        Action::Named(n) => match n.as_str() {
            "quit" => vec![Effect::Quit],
            "refresh" => refresh(st),
            "down" => step(st, |sel, len| (sel + 1).min(len.saturating_sub(1))),
            "up" => step(st, |sel, _| sel.saturating_sub(1)),
            "top" => step(st, |_, _| 0),
            "bottom" => step(st, |_, len| len.saturating_sub(1)),
            "tab_next" => switch(st, 1),
            "tab_prev" => switch(st, -1),
            "fold" => fold(st, Fold::Toggle),
            "collapse" => fold(st, Fold::Close),
            "expand" => fold(st, Fold::Open),
            "focus_next" => cycle(st, 1),
            "focus_prev" => cycle(st, -1),
            other => {
                st.status = Some(format!("unknown action {other:?}"));
                vec![]
            }
        },
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Fold {
    Toggle,
    Open,
    Close,
}

/// Fold or unfold the focused tree's selected row. Closing a row that's closed (or has nothing
/// under it) goes to its parent; opening one that's open goes to its first child.
fn fold(st: &mut State, how: Fold) -> Vec<Effect> {
    let Some(id) = st.focus.clone() else { return vec![] };
    if !st.ui.root.find(&id).is_some_and(|n| matches!(n.kind, Kind::Tree { .. })) {
        return vec![];
    }
    let rows = st.rows.get(&id).map(Vec::as_slice).unwrap_or(&[]);
    let at = selected(st, &id);
    let Some(row) = rows.get(at) else { return vec![] };
    let kids = row["$kids"].as_u64().unwrap_or(0) > 0;
    let open = row["$open"].as_bool().unwrap_or(false);
    let depth = row["$depth"].as_u64().unwrap_or(0);
    let key = row["$key"].as_str().unwrap_or_default().to_string();
    let flip = match how {
        Fold::Toggle => kids,
        Fold::Open => kids && !open,
        Fold::Close => kids && open,
    };
    let local = st.local.entry(id.clone()).or_default();
    if flip {
        if !local.toggled.remove(&key) {
            local.toggled.insert(key);
        }
    } else if how == Fold::Close && depth > 0 {
        let parent = rows[..at].iter().rposition(|r| r["$depth"].as_u64() == Some(depth - 1));
        local.selected = parent.unwrap_or(at);
    } else if how == Fold::Open && kids && open {
        local.selected = at + 1;
    }
    vec![]
}

/// The focused list's (or tree's) selected row as a place in the store: the entry its bind names,
/// and a JSON Pointer to the row in that entry's value. None unless the node shows a store entry.
fn row_target(st: &State) -> Option<(String, String)> {
    let id = st.focus.as_ref()?;
    let n = st.ui.root.find(id)?;
    let bind = n.bind.as_deref().filter(|b| !b.starts_with('@'))?;
    let (key, path) = split_bind(bind);
    if !matches!(st.store.get(key)?.body, Body::Value { .. }) {
        return None;
    }
    let at = selected(st, id);
    let row = items(st, n).get(at)?;
    let inner = match row.get("$ptr").and_then(Value::as_str) {
        Some(p) => p.to_string(),
        None => format!("/{at}"),
    };
    let prefix: String = if path.is_empty() { String::new() } else { path.split('.').map(|p| format!("/{p}")).collect() };
    Some((key.to_string(), format!("{prefix}{inner}")))
}

/// The text a filter reads: a text entry's contents, or a bound string.
fn query_text(st: &State, bind: &str) -> String {
    match st.store.get(bind).map(|e| &e.body) {
        Some(Body::Text { doc }) => doc.text.to_string(),
        _ => crate::model::scalar(value_of(st, bind)),
    }
}

/// Whether every term is somewhere in the row's values (its text, numbers; not its nested rows).
fn row_matches(r: &Value, terms: &[String], nest: Option<&str>) -> bool {
    fn text(v: &Value, nest: Option<&str>, out: &mut String) {
        match v {
            Value::Object(m) => {
                for (k, x) in m {
                    if Some(k.as_str()) != nest {
                        text(x, None, out);
                    }
                }
            }
            Value::Array(a) => a.iter().for_each(|x| text(x, None, out)),
            Value::String(s) => {
                out.push_str(&s.to_lowercase());
                out.push(' ');
            }
            Value::Number(n) => {
                out.push_str(&n.to_string());
                out.push(' ');
            }
            _ => {}
        }
    }
    let mut hay = String::new();
    text(r, nest, &mut hay);
    terms.iter().all(|t| hay.contains(t.as_str()))
}

/// A row as a tree or a filter shows it: the row (an object; anything else under `value`), its
/// nested rows left out, with where it is (`$ptr`) and, in a tree, `$depth`, `$kids`, `$open`
/// and `$key`.
fn shown_row(r: &Value, nest: Option<&str>, ptr: String) -> Value {
    let mut m = match r {
        Value::Object(m) => m.iter().filter(|(k, _)| Some(k.as_str()) != nest).map(|(k, v)| (k.clone(), v.clone())).collect(),
        other => serde_json::Map::from_iter([("value".to_string(), other.clone())]),
    };
    m.insert("$ptr".into(), ptr.into());
    Value::Object(m)
}

struct TreeCx<'a> {
    nest: &'a str,
    key: &'a str,
    open: bool,
    toggled: Option<&'a BTreeSet<String>>,
    terms: &'a [String],
}

/// A tree's shown rows, depth first: every row whose parents are open, or, while filtering, every
/// row that matches or has a match under it (opened to show it). True when any row was shown.
fn flatten(rows: &[Value], at: &str, depth: usize, t: &TreeCx, out: &mut Vec<Value>) -> bool {
    let mut any = false;
    let nest_ptr: String = t.nest.split('.').map(|p| format!("/{p}")).collect();
    for (i, r) in rows.iter().enumerate() {
        let ptr = format!("{at}/{i}");
        let kids = lookup(r, t.nest).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
        let key = Some(crate::model::scalar(lookup(r, t.key))).filter(|k| !k.is_empty()).unwrap_or_else(|| ptr.clone());
        let filtering = !t.terms.is_empty();
        let open = filtering || t.open != t.toggled.is_some_and(|s| s.contains(&key));
        let mut under = Vec::new();
        let hit_under = if open || filtering { flatten(kids, &format!("{ptr}{nest_ptr}"), depth + 1, t, &mut under) } else { false };
        if filtering && !hit_under && !row_matches(r, t.terms, Some(t.nest)) {
            continue;
        }
        let mut row = shown_row(r, Some(t.nest), ptr);
        row["$depth"] = depth.into();
        row["$kids"] = kids.len().into();
        row["$open"] = (open && !kids.is_empty()).into();
        row["$key"] = key.into();
        out.push(row);
        if open {
            out.extend(under);
        }
        any = true;
    }
    any
}

/// Work out `State::rows`: each tree's shown rows, and each filtered list's or table's matches.
fn derive_rows(st: &mut State) {
    let mut all = Vec::new();
    st.ui.root.walk(&mut all);
    let mut out = BTreeMap::new();
    for n in all {
        let Some(id) = &n.id else { continue };
        let query = n.filter().map(|f| query_text(st, f)).unwrap_or_default();
        let terms: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        match &n.kind {
            Kind::Tree { nest, key, open, .. } => {
                let t = TreeCx {
                    nest: nest.as_deref().unwrap_or("children"),
                    key: key.as_deref().unwrap_or("id"),
                    open: *open,
                    toggled: st.local.get(id).map(|l| &l.toggled),
                    terms: &terms,
                };
                let mut rows = Vec::new();
                flatten(raw_items(st, n), "", 0, &t, &mut rows);
                out.insert(id.clone(), rows);
            }
            _ if !terms.is_empty() => {
                let rows = raw_items(st, n).iter().enumerate().filter(|(_, r)| row_matches(r, &terms, None)).map(|(i, r)| shown_row(r, None, format!("/{i}"))).collect();
                out.insert(id.clone(), rows);
            }
            _ => {}
        }
    }
    st.rows = out;
}

/// Show the next or previous child of the focused tabs node.
fn switch(st: &mut State, dir: isize) -> Vec<Effect> {
    let Some(id) = st.focus.clone() else { return vec![] };
    let n = match st.ui.root.find(&id).map(|n| &n.kind) {
        Some(Kind::Tabs { children, .. }) => children.len() as isize,
        _ => return vec![],
    };
    let local = st.local.entry(id).or_default();
    local.tab = ((local.tab as isize + dir).rem_euclid(n.max(1))) as usize;
    settle(st);
    vec![]
}

fn cycle(st: &mut State, dir: isize) -> Vec<Effect> {
    let ids = focus_order(st);
    if ids.is_empty() {
        return vec![];
    }
    let at = st.focus.as_ref().and_then(|f| ids.iter().position(|i| i == f)).unwrap_or(0) as isize;
    let n = ids.len() as isize;
    st.focus = Some(ids[((at + dir).rem_euclid(n)) as usize].clone());
    vec![]
}

fn step(st: &mut State, f: impl Fn(usize, usize) -> usize) -> Vec<Effect> {
    let Some(id) = st.focus.clone() else { return vec![] };
    let Some(n) = st.ui.root.find(&id) else { return vec![] };
    let rows = items(st, n);
    let at = st.local.get(&id).map(|l| l.selected).unwrap_or(0);
    let mut to = f(at, rows.len());
    // Past rows that can't be selected (section headers), in the direction of travel.
    let dir: isize = if to >= at { 1 } else { -1 };
    while rows.get(to).is_some_and(|r| n.skips(r)) {
        match to.checked_add_signed(dir).filter(|t| *t < rows.len()) {
            Some(t) => to = t,
            None => {
                to = at;
                break;
            }
        }
    }
    st.local.entry(id).or_default().selected = to;
    vec![]
}

fn refresh(st: &State) -> Vec<Effect> {
    st.ui.data.keys().map(|k| Effect::Fetch(k.clone())).collect()
}

/// After any change: drop view state for ids that are gone, keep focus on something focusable,
/// and keep each selection inside its list.
fn settle(st: &mut State) {
    let order = focus_order(st);
    let mut all = Vec::new();
    st.ui.root.walk(&mut all);
    let ids: Vec<String> = all.iter().filter_map(|n| n.id.clone()).collect();
    st.local.retain(|k, _| ids.contains(k));
    if !st.focus.as_ref().is_some_and(|f| order.contains(f)) {
        st.focus = order.first().cloned();
    }
    let lens: Vec<(String, usize)> =
        all.iter().filter(|n| n.focusable()).map(|n| (n.id.clone().unwrap(), items(st, n).len())).collect();
    for (id, len) in lens {
        if let Some(l) = st.local.get_mut(&id) {
            l.selected = l.selected.min(len.saturating_sub(1));
        }
    }
    // A selection on a row that can't be selected moves to the next one that can.
    let fix: Vec<(String, usize)> = all
        .iter()
        .filter(|n| n.focusable())
        .filter_map(|n| {
            let id = n.id.clone()?;
            let rows = items(st, n);
            let at = st.local.get(&id).map(|l| l.selected).unwrap_or(0);
            if !rows.get(at).is_some_and(|r| n.skips(r)) {
                return None;
            }
            (at..rows.len()).chain(0..at).find(|k| !n.skips(&rows[*k])).map(|k| (id, k))
        })
        .collect();
    for (id, k) in fix {
        st.local.entry(id).or_default().selected = k;
    }
}

/// Every editor and input shows a text entry in the store: its seed when the entry is new, or a
/// string an agent put there before. Its view of each entry is its own, by id.
fn fields(st: &mut State) {
    let mut all = Vec::new();
    st.ui.root.walk(&mut all);
    let found: Vec<(String, String, String, bool)> = all
        .iter()
        .filter_map(|n| {
            let f = n.field()?;
            let key = doc_key(st, n)?;
            let seed = match n.bind.as_deref().and_then(|b| value_of(st, b)) {
                Some(v) if f.doc.is_some() => crate::model::fill(f.seed, v),
                _ => f.seed.to_string(),
            };
            Some((n.id.clone()?, key, seed, f.line))
        })
        .collect();
    for (id, key, seed, line) in found {
        let fresh = |text: &str| {
            let mut e = Entry::text(text, "seed");
            if let Body::Text { doc } = &mut e.body {
                doc.config.single_line = line;
                doc.sanitize();
            }
            e
        };
        match st.store.get(&key).map(|e| &e.body) {
            None => {
                st.store.insert(key.clone(), fresh(&seed));
                st.store_writes += 1;
            }
            Some(Body::Value { value: Value::String(s) }) => {
                let text = s.clone();
                let e = st.store.get_mut(&key).expect("present");
                *e = Entry { rev: e.rev, by: e.by.clone(), ..fresh(&text) };
            }
            Some(Body::Value { .. }) => {
                st.status = Some(format!("{id}: {key} holds a value, not text"));
                continue;
            }
            Some(Body::Text { .. }) => {}
        }
        let Some(Entry { body: Body::Text { doc }, .. }) = st.store.get(&key) else { continue };
        let view = st.local.entry(id).or_default().editors.entry(key).or_insert_with(|| {
            let mut v = caretline::View::new(caretline::Viewport { width: 1, height: 1 });
            v.config.status_bar = false;
            v
        });
        view.fit(doc);
    }
}

/// The `$store` source: one row per entry (key, rev, by, what it holds), for a UI to show.
fn store_summary(st: &State) -> Value {
    let rows: Vec<Value> = st
        .store
        .iter()
        .map(|(k, e)| {
            let holds = match &e.body {
                Body::Text { doc } => format!("text, {} chars", doc.text.len_chars()),
                Body::Value { value: Value::String(t) } => format!("text, {} chars", t.chars().count()),
                Body::Value { value: Value::Array(a) } => format!("a list of {}", a.len()),
                Body::Value { value: Value::Object(o) } => format!("an object of {}", o.len()),
                Body::Value { .. } => "a value".to_string(),
            };
            serde_json::json!({"key": k, "rev": e.rev, "by": e.by, "holds": holds})
        })
        .collect();
    Value::Array(rows)
}

/// What's wrong with the UI against the store, the sources and the theme (see `problems`).
pub fn problems(st: &State) -> Vec<String> {
    let mut all = Vec::new();
    st.ui.root.walk(&mut all);
    let active = st.theme.as_ref().and_then(|t| st.themes.get(t));
    let mut out = Vec::new();
    let kind = |v: &Value| match v {
        Value::Array(_) => "a list",
        Value::Object(_) => "an object",
        Value::String(_) => "text",
        Value::Number(_) => "a number",
        Value::Bool(_) => "true or false",
        Value::Null => "nothing",
    };
    for n in &all {
        let name = n.id.clone().unwrap_or_else(|| n.kind_name().to_string());
        if let Some(v) = &n.variant
            && active.is_some_and(|t| !t.variants.contains_key(v))
        {
            out.push(format!("{name}: the theme has no variant {v:?}"));
        }
        if let Some(t) = &n.tone {
            let known = active.is_some_and(|a| a.tokens.contains_key(t)) || st.ui.theme.contains_key(t) || t.parse::<ratatui::style::Color>().is_ok();
            if !known {
                out.push(format!("{name}: tone {t:?} isn't a theme token or a colour"));
            }
        }
        if let Kind::Input { accept: Some(a), .. } = &n.kind
            && a != "number"
        {
            out.push(format!("input {name}: accept {a:?} isn't known (only \"number\")"));
        }
        let Some(bind) = n.bind.as_deref().filter(|b| !b.starts_with('@')) else { continue };
        let Some(v) = value_of(st, bind) else { continue };
        let wants_list = matches!(n.kind, Kind::List { .. } | Kind::Table { .. } | Kind::Tree { .. } | Kind::Bars { .. } | Kind::Sparkline { .. });
        let literal = !items(st, n).is_empty() && !v.is_array();
        if wants_list && !v.is_array() && !literal {
            out.push(format!("{} {name}: {bind} should be a list, but it's {}", n.kind_name(), kind(v)));
        }
        if let (Kind::Sparkline { field: None, .. }, Value::Array(a)) = (&n.kind, v)
            && a.iter().any(|x| !x.is_number())
        {
            out.push(format!("sparkline {name}: {bind} should be a list of numbers (or set `field`)"));
        }
    }
    out
}

/// The focusable nodes on screen, in tree order (hidden tabs left out).
pub fn focus_order(st: &State) -> Vec<String> {
    let mut all = Vec::new();
    st.ui.root.walk_shown(&|n| tab_of(st, n), &mut all);
    all.into_iter().filter(|n| n.focusable()).filter_map(|n| n.id.clone()).collect()
}

/// The shown child of a tabs node.
pub fn tab_of(st: &State, n: &Node) -> usize {
    n.id.as_ref().and_then(|id| st.local.get(id)).map(|l| l.tab).unwrap_or(0)
}

/// The value a node is bound to, if it has one and it has arrived.
pub fn bound<'a>(st: &'a State, n: &Node) -> Option<&'a Value> {
    value_of(st, n.bind.as_deref()?)
}

/// A bind's value: a source (and a path into it), or `@id` (and a path), the selected row of
/// node `id`, so a detail panel follows a list's selection.
pub fn value_of<'a>(st: &'a State, bind: &str) -> Option<&'a Value> {
    let (source, path) = split_bind(bind);
    if let Some(id) = source.strip_prefix('@') {
        let n = st.ui.root.find(id)?;
        return lookup(items(st, n).get(selected(st, id))?, path);
    }
    if let Some(e) = st.store.get(source) {
        return match &e.body {
            Body::Value { value } => lookup(value, path),
            Body::Text { .. } => None,
        };
    }
    match st.data.get(source)? {
        Slot::Ready { value } => lookup(value, path),
        _ => None,
    }
}

/// `bind` is a source name, then optionally a path into its value: `"tick.rows"` reads `rows` of
/// the `tick` source, so one source can feed many nodes.
pub fn split_bind(bind: &str) -> (&str, &str) {
    bind.split_once('.').unwrap_or((bind, ""))
}

/// A list's or table's rows: its literal `items`, else its bound array. Borrowed: a 50,000-row
/// source is read in place, never copied per message or per frame.
pub fn items<'a>(st: &'a State, n: &'a Node) -> &'a [Value] {
    if let Some(rows) = n.id.as_ref().and_then(|id| st.rows.get(id)) {
        return rows;
    }
    raw_items(st, n)
}

/// A node's rows as given: its literal `items`, else its bound array.
fn raw_items<'a>(st: &'a State, n: &'a Node) -> &'a [Value] {
    let literal = match &n.kind {
        Kind::List { items, .. } | Kind::Table { items, .. } | Kind::Tree { items, .. } | Kind::Bars { items, .. } => items,
        _ => return &[],
    };
    if !literal.is_empty() {
        return literal;
    }
    match bound(st, n) {
        Some(Value::Array(a)) => a,
        _ => &[],
    }
}

pub fn selected(st: &State, id: &str) -> usize {
    st.local.get(id).map(|l| l.selected).unwrap_or(0)
}

/// The focused list's selected row, for `run` templates.
pub fn selected_item(st: &State) -> Option<Value> {
    let id = st.focus.as_ref()?;
    let n = st.ui.root.find(id)?;
    items(st, n).get(selected(st, id)).cloned()
}

/// A source's value from a command's stdout, narrowed by its `path`.
pub fn narrow(raw: &str, path: Option<&str>) -> Result<Value, String> {
    let v: Value = serde_json::from_str(raw.trim()).map_err(|e| format!("not JSON: {e}"))?;
    match path {
        None => Ok(v),
        Some(p) => lookup(&v, p).cloned().ok_or_else(|| format!("no {p:?} in the output")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ui(v: Value) -> Ui {
        serde_json::from_value(v).unwrap()
    }

    fn two_lists() -> Ui {
        ui(json!({
            "root": {"type": "row", "children": [
                {"type": "list", "id": "a", "items": ["one", "two", "three"]},
                {"type": "list", "id": "b", "bind": "tasks", "item": "{text}",
                 "keys": {"x": {"run": ["thc", "done", "{id}"]}}}
            ]},
            "data": {"tasks": {"value": [{"id": "t1", "text": "Buy milk"}, {"id": "t2", "text": "Essay"}]}}
        }))
    }

    fn note() -> Ui {
        ui(json!({
            "root": {"type": "row", "children": [
                {"type": "editor", "id": "note", "text": "hi"},
                {"type": "list", "id": "side", "items": ["a", "b"]}
            ]}
        }))
    }

    fn text_of(st: &State, key: &str) -> String {
        match &st.store[key].body {
            Body::Text { doc } => doc.text.to_string(),
            Body::Value { value } => panic!("{key} is a value: {value}"),
        }
    }

    fn caret(st: &State, id: &str) -> usize {
        field_view(st, id).unwrap().1.caret()
    }

    fn keys(st: &mut State, ks: &[&str]) -> Vec<Effect> {
        ks.iter().flat_map(|k| update(st, Msg::Key { key: (*k).into() })).collect()
    }

    fn write(st: &mut State, key: &str, w: Value, if_rev: Option<u64>) -> Value {
        let write: Write = serde_json::from_value(w).unwrap();
        match update(st, Msg::Write { key: key.into(), write, if_rev, by: "agent".into() }).pop() {
            Some(Effect::Reply(r)) => r,
            other => panic!("no reply: {other:?}"),
        }
    }

    #[test]
    fn an_editor_takes_the_keys_while_focused() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: note() });
        assert_eq!(st.focus.as_deref(), Some("note"), "the first focusable node");
        update(&mut st, Msg::Size { id: "note".into(), width: 20, height: 5 });
        let fx = keys(&mut st, &["end", "space", "q", "enter", "j", "c-c"]);
        assert!(fx.is_empty(), "q and Ctrl-C don't quit while editing");
        assert_eq!(text_of(&st, "note"), "hi q\nj");
        assert_eq!(st.store["note"].by, "person");
        // Tab leaves the editor; then the keys are the screen's again.
        keys(&mut st, &["tab", "j"]);
        assert_eq!(st.focus.as_deref(), Some("side"));
        assert_eq!(selected(&st, "side"), 1);
        assert_eq!(text_of(&st, "note"), "hi q\nj");
        assert_eq!(keys(&mut st, &["q"]), vec![Effect::Quit]);
    }

    #[test]
    fn typing_groups_into_undo_steps_by_time() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: note() });
        keys(&mut st, &["end", "space", "a", "b"]);
        update(&mut st, Msg::Tick { now_ms: 60_000 });
        keys(&mut st, &["c"]);
        keys(&mut st, &["c-z"]);
        assert_eq!(text_of(&st, "note"), "hi ab", "the later burst undoes alone");
        keys(&mut st, &["c-y"]);
        assert_eq!(text_of(&st, "note"), "hi abc");
    }

    #[test]
    fn a_push_never_loses_what_was_written() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: note() });
        keys(&mut st, &["end", "!"]);
        // A UI without the editor: the note is still in the store.
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "list", "id": "side", "items": ["a"]}})) });
        assert_eq!(text_of(&st, "note"), "hi!");
        assert!(!st.local.contains_key("note"), "its view went with it");
        // The editor back, with a different seed: the note wins over the seed.
        let mut again = note();
        if let Some(c) = again.root.children_mut() {
            c[0].kind = Kind::Editor { text: "a new seed".into(), doc: None, placeholder: None, syntax: None };
        }
        update(&mut st, Msg::Push { ui: again });
        assert_eq!(text_of(&st, "note"), "hi!");
        // Seeds for values likewise only start entries.
        let seeded = |v: Value| ui(json!({"root": {"type": "list", "id": "l", "bind": "rows"}, "store": {"rows": v}}));
        update(&mut st, Msg::Push { ui: seeded(json!(["seed"])) });
        write(&mut st, "rows", json!({"write": "put", "value": ["agent's"]}), None);
        update(&mut st, Msg::Push { ui: seeded(json!(["another seed"])) });
        assert_eq!(st.store["rows"].read()["value"], json!(["agent's"]));
        let back: State = serde_json::from_value(serde_json::to_value(&st).unwrap()).unwrap();
        assert_eq!(back.store, st.store, "texts with their history, values, revisions");
    }

    #[test]
    fn a_stale_write_is_refused_and_says_what_is_there() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "text", "text": "x"}, "store": {"todo": {"done": 1}}})) });
        let r = write(&mut st, "todo", json!({"write": "merge", "value": {"open": 3}}), Some(1));
        assert_eq!((r["ok"].clone(), r["rev"].clone()), (json!(true), json!(2)));
        let r = write(&mut st, "todo", json!({"write": "put", "value": {}}), Some(1));
        assert_eq!(r["ok"], json!(false), "another agent wrote rev 2");
        assert_eq!(r["current"]["value"], json!({"done": 1, "open": 3}));
        assert_eq!(st.store["todo"].rev, 2);
    }

    #[test]
    fn an_agent_edit_lands_around_the_persons_typing() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: note() });
        let rev = st.store["note"].rev;
        // The agent read "hi" at `rev`, then the person typed.
        keys(&mut st, &["end", " ", "t", "h", "e", "r", "e"]);
        let r = write(&mut st, "note", json!({"write": "edit", "changes": [[0, 0, "> "]]}), Some(rev));
        assert_eq!(r["ok"], json!(false), "the agent's view of the text is out of date");
        // Read again, edit again: it lands, and the person's caret moves with the text.
        let rev = st.store["note"].rev;
        assert_eq!(caret(&st, "note"), 8);
        let r = write(&mut st, "note", json!({"write": "edit", "changes": [[0, 0, "> "]]}), Some(rev));
        assert_eq!(r["ok"], json!(true));
        assert_eq!(text_of(&st, "note"), "> hi there");
        assert_eq!(caret(&st, "note"), 10, "still after what they typed");
        keys(&mut st, &["c-z"]);
        assert_eq!(text_of(&st, "note"), "hi there", "one undo takes the agent's edit back");
        // A whole new text is a put of a string; a value write to text is refused.
        write(&mut st, "note", json!({"write": "put", "value": "rewritten"}), None);
        assert_eq!(text_of(&st, "note"), "rewritten");
        assert_eq!(write(&mut st, "note", json!({"write": "merge", "value": {}}), None)["ok"], json!(false));
    }

    #[test]
    fn two_editors_on_one_text_share_it() {
        let mut st = State::default();
        update(&mut st, Msg::Push {
            ui: ui(json!({"root": {"type": "row", "children": [
                {"type": "editor", "id": "a", "bind": "doc", "text": "shared"},
                {"type": "editor", "id": "b", "bind": "doc"}
            ]}})),
        });
        // b's caret at the end, then type at the start in a.
        keys(&mut st, &["tab", "end", "s-tab", "x"]);
        assert_eq!(text_of(&st, "doc"), "xshared");
        assert_eq!((caret(&st, "a"), caret(&st, "b")), (1, 7), "b's caret moved with its text");
    }

    #[test]
    fn a_notes_app_is_a_list_and_an_editor_on_the_selected_note() {
        let mut st = State::default();
        update(&mut st, Msg::Push {
            ui: ui(json!({
                "root": {"type": "col", "children": [
                    {"type": "input", "id": "title", "bind": "draft", "placeholder": "New note",
                     "keys": {"enter": {"do": [
                         {"append": "notes", "value": {"id": "{$id}", "title": "{store.draft}"}},
                         {"put": "draft", "value": ""}
                     ]}}},
                    {"type": "list", "id": "list", "bind": "notes", "item": "{title}", "keys": {"x": {"remove": "notes"}}},
                    {"type": "editor", "id": "body", "bind": "@list", "doc": "note/{id}", "text": "# {title}\n"}
                ]},
                "store": {"notes": [{"id": "a", "title": "Groceries"}]}
            })),
        });
        assert_eq!(text_of(&st, "note/a"), "# Groceries\n", "seeded from the selected row");
        // Type a title into the input and press Enter: a new note, the input cleared.
        keys(&mut st, &["H", "i", "enter"]);
        let notes = st.store["notes"].read()["value"].clone();
        assert_eq!(notes[1]["title"], json!("Hi"));
        assert_eq!(text_of(&st, "draft"), "", "the input is cleared");
        assert!(notes[1]["id"].is_string());
        // Select it: the editor now shows (and seeds) that note.
        keys(&mut st, &["tab", "j", "tab", "end", "!"]);
        let id = notes[1]["id"].as_str().unwrap();
        assert_eq!(text_of(&st, &format!("note/{id}")), "# Hi!\n");
        assert_eq!(text_of(&st, "note/a"), "# Groceries\n", "the first note is untouched");
        // Remove the first note from the list; its text stays in the store.
        keys(&mut st, &["s-tab", "k", "x"]);
        assert_eq!(st.store["notes"].read()["value"].as_array().unwrap().len(), 1);
        assert!(st.store.contains_key("note/a"));
    }

    #[test]
    fn an_input_is_one_line_and_esc_leaves_it() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "row", "children": [
            {"type": "input", "id": "q"}, {"type": "list", "id": "l", "items": [1]}
        ]}, "keys": {"esc": "focus_next"}})) });
        keys(&mut st, &["a", "enter", "b"]);
        assert_eq!(text_of(&st, "q"), "ab", "enter adds no line");
        keys(&mut st, &["esc"]);
        assert_eq!(st.focus.as_deref(), Some("l"));
    }

    #[test]
    fn copy_puts_a_command_for_this_screen_on_the_clipboard() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "list", "id": "l", "items": [{"cmd": "fictty theme paper"}],
            "keys": {"enter": {"copy": "{$self.cli} {cmd}"}}}})) });
        update(&mut st, Msg::Host { bin: "/opt/it's/fictty".into(), socket: "/tmp/f.sock".into(), dir: "/srv/ui".into() });
        assert_eq!(update(&mut st, Msg::Key { key: "enter".into() }), vec![Effect::Copy("FICTTY_SOCKET='/tmp/f.sock' '/opt/it'\\''s/fictty' fictty theme paper".into())]);
        let Some(Slot::Ready { value }) = st.data.get("$self") else { panic!("no $self") };
        assert_eq!(value["env"], "export PATH='/opt/it'\\''s':'/srv/ui':\"$PATH\" FICTTY_SOCKET='/tmp/f.sock'");
    }

    #[test]
    fn a_number_input_takes_only_numbers_and_submits_only_a_whole_one() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "input", "id": "amt", "title": "Amount", "accept": "number",
            "keys": {"enter": {"append": "rows", "value": "{store.amt}"}}}, "store": {"rows": []}})) });
        keys(&mut st, &["-", "enter"]);
        assert_eq!(st.status.as_deref(), Some("Amount: \"-\" isn't a number"));
        assert_eq!(st.store["rows"].read()["value"], json!([]), "nothing booked");
        keys(&mut st, &["3", "-", "x", ".", "5", ".", "0"]);
        assert_eq!(text_of(&st, "amt"), "-3.50", "the second '-', the 'x' and the second '.' were refused");
        keys(&mut st, &["enter"]);
        assert_eq!(st.store["rows"].read()["value"], json!(["-3.50"]));
    }

    #[test]
    fn store_actions_keep_types_and_say_what_went_wrong() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({
            "root": {"type": "list", "id": "l", "bind": "rows", "keys": {
                "a": {"append": "rows", "value": {"amount": "{store.amt}", "n": "{store.n}", "note": "paid {store.amt}"}},
                "b": {"append": "amt", "value": 1}
            }},
            "store": {"rows": [], "amt": "-12.50", "n": 3}
        })) });
        keys(&mut st, &["a"]);
        assert_eq!(st.store["rows"].read()["value"], json!([{"amount": "-12.50", "n": 3, "note": "paid -12.50"}]));
        keys(&mut st, &["b"]);
        assert_eq!(st.status.as_deref(), Some("append: amt isn't a list"));
    }

    #[test]
    fn themes_are_loaded_chosen_and_survive_a_push() {
        let mut st = State::default();
        let theme = |accent: &str| serde_json::from_value::<ThemeSpec>(json!({"tokens": {"accent": accent}})).unwrap();
        update(&mut st, Msg::Theme { add: [("glass".to_string(), theme("#39d6e6")), ("paper".to_string(), theme("#0f8fa0"))].into(), choose: None });
        assert_eq!(st.theme.as_deref(), Some("glass"), "the first loaded is chosen");
        update(&mut st, Msg::Push { ui: note() });
        assert_eq!(st.theme.as_deref(), Some("glass"));
        keys(&mut st, &["tab"]);
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "list", "id": "l", "items": [1], "keys": {"t": {"theme": "next"}}}})) });
        keys(&mut st, &["t"]);
        assert_eq!(st.theme.as_deref(), Some("paper"));
        keys(&mut st, &["t"]);
        assert_eq!(st.theme.as_deref(), Some("glass"), "next wraps around");
        update(&mut st, Msg::Theme { add: BTreeMap::new(), choose: Some("neon".into()) });
        assert!(st.status.as_deref().unwrap().starts_with("theme: no theme named \"neon\""));
    }

    #[test]
    fn problems_say_what_is_wrong_in_words() {
        let mut st = State::default();
        let theme: ThemeSpec = serde_json::from_value(json!({"tokens": {"warn": "#ffbd4f"}, "variants": {"card": {}}})).unwrap();
        update(&mut st, Msg::Theme { add: [("t".to_string(), theme)].into(), choose: None });
        update(&mut st, Msg::Push { ui: ui(json!({
            "root": {"type": "col", "children": [
                {"type": "table", "id": "t", "bind": "rows", "columns": [{"title": "a", "value": "{a}"}]},
                {"type": "sparkline", "id": "s", "bind": "nums"},
                {"type": "text", "id": "x", "variant": "cardd", "tone": "warn", "text": "hi"},
                {"type": "text", "id": "y", "variant": "card", "tone": "warning", "text": "hi"}
            ]},
            "store": {"rows": {"a": 1}, "nums": [1, "two"]}
        })) });
        assert_eq!(st.problems, vec![
            "table t: rows should be a list, but it's an object".to_string(),
            "sparkline s: nums should be a list of numbers (or set `field`)".to_string(),
            "x: the theme has no variant \"cardd\"".to_string(),
            "y: tone \"warning\" isn't a theme token or a colour".to_string(),
        ]);
        // Fixing the data clears it.
        let r = write(&mut st, "rows", json!({"write": "put", "value": [{"a": 1}]}), None);
        assert_eq!(r["ok"], json!(true));
        assert_eq!(st.problems.len(), 3);
    }

    #[test]
    fn a_click_in_an_editor_focuses_it_and_places_the_caret() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "row", "children": [
            {"type": "list", "id": "l", "items": [1]},
            {"type": "editor", "id": "e", "text": "first line\nsecond"}
        ]}})) });
        assert_eq!(st.focus.as_deref(), Some("l"));
        update(&mut st, Msg::Size { id: "e".into(), width: 20, height: 4 });
        update(&mut st, Msg::Point { id: "e".into(), col: 3, row: 1 });
        assert_eq!(st.focus.as_deref(), Some("e"));
        assert_eq!(caret(&st, "e"), 14, "line two, column three");
    }

    #[test]
    fn set_moves_view_state() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: note() });
        update(&mut st, Msg::Set { id: "side".into(), selected: Some(1), tab: None });
        assert_eq!(selected(&st, "side"), 1);
        update(&mut st, Msg::Set { id: "nope".into(), selected: Some(1), tab: None });
        assert_eq!(st.status.as_deref(), Some("set: no node with id \"nope\""));
    }

    #[test]
    fn the_log_joins_typed_keys_and_keeps_the_newest_first() {
        let mut st = State::default();
        let act = |op: &str, what: &str| Msg::Activity { op: op.into(), what: what.into(), at: "09:41:00".into() };
        for m in [act("push", "a new UI"), act("key", "h"), act("key", "i"), act("key", "space"), act("key", "enter"), act("set", "note")] {
            update(&mut st, m);
        }
        let Some(Slot::Ready { value }) = st.data.get("$log") else { panic!("no log") };
        let ops: Vec<(&str, &str)> = value.as_array().unwrap().iter().map(|e| (e["op"].as_str().unwrap(), e["what"].as_str().unwrap())).collect();
        assert_eq!(ops, vec![("set", "note"), ("key", "hi ⏎"), ("push", "a new UI")]);
    }

    #[test]
    fn push_focuses_the_first_list_and_fetches_sources() {
        let mut st = State::default();
        let fx = update(&mut st, Msg::Push { ui: two_lists() });
        assert_eq!(st.focus.as_deref(), Some("a"));
        assert_eq!(fx, vec![Effect::Fetch("tasks".into())]);
    }

    #[test]
    fn keys_move_the_selection_and_run_templates_the_selected_row() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: two_lists() });
        update(&mut st, Msg::Data { name: "tasks".into(), result: Ok(json!([{"id": "t1"}, {"id": "t2"}])) });
        for k in ["j", "j", "j"] {
            update(&mut st, Msg::Key { key: k.into() });
        }
        assert_eq!(selected(&st, "a"), 2, "clamped to the last row");
        update(&mut st, Msg::Key { key: "tab".into() });
        update(&mut st, Msg::Key { key: "j".into() });
        let fx = update(&mut st, Msg::Key { key: "x".into() });
        assert_eq!(fx, vec![Effect::Run(vec!["thc".into(), "done".into(), "t2".into()])]);
    }

    #[test]
    fn view_state_survives_a_push_for_ids_that_remain() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: two_lists() });
        update(&mut st, Msg::Key { key: "j".into() });
        let mut next = two_lists();
        next.root.children_mut().unwrap().remove(1);
        update(&mut st, Msg::Push { ui: next });
        assert_eq!(selected(&st, "a"), 1);
        assert!(!st.local.contains_key("b"));
        assert_eq!(st.version, 2);
    }

    #[test]
    fn a_bind_can_reach_into_its_source() {
        let mut st = State::default();
        let ui = ui(json!({"root": {"type": "list", "id": "l", "bind": "feed.rows.all"},
                           "data": {"feed": {"value": null}}}));
        update(&mut st, Msg::Push { ui });
        update(&mut st, Msg::Data { name: "feed".into(), result: Ok(json!({"rows": {"all": [1, 2, 3]}})) });
        assert_eq!(items(&st, st.ui.root.find("l").unwrap()).len(), 3);
    }

    #[test]
    fn clicks_focus_select_and_switch_tabs_and_focus_skips_hidden_tabs() {
        let mut st = State::default();
        let ui = ui(json!({"root": {"type": "tabs", "id": "t", "tabs": ["A", "B"], "children": [
            {"type": "list", "id": "a", "items": [1, 2, 3]},
            {"type": "list", "id": "b", "items": [1, 2, 3], "click": {"run": ["echo", "{.}"]}}
        ]}}));
        update(&mut st, Msg::Push { ui });
        assert_eq!(focus_order(&st), vec!["t", "a"]);
        update(&mut st, Msg::Click { id: "t".into(), row: None, tab: Some(1) });
        assert_eq!(focus_order(&st), vec!["t", "b"]);
        let fx = update(&mut st, Msg::Click { id: "b".into(), row: Some(2), tab: None });
        assert_eq!(st.focus.as_deref(), Some("b"));
        assert_eq!(fx, vec![Effect::Run(vec!["echo".into(), "3".into()])]);
        update(&mut st, Msg::Scroll { id: "b".into(), by: -5 });
        assert_eq!(selected(&st, "b"), 0);
    }

    #[test]
    fn an_at_bind_follows_another_nodes_selection() {
        let mut st = State::default();
        let ui = ui(json!({"root": {"type": "col", "children": [
            {"type": "list", "id": "l", "items": [{"n": "a"}, {"n": "b"}]},
            {"type": "text", "id": "d", "bind": "@l", "text": "{n}"}
        ]}}));
        update(&mut st, Msg::Push { ui });
        let detail = |st: &State| crate::model::fill("{n}", bound(st, st.ui.root.find("d").unwrap()).unwrap());
        assert_eq!(detail(&st), "a");
        update(&mut st, Msg::Key { key: "j".into() });
        assert_eq!(detail(&st), "b");
    }

    #[test]
    fn patch_replaces_one_node() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: two_lists() });
        let node: Node = serde_json::from_value(json!({"type": "list", "id": "a", "items": ["only"]})).unwrap();
        update(&mut st, Msg::Patch { id: "a".into(), node });
        assert_eq!(items(&st, st.ui.root.find("a").unwrap()), &[json!("only")]);
        update(&mut st, Msg::Patch { id: "zz".into(), node: Node::default() });
        assert!(st.status.unwrap().contains("zz"));
    }
}
