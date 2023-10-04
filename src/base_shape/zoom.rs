use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

// 此属性标注数据的内存布局兼容 C-ABI，令其可用于着色器
#[repr(C)]
// derive 属性自动导入的这些 trait，令其可被存入缓冲区
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    // glam 的数据类型不能直接用于 bytemuck
    // 需要先将 Matrix4 矩阵转为一个 4x4 的浮点数数组
    proj: [[f32; 4]; 4],
}

impl Uniform {
    pub fn new() -> Self {
        Self {
            proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_proj(&mut self, zoom: &Zoom) {
        self.proj = zoom.zoom.to_cols_array_2d();
    }
}

pub struct Zoom {
    zoom: Mat4,
}

impl Zoom {
    pub fn new() -> Self {
        Self { zoom: glam::Mat4::IDENTITY }
    }
}

pub struct State {
    zoom: Zoom,
    uniform: Uniform,
    buffer: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl State {
    pub fn update_proj(&mut self) {
        self.uniform.update_proj(&self.zoom);
    }
    pub fn translation(&mut self, x: f32, y: f32) {
        self.zoom.zoom = self.zoom.zoom * Mat4::from_translation(glam::vec3(x, y, 0.0));
    }
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    pub fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.uniform.proj)
    }
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    pub fn new(device: &wgpu::Device) -> Self {

        let zoom = Zoom::new();

        let mut uniform = Uniform::new();
        uniform.update_proj(&zoom);

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Zoom Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,     // 1
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,              // 2
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("zoom_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("zoom_bind_group"),
        });

        Self {
            zoom,
            uniform,
            buffer, layout, bind_group
        }

    }
}