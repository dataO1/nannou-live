// Chemreact — Gray-Scott reaction-diffusion via WGSL compute shader.
// Self-contained scene: owns its pipelines, textures, bind groups.
// Framework just provides audio features and calls view_frame().

use nannou::prelude::*;
use nannou::wgpu;
use crate::sketch::Sketch;

pub struct Chemreact {
    name: String,
    params: [f32; 16],
    textures: Option<[wgpu::Texture; 2]>,
    current: usize,
    compute_pipeline: Option<wgpu::ComputePipeline>,
    compute_bind_groups: Option<[wgpu::BindGroup; 2]>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    render_bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: Option<wgpu::Buffer>,
    sampler: Option<wgpu::Sampler>,
    size: [u32; 2],
}

impl Chemreact {
    pub fn new() -> Box<Self> {
        let mut params = [0.5; 16];
        params[0] = 0.45; params[1] = 0.35; params[2] = 0.6;
        params[3] = 0.5;  params[4] = 0.4;
        Box::new(Chemreact {
            name: "Chemreact".into(), params,
            textures: None, current: 0,
            compute_pipeline: None, compute_bind_groups: None,
            render_pipeline: None, render_bind_group: None,
            uniform_buffer: None, sampler: None,
            size: [1280, 720],
        })
    }

    fn init_wgpu(&mut self, device: &wgpu::Device, size: [u32; 2]) {
        self.size = size;
        let w = size[0]; let h = size[1];

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
        self.textures = Some([device.create_texture(&tex_desc), device.create_texture(&tex_desc)]);

        let tex = self.textures.as_ref().unwrap();

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
            label: Some("cr_uni"),
            size: 128,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // Compute bind group layout
        let comp_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cr_comp_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::StorageTexture { access: wgpu::StorageTextureAccess::WriteOnly, format: wgpu::TextureFormat::Rgba16Float, view_dimension: wgpu::TextureViewDimension::D2 }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::StorageTexture { access: wgpu::StorageTextureAccess::ReadOnly, format: wgpu::TextureFormat::Rgba16Float, view_dimension: wgpu::TextureViewDimension::D2 }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });

        let ub = self.uniform_buffer.as_ref().unwrap();

        // Two bind groups for ping-pong
        let bg0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cr_bg0"), layout: &comp_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&tex[1].create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&tex[0].create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 2, resource: ub.as_entire_binding() },
            ],
        });
        let bg1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cr_bg1"), layout: &comp_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&tex[0].create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&tex[1].create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 2, resource: ub.as_entire_binding() },
            ],
        });
        self.compute_bind_groups = Some([bg0, bg1]);

        // Compute pipeline
        let cs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cr_cs"),
            source: wgpu::ShaderSource::Wgsl(COMPUTE_SHADER.into()),
        });
        let comp_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cr_comp_layout"),
            bind_group_layouts: &[&comp_bgl],
            push_constant_ranges: &[],
        });
        self.compute_pipeline = Some(device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("cr_comp"),
            layout: Some(&comp_layout),
            module: &cs,
            entry_point: "main",
        }));

        // Render pipeline
        let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cr_vs"),
            source: wgpu::ShaderSource::Wgsl(VERTEX_SHADER.into()),
        });
        let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cr_fs"),
            source: wgpu::ShaderSource::Wgsl(RENDER_SHADER.into()),
        });
        let render_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cr_render_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let render_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cr_render_layout"),
            bind_group_layouts: &[&render_bgl],
            push_constant_ranges: &[],
        });
        self.render_pipeline = Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cr_render"),
            layout: Some(&render_layout),
            vertex: wgpu::VertexState { module: &vs, entry_point: "main", buffers: &[] },
            fragment: Some(wgpu::FragmentState { module: &fs, entry_point: "main", targets: &[Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba16Float, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, ..Default::default() },
            depth_stencil: None, multisample: Default::default(),
            multiview: None,
        }));
        self.render_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cr_render_bg"), layout: &render_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&tex[0].create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(self.sampler.as_ref().unwrap()) },
            ],
        }));

        log::info!("Chemreact WGSL ready: {w}x{h}");
    }
}

impl Sketch for Chemreact {
    fn name(&self) -> &str { &self.name }

    fn init(&mut self, _app: &nannou::App, _window: window::Id, device: Option<&wgpu::Device>, size: [u32; 2]) {
        if let Some(d) = device { self.init_wgpu(d, size); }
    }

    fn view_frame(&self, frame: &nannou::Frame) {
        let tex = match &self.textures { Some(t) => t, None => return };
        let cp = match &self.compute_pipeline { Some(p) => p, None => return };
        let cbg = match &self.compute_bind_groups { Some(bg) => &bg[self.current], None => return };
        let rp = match &self.render_pipeline { Some(p) => p, None => return };
        let rbg = match &self.render_bind_group { Some(bg) => bg, None => return };

        let mut encoder = frame.command_encoder();

        // Compute
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("cr") });
            cpass.set_pipeline(cp);
            cpass.set_bind_group(0, cbg, &[]);
            let w = (self.size[0] + 15) / 16;
            let h = (self.size[1] + 15) / 16;
            cpass.dispatch_workgroups(w, h, 1);
        }

        // Render
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("cr_render"),
                color_attachments: &[Some(frame.color_attachment_descriptor())],
                depth_stencil_attachment: None,
            });
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
    params: array<f32, 16>,
    audio: array<f32, 16>,
}

@group(0) @binding(0) var dst: texture_storage_2d<rgba16float, write>;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var<uniform> u: Uniforms;

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
    let feed = u.params[0] * 0.06 + 0.02;
    let kill = u.params[1] * 0.08 + 0.04;

    let lapA = (l.r + r.r + u_.r + d.r) - 4.0 * A;
    let lapB = (l.g + r.g + u_.g + d.g) - 4.0 * B;
    let react = A * B * B;

    A = clamp(A + dA * lapA - react + feed * (1.0 - A), 0.0, 1.0);
    B = clamp(B + dB * lapB + react - (kill + feed) * B, 0.0, 1.0);

    // Seed on first frames
    let uv = vec2f(gid.xy) / vec2f(dims) - 0.5;
    let seed = exp(-length(uv) * 12.0) * 0.7;
    A = max(A, seed * step(0.0, u.audio[14]));
    B = max(B, seed * 0.25 * step(0.0, u.audio[14]));

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
    let bg = vec3f(0.02, 0.015, 0.01);
    let cream = vec3f(0.94, 0.88, 0.72);
    return vec4f(mix(bg, cream, u), 1.0);
}
"#;
