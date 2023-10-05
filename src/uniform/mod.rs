use wgpu::{Buffer, BindGroup, BindGroupLayout, Device, ShaderStages};
use wgpu::util::DeviceExt;

pub struct BufferAndBindGroupBinding {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

pub struct BufferAndBindGroupBindingVec(Vec<BufferAndBindGroupBinding>);

impl BufferAndBindGroupBindingVec {
    pub fn bind_group(&self, offset: usize) -> &BindGroup {
        &self.0[offset].bind_group
    }
    pub fn buffer(&self, offset: usize) -> &Buffer {
        &self.0[offset].buffer
    }
}

pub struct Proxy {
    buffers: Vec<BufferAndBindGroupBindingVec>,
    layout: BindGroupLayout,
}


impl Proxy {
    pub fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }
    pub fn bind_group(&self, group: usize, binding: usize) -> &BindGroup {
        &self.buffers[group].0[binding].bind_group
    }
    pub fn buffer(&self, group: usize, binding: usize) -> &Buffer {
        &self.buffers[group].0[binding].buffer
    }
    pub fn buffers(&self, group: usize) -> &BufferAndBindGroupBindingVec {
        &self.buffers[group]
    }
    pub fn new(initial_value: &[u8], count_buffers: &[usize], device: &Device, visibility: ShaderStages) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility,     // 1
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,              // 2
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("uniform layout"),
        });
        let mut buffer_groups = Vec::new();
        for &group_len in count_buffers {
            let mut buffer_bindings = Vec::new();
            for _ in 0 .. group_len {
                let buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Uniform Buffer"),
                        contents: initial_value,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    }
                );

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: buffer.as_entire_binding(),
                        }
                    ],
                    label: Some("uniform bind_group"),
                });
                buffer_bindings.push(BufferAndBindGroupBinding { buffer, bind_group, })

            }

            buffer_groups.push(BufferAndBindGroupBindingVec(buffer_bindings));

        }

        Self {
            buffers: buffer_groups,
            layout,
        }
    }
}