//! [`Surface`] — presenter. wgpu.
//!
//! **The only f64 → f32 narrowing in the repository** (D29, `PRESENTER.md` §3.3),
//! and the only file in `src/` that may contain the token `f32`. The WGSL below is
//! part of that: a shader is `f32` all the way down, so it lives here or the check
//! that pins the narrowing point stops meaning anything.
//!
//! D42: the **portal** owns the instance, adapter, device, queue and swapchain,
//! because those are OS-shaped resources and D18 makes the OS the portal's. This
//! file owns the **drawing** — pipeline, buffers, encoder, pass — and borrows the
//! device it was attached with. `PRESENTER.md` §9's split one level further down.
//!
//! D44: fills arrive resolved. `Placed` carries no style by design and this file may
//! not name the editor (R2), so `Store::draw_with` resolves address → fill from the
//! `SceneSet` it already holds and hands the result to [`Surface::set_fills`].

use std::collections::BTreeMap;
use std::sync::Arc;

use infinite_presenter::binding::ports::Surface as Port;
use infinite_presenter::core::{Addr, Placement, Point, SurfaceRect, TEXT};

use crate::facade::ports::text::TextRenderer;

/// Bytes per area instance: four f32 of rectangle, four f32 of fill.
const INSTANCE_STRIDE: u64 = 32;

/// Bytes per link instance: four f32 of segment, four f32 of fill, four f32 of shape
/// (half-width, then room the next link primitive that needs it will use).
const LINK_STRIDE: u64 = 48;

/// The primitive key a link draws under (D46). The facade's half of the registry:
/// the presenter authors the key, this file resolves it to a pipeline, and neither
/// side names an enum (R16).
const LINK: &str = "wire";

/// The default fill for a style key with no authored row. Grey, and visible —
/// `PRESENTER.md` §13 finding 8: a failed lookup and an empty screen must never be
/// indistinguishable.
const UNKNOWN_FILL: [f64; 4] = [0.55, 0.55, 0.55, 1.0];

const SHADER: &str = r#"
struct Inst {
    @location(0) rect: vec4<f32>,
    @location(1) fill: vec4<f32>,
};

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) fill: vec4<f32>,
};

// x, y: drawable size in device pixels. z: device pixels per logical unit.
@group(0) @binding(0) var<uniform> drawable: vec4<f32>;

@vertex
fn vs(@builtin(vertex_index) vi: u32, inst: Inst) -> VsOut {
    var corner = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
    );
    let c = corner[vi];
    let logical = inst.rect.xy + c * inst.rect.zw;
    let device_px = logical * drawable.z;
    let ndc = vec2<f32>(
        device_px.x / drawable.x * 2.0 - 1.0,
        1.0 - device_px.y / drawable.y * 2.0,
    );
    var out: VsOut;
    out.pos = vec4<f32>(ndc, 0.0, 1.0);
    out.fill = inst.fill;
    return out;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    return in.fill;
}
"#;

/// E11's second primitive. A segment, expanded to a quad about its own direction,
/// so one instanced draw covers every wire in a batch exactly as the quad pipeline
/// covers every rectangle.
const LINK_SHADER: &str = r#"
struct Inst {
    @location(0) seg: vec4<f32>,
    @location(1) fill: vec4<f32>,
    @location(2) shape: vec4<f32>,
};

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) fill: vec4<f32>,
};

@group(0) @binding(0) var<uniform> drawable: vec4<f32>;

