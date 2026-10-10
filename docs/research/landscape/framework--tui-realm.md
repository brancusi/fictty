# tui-realm (added in this pass)

- **Maker:** Christian Visintin (veeso).
- **URL:** https://github.com/veeso/tui-realm
- **Status, 10 October 2026:** about 1,000 stars, MIT. v4.1.0 (2 May 2026), v4.0.0 (18 April 2026).
  Last push 29 July 2026. Used by the author's termscp file transfer client.

## What it is

"A ratatui framework inspired by Elm and React": stateful components with properties and internal
state, a `View` that mounts components by id, an `Application` that routes events to the focused
component and returns messages to the app's `update`, and a standard library of components.

## The problem it's solving

ratatui leaves state, focus and event routing to every app. tui-realm supplies them.

## Its path / bet

Retained components keyed by id on top of immediate-mode ratatui, with Elm-style messages up and
React-style props down.

## How it works

Components implement `MockComponent` (drawing and props) and `Component` (event to message). The
`Application` holds mounted components by id, tracks focus, polls input ports (including custom
ones, for data), and calls `update` with the resulting messages. Props can be changed by id with
`attr`.

## Strengths

- It solves the same layer fictty solves inside its runtime (ids, focus, routing, ports for
  external data), on the same substrate.
- Mature (v4) and maintained.

## Weaknesses / limits

- Components are Rust code; nothing is serializable; there is no external control surface.
- Mounting by id and setting attributes is imperative.

## Relation to fictty

**Inspire.** It's a reference implementation of "focus and state by id on ratatui".

## Could fictty adopt it instead of building?

Partly, in principle: fictty could use tui-realm for focus and event routing. In practice no: its
component state lives in Rust objects outside fictty's `State`, which breaks the rule that
everything readable and replayable is in the state. fictty's own focus and selection code is small.

## What fictty should take from it

- Its **ports** (polled input sources that emit messages) are the same idea as fictty's data
  sources; compare their back-pressure and polling choices with fictty's.
- Its standard library is a checklist of components people needed on ratatui.

## Sources

- https://github.com/veeso/tui-realm (README; GitHub API 10 Oct 2026)
- https://docs.rs/tuirealm
- Not verified: the exact v4 API names (taken from earlier versions' docs).
