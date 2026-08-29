//! [`Window`] — winit: create the window, own the event loop, own the swapchain.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::{Window as OsWindow, WindowId};

use crate::facade::ports::Surface as Renderer;
use crate::editor::addresses;
use crate::facade::Store;
use crate::portal::device::Device;
use crate::portal::drive;
use crate::portal::input::Input;

/// The OS window.
pub struct Window;

/// The swapchain and the thing that draws into it. Present only once a window and an
/// adapter both exist; absent on a machine with no usable GPU, which runs the whole
/// loop with nothing drawn rather than failing to start.
struct Swapchain {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    device: Arc<wgpu::Device>,
    /// wgpu 30 presents through the queue, not through the frame.
    queue: Arc<wgpu::Queue>,
    renderer: Renderer,
}

struct App {
    store: Store,
    input: Input,
    device: Device,
    window: Option<Arc<OsWindow>>,
    swapchain: Option<Swapchain>,
    last_cursor: Option<(f64, f64)>,
    panning: bool,
    /// Held modifier keys, tracked so `Ctrl+Z` / `Ctrl+Shift+Z` (E12.6) can be
    /// read off a later, unrelated `KeyboardInput` event — winit reports the two
    /// separately rather than bundling modifiers onto every key event.
    modifiers: ModifiersState,
}

impl Window {
    /// Creates the window and owns the event loop until the person closes it.
    pub fn open(store: Store, device: Device) {
        let event_loop = EventLoop::new().expect("event loop");
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = App {
            store,
            input: Input,
            device,
            window: None,
            swapchain: None,
            last_cursor: None,
            panning: false,
            modifiers: ModifiersState::empty(),
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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::CursorMoved { position, .. } => {
                // Finding 15: winit reports device pixels; everything above the
                // portal is logical. Divide once, here, and the two never disagree.
                let scale = self.scale();
                let logical = (position.x / scale, position.y / scale);
                if self.panning {
                    if let Some(previous) = self.last_cursor {
                        self.store
                            .pan_by(logical.0 - previous.0, logical.1 - previous.1);
                    }
                }
                self.last_cursor = Some(logical);
                self.input
                    .on_pointer_move(&self.store, logical.0, logical.1);
                self.redraw();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let bit = match button {
                    MouseButton::Left => 1,
                    MouseButton::Right => 2,
                    MouseButton::Middle => 4,
                    _ => 0,
                };
                let flags = if state == ElementState::Pressed { bit } else { 0 };
                if button == MouseButton::Left {
                    if state == ElementState::Pressed && self.modifiers.shift_key() {
                        self.store.amend(addresses::WIRE_MODE_KEY, &[1]);
                    }
                    if state == ElementState::Released {
                        self.store.discard_at(addresses::WIRE_MODE_KEY);
                    }
                }
                if button == MouseButton::Middle {
                    self.panning = state == ElementState::Pressed;
                }
                self.input.on_pointer_button(&self.store, flags);
                self.redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let steps = match delta {
                    MouseScrollDelta::LineDelta(_, y) => f64::from(y),
                    MouseScrollDelta::PixelDelta(p) => p.y / 40.0,
                };
                self.store.zoom_by(steps);
                self.redraw();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    let pressed = u8::from(event.state == ElementState::Pressed);
                    let mut payload = format!("{code:?}").into_bytes();
                    payload.push(pressed);
                    self.input.on_key(&self.store, &payload);

                    // E12.6 — Ctrl+Z / Ctrl+Shift+Z. A held key auto-repeats
                    // (`event.repeat`), which `undo`/`redo` tolerate the same way
                    // any other "nothing left to do" call does (`None`, silently);
                    // gating on `!repeat` would need to be manual-verified anyway
                    // (R23) and would just be one more thing to get wrong before
                    // that pass.
                    if code == KeyCode::KeyZ
                        && event.state == ElementState::Pressed
                        && self.modifiers.control_key()
                    {
                        if self.modifiers.shift_key() {
                            self.store.redo();
                        } else {
                            self.store.undo();
                        }
                        self.redraw();
                    }
                }
            }
            WindowEvent::Resized(size) => {
                self.reconfigure(size.width, size.height);
                self.redraw();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(window) = &self.window {
                    let size = window.inner_size();
                    self.reconfigure(size.width, size.height);
                }
                self.redraw();
            }
            WindowEvent::RedrawRequested => {
                drive::drive(&self.store);
                self.draw();
            }
            _ => {}
        }
    }
}

impl App {
    fn scale(&self) -> f64 {
        self.window.as_ref().map(|w| w.scale_factor()).unwrap_or(1.0)
    }

    fn redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn ensure_window(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = OsWindow::default_attributes().with_title("Infinite Solutions");
        let window = Arc::new(event_loop.create_window(attrs).expect("window"));
        let size = window.inner_size();
        let scale = window.scale_factor();
        self.window = Some(Arc::clone(&window));

        if let Ok(surface) = self.device.instance().create_surface(Arc::clone(&window)) {
            if let Some((device, queue, format)) = self.device.resolve(&surface) {
                let config = default_config(format, size.width.max(1), size.height.max(1));
                surface.configure(&device, &config);
                // No presenter type crosses this line. `Renderer::attach` takes a
                // device, a queue and a format; the geometry reaches it from
                // `Store::draw_with`, which is the facade's side. R2's grep is what
                // keeps that true — `src/portal/` may not name a layer crate.
                let renderer = Renderer::attach(Arc::clone(&device), Arc::clone(&queue), format);
                self.swapchain = Some(Swapchain {
                    surface,
                    config,
                    device,
                    queue,
                    renderer,
                });
            }
        }
        self.store.set_surface(
            0.0,
            0.0,
            f64::from(size.width) / scale,
            f64::from(size.height) / scale,
            scale,
        );
        window.request_redraw();
    }

    /// E10.3: the window's geometry reaches the presenter, and the swapchain follows.
    fn reconfigure(&mut self, width: u32, height: u32) {
        let scale = self.scale();
        if let Some(swapchain) = &mut self.swapchain {
            swapchain.config.width = width.max(1);
            swapchain.config.height = height.max(1);
            swapchain
                .surface
                .configure(&swapchain.device, &swapchain.config);
        }
        self.store.set_surface(
            0.0,
            0.0,
            f64::from(width) / scale,
            f64::from(height) / scale,
            scale,
        );
    }

    fn draw(&mut self) {
        let Some(swapchain) = &mut self.swapchain else {
            self.store.draw();
            return;
        };
        match swapchain.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                swapchain.renderer.set_target(
                    view,
                    swapchain.config.width,
                    swapchain.config.height,
                );
                self.store.draw_with(&mut swapchain.renderer);
                swapchain.queue.present(frame);
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                swapchain
                    .surface
                    .configure(&swapchain.device, &swapchain.config);
            }
            _ => {}
        }
    }
}

fn default_config(
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        // Auto resolves to sRGB for an 8-bit format. `Device::resolve` already
        // picked a non-sRGB *format*, so the shader's value reaches the framebuffer
        // unencoded and the window matches `tests/pixels.rs`.
        color_space: wgpu::SurfaceColorSpace::Auto,
        width,
        height,
        present_mode: wgpu::PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: Vec::new(),
    }
}
