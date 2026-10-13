//! fictty's core: a UI that is one serializable value, with nothing that draws it.
//!
//! - `model`: the UI as data (a node tree, data sources, keys) and the template language.
//! - `state`: the screen's state, messages and a pure `update`.
//! - `store`: the app's data, shared by the person and agents, apart from the UI.
//! - `engine`: what runs a screen: the control socket, child processes, the host; `App` starts
//!   one, `Live` runs it for whatever draws it, `Headless` drives it in tests.
//! - `readback`: the screen as an agent reads it, node by node.
//! - `host`: what an app that embeds fictty provides (sources and actions in process).
//! - `lists`: Markdown lists as text.
//! - `style`: each shown node's style, resolved from the active theme.

pub mod engine;
#[cfg(feature = "fuzz")]
pub mod fuzz;
pub mod host;
pub mod lists;
pub mod model;
pub mod readback;
pub mod state;
pub mod store;
pub mod style;

pub use engine::{App, Headless, Live};
pub use host::{Host, NoHost, Notifier};
