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
    fn red() -> Self {
        Self {
            color: [1.0, 0.0, 0.0, 0.0],
        }
    }
    fn green() -> Self {
        Self {
            color: [0.0, 1.0, 0.0, 0.0],
        }
    }
}
pub struct State {
    layout: wgpu::BindGroupLayout,
    red_bind_group: wgpu::BindGroup,
    green_bind_group: wgpu::BindGroup,
}

impl State {
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
    pub fn red_bind_group(&self) -> &wgpu::BindGroup {
        &self.red_bind_group
    }
    pub fn green_bind_group(&self) -> &wgpu::BindGroup {
        &self.green_bind_group
    }
    pub fn new(device: &wgpu::Device) -> Self {

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

        let red_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Color Buffer"),
                contents: bytemuck::cast_slice(&[Uniform::red()]),
                usage: wgpu::BufferUsages::UNIFORM,
            }
        );

        let red_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: red_buffer.as_entire_binding(),
                }
            ],
            label: Some("color bind_group"),
        });


        let green_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Color Buffer"),
                contents: bytemuck::cast_slice(&[Uniform::green()]),
                usage: wgpu::BufferUsages::UNIFORM,
            }
        );

        let green_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: green_buffer.as_entire_binding(),
                }
            ],
            label: Some("color bind_group"),
        });

        Self {
            layout, red_bind_group, green_bind_group,
        }

    }
}