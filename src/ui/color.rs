use wgpu::util::DeviceExt;

// 此属性标注数据的内存布局兼容 C-ABI，令其可用于着色器
#[repr(C)]
// derive 属性自动导入的这些 trait，令其可被存入缓冲区
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    // glam 的数据类型不能直接用于 bytemuck
    // 需要先将矩阵转为一个浮点数数组
    color: [f32; 4],
}

impl Uniform {
    pub fn new() -> Self {
        Self {
            color: [1.0, 0.0, 0.0, 0.0],
        }
    }
}
pub struct State {
    uniform: Uniform,
    buffer: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl State {
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    pub fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.uniform.color)
    }
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    pub fn new(device: &wgpu::Device) -> Self {

        let uniform = Uniform::new();

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Color Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,     // 1
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,              // 2
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("color layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("color bind_group"),
        });

        Self {
            uniform,
            buffer, layout, bind_group
        }

    }
}