// Chemreact — Gray-Scott reaction-diffusion via WGSL compute + render.
// Uses nannou's wgpu wrapper API correctly.

use nannou::prelude::*;
use nannou::wgpu;
use crate::sketch::Sketch;

pub struct Chemreact {
    name: String,
    params: [f32; 16],
    textures: Option<[wgpu::TextureHandle; 2]>,
    tex_views: Option<[wgpu::TextureViewHandle; 2]>,
    current: usize,
    compute_pipeline: Option<wgpu::ComputePipeline>,
    compute_bind_group_pairs: Option<[(wgpu::BindGroup, wgpu::BindGroup); 2]>,  // (write, read) × 2
    write_bgl: Option<wgpu::BindGroupLayout>,
    read_bgl: Option<wgpu::BindGroupLayout>,
    // Render
    render_pipeline: Option<wgpu::RenderPipeline>,
    render_bind_group: Option<wgpu::BindGroup>,
    render_bgl: Option<wgpu::BindGroupLayout>,
    uniform_buffer: Option<wgpu::Buffer>,
    sampler: Option<wgpu::Sampler>,
    size: (u32, u32),
    frame: u32,
}

impl Chemreact {
    pub fn new() -> Box<Self> {
        let mut p = [0.5; 16];
        p[0] = 0.45; p[1] = 0.35; p[2] = 0.6; p[3] = 0.5; p[4] = 0.4;
        Box::new(Chemreact {
            name: "Chemreact".into(), params: p,
            textures: None, tex_views: None, current: 0, frame: 0,
            compute_pipeline: None, compute_bind_group_pairs: None, write_bgl: None, read_bgl: None,
            render_pipeline: None, render_bind_group: None, render_bgl: None,
            uniform_buffer: None, sampler: None, size: (1280, 720),
        })
    }

    fn init_wgpu(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        self.size = size;
        let (w, h) = size;

        // Ping-pong textures
        let tex_desc = wgpu::TextureDescriptor {
            label: Some("cr_tex"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let t0 = device.create_texture(&tex_desc);
        let t1 = device.create_texture(&tex_desc);
        let vd = wgpu::TextureViewDescriptor::default();
        let v0 = t0.create_view(&vd);
        let v1 = t1.create_view(&vd);
        self.textures = Some([t0, t1]);
        self.tex_views = Some([v0, v1]);

        // Sampler
        self.sampler = Some(device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }));

        // Uniform buffer
        self.uniform_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cr_uni"), size: 128,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        let ub = self.uniform_buffer.as_ref().unwrap();

        // Compute BGL
        // Write BGL: storage texture for output
        let write_bgl = wgpu::BindGroupLayoutBuilder::new()
            .storage_texture(wgpu::ShaderStages::COMPUTE, wgpu::TextureFormat::Rgba16Float, wgpu::TextureViewDimension::D2, wgpu::StorageTextureAccess::WriteOnly)
            .build(device);
        // Read BGL: regular texture (sampled) + uniforms — NOT storage
        let read_bgl = wgpu::BindGroupLayoutBuilder::new()
            .texture(wgpu::ShaderStages::COMPUTE, false, wgpu::TextureViewDimension::D2, wgpu::TextureSampleType::Float { filterable: false })
            .uniform_buffer(wgpu::ShaderStages::COMPUTE, false)
            .build(device);

        self.write_bgl = Some(write_bgl);
        self.read_bgl = Some(read_bgl);

        let views = self.tex_views.as_ref().unwrap();
        let ub = self.uniform_buffer.as_ref().unwrap();

        // Ping-pong: bg pair 0 writes to tex[1], reads from tex[0] + uniforms
        let wbg0 = wgpu::BindGroupBuilder::new()
            .texture_view(&views[1])
            .build(device, self.write_bgl.as_ref().unwrap());
        let rbg0 = wgpu::BindGroupBuilder::new()
            .texture_view(&views[0])
            .buffer_bytes(ub, 0, wgpu::BufferSize::new(128))
            .build(device, self.read_bgl.as_ref().unwrap());

        // Ping-pong: bg pair 1 writes to tex[0], reads from tex[1] + uniforms
        let wbg1 = wgpu::BindGroupBuilder::new()
            .texture_view(&views[0])
            .build(device, self.write_bgl.as_ref().unwrap());
        let rbg1 = wgpu::BindGroupBuilder::new()
            .texture_view(&views[1])
            .buffer_bytes(ub, 0, wgpu::BufferSize::new(128))
            .build(device, self.read_bgl.as_ref().unwrap());

        // Store as pairs: (write_bg, read_bg) for each frame
        self.compute_bind_group_pairs = Some([(wbg0, rbg0), (wbg1, rbg1)]);

        // Compute pipeline
        let cs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cr_cs"), source: wgpu::ShaderSource::Wgsl(COMPUTE_SHADER.into()),
        });
        let comp_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cr_comp_layout"),
            bind_group_layouts: &[self.write_bgl.as_ref().unwrap(), self.read_bgl.as_ref().unwrap()],
            push_constant_ranges: &[],
        });
        self.compute_pipeline = Some(device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("cr_comp"), layout: Some(&comp_layout),
            module: &cs, entry_point: "main",
        }));

        // Render BGL
        let render_bgl = wgpu::BindGroupLayoutBuilder::new()
            .texture(wgpu::ShaderStages::FRAGMENT, false, wgpu::TextureViewDimension::D2, wgpu::TextureSampleType::Float { filterable: true })
            .sampler(wgpu::ShaderStages::FRAGMENT, true)
            .build(device);
        self.render_bgl = Some(render_bgl);
        let render_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cr_render_layout"),
            bind_group_layouts: &[self.render_bgl.as_ref().unwrap()],
            push_constant_ranges: &[],
        });

        // Render pipeline
        let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cr_vs"), source: wgpu::ShaderSource::Wgsl(VERTEX_SHADER.into()),
        });
        let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cr_fs"), source: wgpu::ShaderSource::Wgsl(RENDER_SHADER.into()),
        });
        self.render_pipeline = Some(
            wgpu::RenderPipelineBuilder::from_layout(&render_layout, &vs)
                .fragment_shader(&fs)
                .color_format(Frame::TEXTURE_FORMAT)
                .sample_count(1)
                .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
                .build(device)
        );
        self.render_bind_group = Some(
            wgpu::BindGroupBuilder::new()
                .texture_view(&views[0])
                .sampler(self.sampler.as_ref().unwrap())
                .build(device, self.render_bgl.as_ref().unwrap())
        );

        log::info!("Chemreact WGSL ready: {w}x{h}");
    }
}