@vertex
fn vs(@builtin(vertex_index) vi: u32, inst: Inst) -> VsOut {
    let a = inst.seg.xy;
    let b = inst.seg.zw;
    let along = b - a;
    let len = max(length(along), 1e-6);
    let unit = along / len;
    let side = vec2<f32>(-unit.y, unit.x) * max(inst.shape.x, 0.5);
    var corner = array<vec2<f32>, 4>(
        vec2<f32>(0.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
    );
    let c = corner[vi];
    let logical = a + along * c.x + side * c.y;
    let device_px = logical * drawable.z;
    let ndc = vec2<f32>(
        device_px.x / drawable.x * 2.0 - 1.0,
        1.0 - device_px.y / drawable.y * 2.0,
    );
    var out: VsOut;
    out.pos = vec4<f32>(ndc, 0.0, 1.0);
    out.fill = inst.fill;
    return out;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    return in.fill;
}
"#;

/// The GPU objects this file owns. Absent in the headless-arithmetic case, which is
/// what every test before E10.1 used and what still runs when no device is attached.
struct Gpu {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipeline: wgpu::RenderPipeline,
    links: wgpu::RenderPipeline,
    bind: wgpu::BindGroup,
    uniform: wgpu::Buffer,
    instances: wgpu::Buffer,
    capacity: u64,
    link_instances: wgpu::Buffer,
    link_capacity: u64,
}

/// One batch, after narrowing: the pipeline it wants, and where its instances sit
/// in the buffer that pipeline reads.
struct Run {
    link: bool,
    first: u32,
    count: u32,
}

/// The thing being drawn into.
pub struct Surface {
    geometry: SurfaceRect,
    gpu: Option<Gpu>,
    target: Option<wgpu::TextureView>,
    /// Drawable size in device pixels. The swapchain's, not the placement's.
    target_px: (u32, u32),
    clear: [f64; 4],
    fills: BTreeMap<Addr, [f64; 4]>,
    /// Address → run, for the text primitive batch (E13.0 / E14).
    text_runs: BTreeMap<Addr, Box<str>>,
    /// Shaped-text rasteriser (E14). Absent when no device is attached.
    text: Option<TextRenderer>,
    /// Owned only by [`Self::offscreen`], so [`Self::read_back`] has something to copy.
    offscreen: Option<wgpu::Texture>,
    /// Last frame, after narrowing. Kept so a test can see the f32 path ran.
    narrowed: usize,
}

impl Surface {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::with_geometry(SurfaceRect::new(
            Point::ORIGIN,
            Point::new(800.0, 600.0),
            1.0,
        ))
    }

    /// A surface with no device. Reports geometry, narrows, draws nothing.
    ///
    /// This is what every pre-E10 test used, and keeping it is what lets the
    /// arithmetic be exercised without standing up a GPU — D29's whole argument.
    pub(crate) fn with_geometry(geometry: SurfaceRect) -> Self {
        Self {
            geometry,
            gpu: None,
            target: None,
            target_px: (0, 0),
            clear: [0.0, 0.0, 0.0, 1.0],
            fills: BTreeMap::new(),
            text_runs: BTreeMap::new(),
            text: None,
            offscreen: None,
            narrowed: 0,
        }
    }

    /// Attaches the device the portal resolved (D42). Builds the pipeline once.
    ///
    /// Takes no geometry: the portal may not name a layer crate (R2), and
    /// `SurfaceRect` is the presenter's. `Store::draw_with` sets it every frame from
    /// the one place it lives, which is `Store::set_surface` (D43).
    pub fn attach(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        format: wgpu::TextureFormat,
    ) -> Self {
        let text = TextRenderer::new(&device, &queue, format);
        let gpu = build(device, queue, format);
        Self {
            geometry: SurfaceRect::new(Point::ORIGIN, Point::new(800.0, 600.0), 1.0),
            gpu: Some(gpu),
            target: None,
            target_px: (0, 0),
            clear: [0.0, 0.0, 0.0, 1.0],
            fills: BTreeMap::new(),
            text_runs: BTreeMap::new(),
            text: Some(text),
            offscreen: None,
            narrowed: 0,
        }
    }

    /// A surface that owns its own device and renders into a texture it can read back.
    ///
    /// **This is E10.1's green check.** It needs no window and no display server, so
    /// it runs wherever the test suite runs, and it is the only check in the
    /// repository that can fail for the reason "nothing was drawn" (D41).
    ///
    /// `None` when no adapter is available at all, so a machine with no GPU stack
    /// skips rather than fails — an absent adapter is not a defect in this code.
    pub fn offscreen(width_px: u32, height_px: u32, geometry: SurfaceRect) -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: None,
            apply_limit_buckets: false,
        }))
        .ok()?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("infinite-solutions offscreen"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        }))
        .ok()?;
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("readback target"),
            size: wgpu::Extent3d {
                width: width_px,
                height: height_px,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut surface = Self::attach(Arc::new(device), Arc::new(queue), format);
        surface.geometry = geometry;
        surface.target = Some(view);
        surface.target_px = (width_px, height_px);
        surface.offscreen = Some(texture);
        Some(surface)
    }

    /// The swapchain texture this frame draws into, and its size in device pixels.
    pub fn set_target(&mut self, view: wgpu::TextureView, width_px: u32, height_px: u32) {
        self.target = Some(view);
        self.target_px = (width_px, height_px);
    }

    /// The drawable rectangle, in logical units. The portal sets this on resize.
    pub fn set_geometry(&mut self, geometry: SurfaceRect) {
        self.geometry = geometry;
    }

    /// The background, as an authored fill (E10.2). Never a constant.
    pub fn set_clear(&mut self, fill: [f64; 4]) {
        self.clear = fill;
    }

    /// Address → fill, resolved by the facade (D44). Keyed by address, never by a
    /// style name: L5 and `check-rules.sh`'s `maps_keyed_by_addr` both require it.
    pub fn set_fills(&mut self, fills: BTreeMap<Addr, [f64; 4]>) {
        self.fills = fills;
    }

    /// Address → run for text batches. Resolved by the facade from the scene set.
    pub fn set_text_runs(&mut self, runs: BTreeMap<Addr, Box<str>>) {
        self.text_runs = runs;
    }

    /// How many vertices were narrowed this submit. For the agreement test.
    pub fn narrowed_count(&self) -> usize {
        self.narrowed
    }

    /// Whether a device is attached. A frame without one draws nothing, and saying
    /// so is what keeps "no GPU" from looking like "nothing to draw".
    pub fn has_device(&self) -> bool {
        self.gpu.is_some()
    }

    /// The rendered pixels, row-major RGBA8, from a surface built by
    /// [`Self::offscreen`]. `None` for any other surface.
    pub fn read_back(&self) -> Option<Vec<u8>> {
        let gpu = self.gpu.as_ref()?;
        let texture = self.offscreen.as_ref()?;
        let (w, h) = self.target_px;
        // copy_texture_to_buffer wants rows aligned to 256 bytes; the caller wants
        // them packed, so the padding is added for the copy and stripped after.
        let packed = w * 4;
        let padded = packed.div_ceil(256) * 256;
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(padded) * u64::from(h),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue.submit(Some(encoder.finish()));
        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let view = slice.get_mapped_range().ok()?;
        let mut out = Vec::with_capacity((packed * h) as usize);
        for row in 0..h {
            let start = (row * padded) as usize;
            out.extend_from_slice(&view[start..start + packed as usize]);
        }
        drop(view);
        buffer.unmap();
        Some(out)
    }

    /// RGBA at one device pixel of a read-back frame.
    pub fn pixel(pixels: &[u8], width_px: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * width_px + x) * 4) as usize;
        [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
    }
}

