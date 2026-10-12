//! Setting terminals up for fictty. WezTerm doesn't pass ⌘ (super) to programs, so it needs a
//! small module that sends ⌘ keys to fictty screens as the kitty keyboard protocol's bytes. A
//! running screen marks its pane with the `fictty` user var, so the module serves every app built
//! on fictty, and an app's own setup command can install the same file with these functions.

use anyhow::{Context, Result};
use std::path::PathBuf;

/// The WezTerm module, `fictty.lua` (see `src/wezterm.lua`): `require('fictty').apply(config)`.
pub fn wezterm_lua() -> &'static str {
    include_str!("wezterm.lua")
}

/// Install the WezTerm module beside the person's wezterm.lua (`$XDG_CONFIG_HOME/wezterm`, else
/// `~/.config/wezterm`). Their wezterm.lua is never edited. Returns what to tell them: where it
/// went, and the line their config needs unless it has it already.
pub fn wezterm() -> Result<String> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    // An empty XDG_CONFIG_HOME counts as unset (the XDG spec).
    let base = std::env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()).map(PathBuf::from).or_else(|| home.clone().map(|h| h.join(".config")));
    let dir = base.context("no HOME to find the WezTerm config in")?.join("wezterm");
    std::fs::create_dir_all(&dir)?;
    let file = dir.join("fictty.lua");
    std::fs::write(&file, wezterm_lua())?;
    let configs = [Some(dir.join("wezterm.lua")), home.map(|h| h.join(".wezterm.lua"))];
    let wired = configs.iter().flatten().any(|c| {
        std::fs::read_to_string(c).is_ok_and(|t| ["require('fictty')", "require 'fictty'", "require(\"fictty\")"].iter().any(|r| t.contains(r)))
    });
    let mut out = format!("wrote {}\n", file.display());
    if wired {
        out.push_str("your wezterm.lua already applies it; WezTerm reloads it on save");
    } else {
        out.push_str("add this to your wezterm.lua, after the config is made and before it's returned:\n\n    require('fictty').apply(config)\n");
    }
    Ok(out)
}
