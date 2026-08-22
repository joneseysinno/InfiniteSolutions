//! [`Device`] — wgpu. Instance, adapter, device, queue.
//!
//! D42: the portal owns the OS-shaped GPU resources — the instance, the adapter,
//! the logical device, the queue and the swapchain — because D18 makes the operating
//! system the portal's side of a boundary, and a swapchain is a property of a window.
//! The **drawing** belongs to `facade::ports::Surface`, which is the only file in
//! `src/` allowed to contain the token `f32`. Nothing here holds a float, and
//! `scripts/check-rules.sh` is what keeps that true.

use std::sync::Arc;

/// The GPU instance, and the adapter selection that needs a window to be sound.
pub struct Device {
    instance: wgpu::Instance,
}

impl Device {
    /// Requests a wgpu instance. Adapter selection waits for a window.
    ///
    /// wgpu 29 removed `Instance::default()` because a display handle is now
    /// part of instance creation. The event loop does not exist yet at `open`,
    /// so this uses `new_without_display_handle`; [`Self::resolve`] picks the
    /// adapter once a surface exists, which is what makes the choice compatible
    /// with the thing actually being drawn into.
    pub fn open() -> Self {
        Self {
            instance: wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle()),
        }
    }

    /// The instance, for a window that is creating its surface.
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    /// Picks an adapter compatible with `surface`, opens a device and a queue.
    ///
    /// Blocking is correct here and only here: this runs once, inside the event
    /// loop's `resumed`, where waiting is the loop's own concern. It is **not** on
    /// the tick path — R8 and L1 forbid that, and `check-rules.sh` greps
    /// `portal/drive.rs` for exactly this.
    ///
    /// `None` when the machine has no usable adapter. The caller runs without a
    /// device rather than dying, so "no GPU" and "nothing to draw" stay distinct.
    pub fn resolve(
        &self,
        surface: &wgpu::Surface<'static>,
    ) -> Option<(Arc<wgpu::Device>, Arc<wgpu::Queue>, wgpu::TextureFormat)> {
        let adapter = pollster::block_on(self.instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(surface),
                apply_limit_buckets: false,
            },
        ))
        .ok()?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("infinite-solutions"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        }))
        .ok()?;
        // A non-sRGB format is chosen deliberately. The shader writes the authored
        // fill straight out, so an sRGB swapchain would encode it and the pixel on
        // screen would differ from the pixel `tests/pixels.rs` asserts on. Same
        // format on both paths, or the readback stops being evidence about the
        // window.
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .or_else(|| caps.formats.first().copied())?;
        Some((Arc::new(device), Arc::new(queue), format))
    }
}
