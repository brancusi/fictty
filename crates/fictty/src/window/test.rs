//! The window as a test's frame (`App::in_window`): GPUI's test platform lays the window out and
//! hands it input with no screen, so a `Headless` drawn by it goes through the window's own
//! layout, its handover to the engine (rectangles, hits, editor sizes) and its own mouse: a
//! press at a cell is a GPUI mouse-down in the middle of that cell's pixels, which the window
//! turns into the same messages it sends on screen.
//!
//! The test platform's text has fixed metrics (every character 0.6 em), so a cell is 8.4 × 24
//! px and layout is the same on every machine; nothing is rasterised.

use super::{Cell, Engine, Portal};
use crate::engine::{Drawn, Frame, Pointer};
use crate::state::{Msg, State};
use gpui::{Modifiers, MouseButton, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase, VisualTestContext, WindowHandle, point, px, size};
use std::cell::RefCell;
use std::rc::Rc;

/// What the window and its test share: the state it draws, what it sent, what it laid out.
#[derive(Default)]
pub(super) struct Side {
    pub st: State,
    pub sent: Vec<Msg>,
    pub drawn: Option<Drawn>,
}

/// The window, drawn and driven under GPUI's test platform.
pub struct Windowed {
    cx: VisualTestContext,
    window: WindowHandle<Portal>,
    side: Rc<RefCell<Side>>,
    cell: Cell,
    /// The last frame's readback, as an outline.
    text: String,
    /// Whether the left button is down (between a press and a release).
    held: bool,
}

impl Windowed {
    /// A window of `width` × `height` cells.
    pub fn new(width: u16, height: u16) -> Windowed {
        let mut app = TestAppContext::single();
        let side = Rc::new(RefCell::new(Side::default()));
        let engine = Engine::Test(side.clone());
        let window = app.add_window(|window, cx| Portal::new(engine, window, cx));
        let mut cx = VisualTestContext::from_window(window.into(), &app);
        let cell = window.update(&mut cx, |p, _, _| p.cell).expect("the window");
        let mut w = Windowed { cx, window, side, cell, text: String::new(), held: false };
        w.resize(width, height);
        w
    }

    /// The middle of cell (`x`, `y`), in the window's pixels.
    fn at(&self, x: u16, y: u16) -> gpui::Point<gpui::Pixels> {
        point(px((x as f32 + 0.5) * self.cell.w), px((y as f32 + 0.5) * self.cell.h))
    }
}

impl Frame for Windowed {
    fn draw(&mut self, st: &State) -> Drawn {
        {
            let mut side = self.side.borrow_mut();
            side.st = st.clone();
            side.drawn = None;
        }
        // Told the state changed, the window draws when the update's effects flush.
        let _ = self.window.update(&mut self.cx, |_, _, cx| cx.notify());
        self.cx.run_until_parked();
        let drawn = self.side.borrow_mut().drawn.take().unwrap_or_default();
        self.text = crate::readback::outline(&crate::readback::readback(st, &drawn.rects));
        drawn
    }

    fn resize(&mut self, width: u16, height: u16) {
        // A half cell over, so the window's size in whole cells is exactly this.
        self.cx.simulate_resize(size(px((width as f32 + 0.5) * self.cell.w), px((height as f32 + 0.5) * self.cell.h)));
        self.cx.run_until_parked();
    }

    fn text(&self) -> String {
        self.text.clone()
    }

    fn pointer(&mut self, kind: Pointer, x: u16, y: u16) -> bool {
        let at = self.at(x, y);
        let left = MouseButton::Left;
        match kind {
            Pointer::Press { shift } => {
                // A press with the button still down from the last one lets go of it first.
                if self.held {
                    self.cx.simulate_mouse_up(at, left, Modifiers::none());
                }
                let mods = if shift { Modifiers::shift() } else { Modifiers::none() };
                self.cx.simulate_mouse_down(at, left, mods);
                self.held = true;
            }
            Pointer::Drag if self.held => self.cx.simulate_mouse_move(at, left, Modifiers::none()),
            Pointer::Drag | Pointer::Move => self.cx.simulate_mouse_move(at, None, Modifiers::none()),
            Pointer::Release => {
                self.cx.simulate_mouse_up(at, left, Modifiers::none());
                self.held = false;
            }
            Pointer::WheelUp | Pointer::WheelDown => {
                // One notch of a mouse wheel: up is a positive line in GPUI.
                let y = if kind == Pointer::WheelUp { 1.0 } else { -1.0 };
                self.cx.simulate_event(ScrollWheelEvent { position: at, delta: ScrollDelta::Lines(point(0.0, y)), modifiers: Modifiers::none(), touch_phase: TouchPhase::Moved });
            }
        }
        true
    }

    fn messages(&mut self) -> Vec<Msg> {
        std::mem::take(&mut self.side.borrow_mut().sent)
    }
}
