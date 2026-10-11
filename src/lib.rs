//! fictty: a terminal UI that is one serializable value.
//!
//! - `model`: the UI as data (a node tree, data sources, keys).
//! - `state`: the screen's state, messages and a pure `update`.
//! - `store`: the app's data, shared by the person and agents, apart from the UI.
//! - `view`: state in, frame out.
//! - `runtime`: the terminal, the control socket, child processes; `App` runs a screen.
//! - `setup`: setting terminals up for fictty (WezTerm's ⌘ keys).
//! - `host`: what an app that embeds fictty provides (sources and actions in process).
//! - `gfx`: pictures (cards, plots, images) placed with the kitty graphics protocol.

#[cfg(feature = "fuzz")]
pub mod fuzz;
pub mod gfx;
pub mod host;
pub mod lists;
pub mod model;
pub mod runtime;
pub mod setup;
#[cfg(feature = "sim")]
pub mod sim;
pub mod state;
pub mod store;
pub mod view;

pub use host::{Host, NoHost, Notifier};
pub use runtime::{App, Headless};