impl Sketch for Chemreact {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, _app: &nannou::App, _window: window::Id, device: Option<&wgpu::Device>, size: (u32, u32)) {
        if let Some(d) = device { self.init_wgpu(d, size); }
    }
    fn update(&mut self, _app: &nannou::App, _window: window::Id, _t: &Update, _audio: &crate::audio::AudioFeatures, _params: &[f32; 16]) {
        self.frame = self.frame.wrapping_add(1);
    }
    fn view_frame(&self, frame: &nannou::Frame) {
        let cp = match &self.compute_pipeline { Some(p) => p, None => return };
        let bg_pair = match &self.compute_bind_group_pairs { Some(bg) => &bg[self.current], None => return };
        let rp = match &self.render_pipeline { Some(p) => p, None => return };
        let rbg = match &self.render_bind_group { Some(bg) => bg, None => return };
        let mut encoder = frame.command_encoder();
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("cr") });
            cpass.set_pipeline(cp);
            cpass.set_bind_group(0, &bg_pair.0, &[]);
            cpass.set_bind_group(1, &bg_pair.1, &[]);
            cpass.dispatch_workgroups((self.size.0 + 15) / 16, (self.size.1 + 15) / 16, 1);
        }
        {
            let mut rpass = wgpu::RenderPassBuilder::new()
                .color_attachment(frame.texture_view(), |c| c)
                .begin(&mut encoder);
            rpass.set_pipeline(rp);
            rpass.set_bind_group(0, rbg, &[]);
            rpass.draw(0..4, 0..1);
        }
    }
    fn params(&self) -> &[f32; 16] { &self.params }
}

const VERTEX_SHADER: &str = r#"
@vertex
fn main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4f {
    let x = f32((vi & 1u) * 2u) - 1.0;
    let y = f32(((vi >> 1u) & 1u) * 2u) - 1.0;
    return vec4f(x, y, 0.0, 1.0);
}
"#;

const COMPUTE_SHADER: &str = r#"
struct Uniforms {
    params: array<vec4<f32>, 4>,
    audio: array<vec4<f32>, 4>,
}
@group(0) @binding(0) var dst: texture_storage_2d<rgba16float, write>;
@group(1) @binding(0) var src: texture_2d<f32>;
@group(1) @binding(1) var<uniform> u: Uniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let dims = textureDimensions(src);
    if gid.x >= dims.x || gid.y >= dims.y { return; }
    let c = vec2i(gid.xy);
    let w = i32(dims.x); let h = i32(dims.y);
    let ct = textureLoad(src, c, 0);
    let l  = textureLoad(src, vec2i((c.x - 1 + w) % w, c.y), 0);
    let r  = textureLoad(src, vec2i((c.x + 1) % w, c.y), 0);
    let u_  = textureLoad(src, vec2i(c.x, (c.y - 1 + h) % h), 0);
    let d  = textureLoad(src, vec2i(c.x, (c.y + 1) % h), 0);
    var A = ct.r; var B = ct.g;
    let dA = 0.8; let dB = 0.35;
    let feed = u.params[0][0] * 0.06 + 0.02;
    let kill = u.params[0][1] * 0.08 + 0.04;
    let lapA = (l.r + r.r + u_.r + d.r) - 4.0 * A;
    let lapB = (l.g + r.g + u_.g + d.g) - 4.0 * B;
    let react = A * B * B;
    A = clamp(A + dA * lapA - react + feed * (1.0 - A), 0.0, 1.0);
    B = clamp(B + dB * lapB + react - (kill + feed) * B, 0.0, 1.0);
    // Seed
    let uv = vec2f(gid.xy) / vec2f(dims) - 0.5;
    let s = exp(-dot(uv, uv) * 30.0);
    A = max(A, s * 0.7); B = max(B, s * 0.25);
    textureStore(dst, c, vec4f(A, B, A, 1.0));
}
"#;

const RENDER_SHADER: &str = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var smp: sampler;
@fragment
fn main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let uv = pos.xy / vec2f(textureDimensions(tex));
    let c = textureSample(tex, smp, uv);
    let u = c.r;
    return vec4f(mix(vec3f(0.02, 0.015, 0.01), vec3f(0.94, 0.88, 0.72), u), 1.0);
}
"#;
