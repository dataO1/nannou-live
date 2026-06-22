// Effect pipeline — wraps a WGSL compute + render pass with ping-pong textures.
// One effect = one visual scene. Multiple effects can share textures via the
// EffectGraph for cross-effect feedback.

use nannou::wgpu;

/// A self-contained visual effect with compute + render + feedback textures.
pub struct EffectPipeline {
    /// Ping-pong storage textures for feedback (e.g. Gray-Scott U/V state)
    pub textures: [wgpu::Texture; 2],
    pub current_tex: usize, // 0 or 1 — which is the "previous frame" (read source)
    /// Compute pipeline (Gray-Scott step, flow field, etc.)
    pub compute_pipeline: wgpu::ComputePipeline,
    /// Compute bind group (textures + uniforms)
    pub compute_bind_group: wgpu::BindGroup,
    /// Render pipeline (visualization pass)
    pub render_pipeline: wgpu::RenderPipeline,
    /// Render bind group
    pub render_bind_group: wgpu::BindGroup,
    /// Uniform buffer (16 params + 16 audio features = 128 bytes)
    pub uniform_buffer: wgpu::Buffer,
}

impl EffectPipeline {
    /// Swap ping-pong textures after each frame.
    pub fn swap(&mut self) {
        self.current_tex = 1 - self.current_tex;
    }

    /// Dispatch the compute pass.
    pub fn dispatch_compute<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("effect_compute"),
            timestamp_writes: None,
        });
        cpass.set_pipeline(&self.compute_pipeline);
        cpass.set_bind_group(0, &self.compute_bind_group, &[]);
        // 16×16 workgroups covering a 1280×720 texture = 80×45
        cpass.dispatch_workgroups(80, 45, 1);
    }
}

/// Build a ping-pong texture pair for feedback.
pub fn create_effect_textures(device: &wgpu::Device, width: u32, height: u32) -> [wgpu::Texture; 2] {
    let desc = |label: &str| wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    };
    [device.create_texture(&desc("effect_tex_0")),
     device.create_texture(&desc("effect_tex_1"))]
}

/// Build a uniform buffer: 16 params + 15 audio features + 1 pad = 32 f32s = 128 bytes.
pub fn create_uniform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("effect_uniforms"),
        size: 128,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
