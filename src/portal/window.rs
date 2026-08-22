//! [`Window`] — winit: create the window, own the event loop.

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window as OsWindow, WindowId};

use crate::facade::Store;
use crate::portal::drive;
use crate::portal::input::Input;

/// The OS window.
pub struct Window;

struct App {
    store: Store,
    input: Input,
    window: Option<OsWindow>,
}

impl Window {
    /// Creates the window and owns the event loop until the person closes it.
    pub fn open(store: Store) {
        let event_loop = EventLoop::new().expect("event loop");
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = App {
            store,
            input: Input,
            window: None,
        };
        event_loop.run_app(&mut app).expect("event loop run");
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.ensure_window(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.ensure_window(event_loop);
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::CursorMoved { position, .. } => {
                self.input
                    .on_pointer_move(&self.store, position.x, position.y);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let bit = match button {
                    MouseButton::Left => 1,
                    MouseButton::Right => 2,
                    MouseButton::Middle => 4,
                    _ => 0,
                };
                let flags = if state == ElementState::Pressed {
                    bit
                } else {
                    0
                };
                self.input.on_pointer_button(&self.store, flags);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    let pressed = u8::from(event.state == ElementState::Pressed);
                    let mut payload = format!("{code:?}").into_bytes();
                    payload.push(pressed);
                    self.input.on_key(&self.store, &payload);
                }
            }
            WindowEvent::Resized(size) => {
                let scale = self.window.as_ref().map(|w| w.scale_factor()).unwrap_or(1.0);
                self.input.on_resize(
                    &self.store,
                    f64::from(size.width),
                    f64::from(size.height),
                    scale,
                );
            }
            WindowEvent::RedrawRequested => {
                drive::drive(&self.store);
                self.store.draw();
            }
            _ => {}
        }
    }
}

impl App {
    fn ensure_window(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = OsWindow::default_attributes().with_title("Infinite Solutions");
        let window = event_loop.create_window(attrs).expect("window");
        window.request_redraw();
        self.window = Some(window);
    }
}
