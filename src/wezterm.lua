-- fictty's ⌘ keys for WezTerm.
--
-- WezTerm doesn't send the ⌘ (super) bit to programs, even with the kitty keyboard protocol on:
-- ⌘Z arrives as `z`, ⌘← as `←`, and ⌘C copies in WezTerm without the program ever seeing it.
-- This sends those keys as the kitty protocol's bytes instead, but only in a pane where a fictty
-- screen is running (it says so with the `fictty` user var), so every other program gets
-- WezTerm's usual keys. It works for any app built on fictty, not only `fictty run`.
--
-- Installed by `fictty setup wezterm`. In your wezterm.lua, after `local config = ...`:
--
--     require('fictty').apply(config)

local wezterm = require 'wezterm'
local act = wezterm.action

local M = {}

-- The kitty keyboard protocol's modifier number is 1 + shift 1 + alt 2 + ctrl 4 + super 8.
local SUPER, SUPER_SHIFT = 9, 10

local function letter(c, m) return string.format('\x1b[%d;%du', string.byte(c), m) end
local function arrow(final, m) return string.format('\x1b[1;%d%s', m, final) end

-- Each: the key, WezTerm's mods, the bytes for a fictty screen, and what WezTerm does otherwise.
local KEYS = {
  { 'a', 'CMD', letter('a', SUPER) },
  { 'c', 'CMD', letter('c', SUPER), act.CopyTo 'Clipboard' },
  { 'x', 'CMD', letter('x', SUPER) },
  { 'z', 'CMD', letter('z', SUPER) },
  { 'z', 'CMD|SHIFT', letter('z', SUPER_SHIFT) },
  { 'LeftArrow', 'CMD', arrow('D', SUPER) },
  { 'RightArrow', 'CMD', arrow('C', SUPER) },
  { 'UpArrow', 'CMD', arrow('A', SUPER) },
  { 'DownArrow', 'CMD', arrow('B', SUPER) },
  { 'LeftArrow', 'CMD|SHIFT', arrow('D', SUPER_SHIFT) },
  { 'RightArrow', 'CMD|SHIFT', arrow('C', SUPER_SHIFT) },
  { 'UpArrow', 'CMD|SHIFT', arrow('A', SUPER_SHIFT) },
  { 'DownArrow', 'CMD|SHIFT', arrow('B', SUPER_SHIFT) },
  { 'Backspace', 'CMD', string.format('\x1b[127;%du', SUPER) },
  { 'Delete', 'CMD', string.format('\x1b[3;%d~', SUPER) },
}

-- Whether the pane is showing a fictty screen right now.
local function fictty(pane)
  local ok, vars = pcall(function() return pane:get_user_vars() end)
  return ok and vars.fictty == '1'
end

function M.apply(config)
  config.keys = config.keys or {}
  for _, k in ipairs(KEYS) do
    local key, mods, bytes, otherwise = k[1], k[2], k[3], k[4]
    table.insert(config.keys, {
      key = key,
      mods = mods,
      action = wezterm.action_callback(function(window, pane)
        if fictty(pane) then
          window:perform_action(act.SendString(bytes), pane)
        else
          window:perform_action(otherwise or act.SendKey { key = key, mods = mods }, pane)
        end
      end),
    })
  end
  return config
end

return M
