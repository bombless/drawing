use glam::{Mat4, vec3};
use wgpu::util::DeviceExt;

// 此属性标注数据的内存布局兼容 C-ABI，令其可用于着色器
#[repr(C)]
// derive 属性自动导入的这些 trait，令其可被存入缓冲区
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    // glam 的数据类型不能直接用于 bytemuck
    // 需要先将 Matrix4 矩阵转为一个 4x4 的浮点数数组
    proj: [[f32; 4]; 4],
    ratio: f32,
}

impl Uniform {
    pub fn new() -> Self {
        Self {
            proj: Mat4::IDENTITY.to_cols_array_2d(),
            ratio: 16.0 / 9.0,
        }
    }

    pub fn update_proj(&mut self, zoom: &Zoom) {
        let scale = Mat4::from_scale(vec3(1.0 / self.ratio, 1.0, 1.0));
        let trans = Mat4::from_translation(vec3(0.0, 0.0, 0.2));
        let scale_w = Mat4::from_cols_array_2d(&[
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 2.0],
        ]);
        let scale_z = Mat4::from_cols_array_2d(&[
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 0.5, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        let zoom = scale_z * scale_w * trans * scale * zoom.translation * zoom.camera;
        self.proj = zoom.to_cols_array_2d();
    }

    pub fn get(&self, zoom: &Zoom) -> Mat4 {
        let mut scale = Mat4::IDENTITY.to_cols_array_2d();
        scale[0][0] = 1.0 / self.ratio;
        scale[3][3] = 3.0;
        let trans = Mat4::from_translation(vec3(0.0, 0.0, 0.3));
        trans * Mat4::from_cols_array_2d(&scale) * zoom.translation * zoom.camera
    }
}

pub struct Zoom {
    translation: Mat4,
    camera: Mat4,
}

impl Zoom {
    pub fn new() -> Self {
        Self { translation: Mat4::IDENTITY, camera: Mat4::IDENTITY }
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
        self.zoom.translation = self.zoom.translation * Mat4::from_translation(glam::vec3(x, y, 0.0));
    }
    pub fn move_cam(&mut self, x: f32, y: f32) {
        let distance = (x * x + y * y).sqrt();
        let axis = vec3(y / distance, x / distance, 0.0);
        let angle = Mat4::from_axis_angle(axis, distance);
        // let rotate_x = Mat4::from_rotation_x(x);
        // let rotate_y = Mat4::from_rotation_y(y);
        // self.zoom.camera = rotate_x * rotate_y * angle * self.zoom.camera;
        self.zoom.camera = angle * self.zoom.camera;
    }
    pub fn get_mat4(&self) -> Mat4 {
        self.uniform.get(&self.zoom)
    }
    pub fn scale_x(&mut self, ratio: f32) {
        self.uniform.ratio = ratio;
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