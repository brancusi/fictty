//! fictty: a UI that is one serializable value. The core (`fictty-core`, re-exported here) holds
//! the value, the state and the engine; this crate draws it.
//!
//! - `model`: the UI as data (a node tree, data sources, keys).
//! - `state`: the screen's state, messages and a pure `update`.
//! - `store`: the app's data, shared by the person and agents, apart from the UI.
//! - `engine`: what runs a screen (the socket, child processes, the host), with nothing that draws.
//! - `readback`: the screen as an agent reads it, node by node.
//! - `view`: state in, frame out (the terminal's grid).
//! - `runtime`: the terminal; `App` runs a screen.
//! - `window`: the native window (`fictty run --window`), macOS first.
//! - `setup`: setting terminals up for fictty (WezTerm's ⌘ keys).
//! - `host`: what an app that embeds fictty provides (sources and actions in process).
//! - `gfx`: pictures (cards, plots, images) placed with the kitty graphics protocol.

pub use fictty_core::{engine, host, lists, model, readback, state, store};
#[cfg(feature = "fuzz")]
pub use fictty_core::fuzz;

pub mod gfx;
pub mod runtime;
pub mod setup;
#[cfg(feature = "sim")]
pub mod sim;
pub mod view;
#[cfg(target_os = "macos")]
pub mod window;

pub use engine::{Headless, Live};
pub use host::{Host, NoHost, Notifier};
pub use runtime::App;