impl Port for Surface {
    fn geometry(&self) -> SurfaceRect {
        self.geometry
    }

    fn submit(&mut self, placement: &Placement) {
        // The narrowing point. f64 world → f32 device, once, here.
        //
        // D46: the placement arrives already grouped, so this loop selects a pipeline
        // per batch and invents no grouping of its own. An unknown primitive key
        // falls through to the area pipeline rather than drawing nothing — a key with
        // no pipeline and an empty screen must not look the same (`PRESENTER.md` §13
        // finding 8), and its bounding box is the honest fallback shape.
        //
        // E14: text is queued for glyphon rather than expanded to ink-cell quads.
        let mut instances: Vec<u8> = Vec::with_capacity(placement.placed.len() * 32);
        let mut link_instances: Vec<u8> = Vec::new();
        let mut runs: Vec<Run> = Vec::new();
        let mut count = 0u32;
        let mut link_count = 0u32;
        let mut text_draws: Vec<TextDraw> = Vec::new();
        for batch in &placement.batches {
            let is_link = &*batch.primitive == LINK;
            let is_text = &*batch.primitive == TEXT;
            let first = if is_link { link_count } else { count };
            let mut emitted = 0u32;
            for item in placement
                .placed
                .iter()
                .skip(batch.first)
                .take(batch.count)
            {
                let showing = item.showing();
                if showing.is_empty() {
                    continue;
                }
                let fill = self.fills.get(&item.at).copied().unwrap_or(UNKNOWN_FILL);
                match (is_link, is_text, item.span) {
                    (true, _, Some((a, b))) => {
                        // The stroke's half-width, recovered from the unclipped
                        // bounding box `place_link` inflated by exactly that amount.
                        // Not from `showing`: a clipped wire has a narrower box and
                        // would come back thinner, which is a bug that only appears
                        // once something is half off screen.
                        let half = (a.x.min(b.x) - item.rect.min.x).max(0.5);
                        let inst: [f32; 12] = [
                            a.x as f32,
                            a.y as f32,
                            b.x as f32,
                            b.y as f32,
                            fill[0] as f32,
                            fill[1] as f32,
                            fill[2] as f32,
                            fill[3] as f32,
                            half as f32,
                            0.0,
                            0.0,
                            0.0,
                        ];
                        for n in inst {
                            link_instances.extend_from_slice(&n.to_le_bytes());
                        }
                        link_count += 1;
                    }
                    (_, true, _) => {
                        let Some(run) = self.text_runs.get(&item.at) else {
                            continue;
                        };
                        let em = (showing.max.y - showing.min.y).max(1e-12);
                        let clip = item.clip.map(|c| {
                            (
                                c.min.x.floor() as i32,
                                c.min.y.floor() as i32,
                                c.max.x.ceil() as i32,
                                c.max.y.ceil() as i32,
                            )
                        });
                        text_draws.push(TextDraw {
                            run: run.clone(),
                            left: showing.min.x as f32,
                            top: showing.min.y as f32,
                            em: em as f32,
                            fill: [
                                fill[0] as f32,
                                fill[1] as f32,
                                fill[2] as f32,
                                fill[3] as f32,
                            ],
                            clip,
                        });
                    }
                    _ => {
                        let quad: [f32; 8] = [
                            showing.min.x as f32,
                            showing.min.y as f32,
                            (showing.max.x - showing.min.x) as f32,
                            (showing.max.y - showing.min.y) as f32,
                            fill[0] as f32,
                            fill[1] as f32,
                            fill[2] as f32,
                            fill[3] as f32,
                        ];
                        for n in quad {
                            instances.extend_from_slice(&n.to_le_bytes());
                        }
                        count += 1;
                    }
                }
                emitted += 1;
            }
            if is_text {
                // Text is drawn by glyphon after the quad pass — no instance run.
                continue;
            }
            if is_link {
                if link_count > first {
                    runs.push(Run {
                        link: true,
                        first,
                        count: link_count - first,
                    });
                } else if emitted > 0 {
                    runs.push(Run {
                        link: false,
                        first: count.saturating_sub(emitted),
                        count: emitted,
                    });
                }
            } else if count > first {
                runs.push(Run {
                    link: false,
                    first,
                    count: count - first,
                });
            }
        }
        self.narrowed = (count + link_count) as usize * 4 + text_draws.len() * 4;

        if self.gpu.is_none() || self.target.is_none() {
            return;
        }
        let (width_px, height_px) = self.target_px;
        if width_px == 0 || height_px == 0 {
            return;
        }

        // Take text out so gpu and text are not borrowed from self at once.
        let mut text = self.text.take();
        let scale = self.geometry.scale_factor;
        let clear = self.clear;
        {
            let gpu = self.gpu.as_mut().expect("gpu checked above");
            let target = self.target.as_ref().expect("target checked above");

            let needed = u64::from(count).max(1) * INSTANCE_STRIDE;
            if needed > gpu.capacity {
                gpu.capacity = needed.next_power_of_two();
                gpu.instances = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("instances"),
                    size: gpu.capacity,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }
            if !instances.is_empty() {
                gpu.queue.write_buffer(&gpu.instances, 0, &instances);
            }
            let link_needed = u64::from(link_count).max(1) * LINK_STRIDE;
            if link_needed > gpu.link_capacity {
                gpu.link_capacity = link_needed.next_power_of_two();
                gpu.link_instances = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("link instances"),
                    size: gpu.link_capacity,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }
            if !link_instances.is_empty() {
                gpu.queue
                    .write_buffer(&gpu.link_instances, 0, &link_instances);
            }
            let uniform: [f32; 4] = [
                width_px as f32,
                height_px as f32,
                scale as f32,
                0.0,
            ];
            let mut uniform_bytes = Vec::with_capacity(16);
            for n in uniform {
                uniform_bytes.extend_from_slice(&n.to_le_bytes());
            }
            gpu.queue.write_buffer(&gpu.uniform, 0, &uniform_bytes);

            if let Some(text) = text.as_mut() {
                let logical_w = (width_px as f64 / scale.max(1e-12)).ceil().max(1.0) as u32;
                let logical_h = (height_px as f64 / scale.max(1e-12)).ceil().max(1.0) as u32;
                text.resize(logical_w, logical_h, scale as f32);
                text.clear_pending();
                for draw in &text_draws {
                    text.queue_text(
                        &draw.run,
                        draw.left,
                        draw.top,
                        draw.em,
                        400,
                        draw.fill,
                        draw.clip,
                    );
                }
                text.prepare(&gpu.device, &gpu.queue);
            }

            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("frame"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: target,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: clear[0],
                                g: clear[1],
                                b: clear[2],
                                a: clear[3],
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                // Four corners as a strip, one instance per placed thing. Draw order is
                // the placement's order, which is address order within a level and level
                // order across levels (`PRESENTER.md` §9). No z-index — and the batches
                // are a partition of that order, so drawing them in turn preserves it.
                pass.set_bind_group(0, &gpu.bind, &[]);
                for run in &runs {
                    if run.link {
                        pass.set_pipeline(&gpu.links);
                        pass.set_vertex_buffer(0, gpu.link_instances.slice(..));
                    } else {
                        pass.set_pipeline(&gpu.pipeline);
                        pass.set_vertex_buffer(0, gpu.instances.slice(..));
                    }
                    pass.draw(0..4, run.first..run.first + run.count);
                }
                // Text after rects — Innovator's proven order.
                if let Some(text) = text.as_ref() {
                    let _ = text.render_into(&mut pass);
                }
            }
            gpu.queue.submit(Some(encoder.finish()));
        }
        if let Some(text) = text.as_mut() {
            text.trim();
        }
        self.text = text;
    }
}

