//! [`Device`] — wgpu instance. Adapter, queue and swapchain attach when a
//! window is ready; tests never construct an event loop.

/// The GPU device.
pub struct Device {
    instance: wgpu::Instance,
}

impl Device {
    /// Requests a wgpu instance. Adapter selection waits for a window.
    ///
    /// wgpu 29 removed `Instance::default()` because a display handle is now
    /// part of instance creation. The event loop does not exist yet at `open`,
    /// so this uses `new_without_display_handle`; a window that later wants a
    /// swapchain can pass its handle when configuring the surface.
    pub fn open() -> Self {
        Self {
            instance: wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle()),
        }
    }

    /// The instance, for a window that is configuring a swapchain.
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }
}
