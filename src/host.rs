//! The host: the app that embeds fictty as a library. It gives the screen data (`{"host": name}`
//! sources) and does things for it (`{"host": name}` actions) in process, where a standalone
//! screen would run commands. Whatever the host contributes enters the state as a message, so it
//! shows in `get --state` and `$log` like everything else; the view never reads the host.

use crate::state::Msg;
use crate::store::Write;
use serde_json::Value;
use std::sync::mpsc::Sender;

/// What an embedding app provides. Every method has a default, so a host implements only what
/// it serves. `fetch` and `act` run on worker threads (inline in a headless render), never on
/// the thread that draws, so they may block on the app's own world.
pub trait Host: Send + Sync + 'static {
    /// Called once, before the first frame, with a way to tell the screen when the world changed.
    fn start(&self, notifier: Notifier) {
        let _ = notifier;
    }

    /// The value of host source `name` with its `args` (a source `{"host": name, "args": …}`).
    fn fetch(&self, name: &str, args: &Value) -> Result<Value, String> {
        let _ = args;
        Err(format!("this screen has no host source {name:?}"))
    }

    /// Do host action `name` with its filled `args` (an action `{"host": name, "args": …}`). The
    /// value it returns is logged; the screen learns about changes through the sources it reads.
    fn act(&self, name: &str, args: &Value) -> Result<Value, String> {
        let _ = args;
        Err(format!("this screen has no host action {name:?}"))
    }
}

/// A host that serves nothing: a standalone screen's.
pub struct NoHost;

impl Host for NoHost {}

/// The host's line into a running screen: cloneable, and usable from any thread.
#[derive(Clone)]
pub struct Notifier {
    send: Sender<Msg>,
}

impl Notifier {
    pub(crate) fn new(send: Sender<Msg>) -> Notifier {
        Notifier { send }
    }

    /// Host source `name` changed: every source on screen that reads it fetches again.
    pub fn changed(&self, name: &str) {
        self.msg(Msg::Changed { host: name.into() });
    }

    /// Write a store entry, as `by` (shown in the store and in `$log`).
    pub fn write(&self, key: &str, write: Write, by: &str) {
        self.msg(Msg::Write { key: key.into(), write, if_rev: None, by: by.into() });
    }

    /// Any message, as if it came from the socket. False once the screen has gone.
    pub fn msg(&self, msg: Msg) -> bool {
        self.send.send(msg).is_ok()
    }
}