/// One text run queued for glyphon after the instance walk.
struct TextDraw {
    run: Box<str>,
    left: f32,
    top: f32,
    em: f32,
    fill: [f32; 4],
    clip: Option<(i32, i32, i32, i32)>,
}

fn build(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, format: wgpu::TextureFormat) -> Gpu {
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("drawable"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("drawable"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("drawable"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        }],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("quad"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("quad"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("quad"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: INSTANCE_STRIDE,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 16,
                        shader_location: 1,
                    },
                ],
            })],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    let link_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("link"),
        source: wgpu::ShaderSource::Wgsl(LINK_SHADER.into()),
    });
    let links = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("link"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &link_shader,
            entry_point: Some("vs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: LINK_STRIDE,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 16,
                        shader_location: 1,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 32,
                        shader_location: 2,
                    },
                ],
            })],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &link_shader,
            entry_point: Some("fs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    let capacity = 64 * INSTANCE_STRIDE;
    let instances = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("instances"),
        size: capacity,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let link_capacity = 16 * LINK_STRIDE;
    let link_instances = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("link instances"),
        size: link_capacity,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    Gpu {
        device,
        queue,
        pipeline,
        links,
        bind,
        uniform,
        instances,
        capacity,
        link_instances,
        link_capacity,
    }
}
