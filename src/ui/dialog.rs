use glam::{Mat4, vec3};
use wgpu::{Buffer, Device, Queue};
use wgpu::util::DeviceExt;

use crate::ui::utils::{draw_line, scale_x};

pub struct State {
    vertices: Vec<f32>,
    vertices_buffer: Buffer,
    indices: Vec<u16>,
    indices_buffer: Buffer,
    gallery: Gallery,
}

#[derive(Default)]
struct Gallery {
    gallery: Vec<Vec<(f32, f32)>>,
}

impl State {
    pub fn update_transform(&self, queue: &Queue, buffer: &Buffer, ratio: f32) {
        let transform = Mat4::from_scale(vec3(0.5 / ratio, 0.5, 1.0));
        let x_offset = 0.5 - 0.5 / ratio;
        let transform = Mat4::from_translation(vec3(0.5 + x_offset, 0.5, 0.0)) * transform;
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&transform.to_cols_array_2d()));
    }
    pub fn update_color(&self, queue: &Queue, buffer: &Buffer) {
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[252f32 / 256.0, 222.0 / 256.0, 144.0 / 256.0, 1.0]));
    }
    pub fn draw<'a, 'b>(&'a self, rpass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        rpass.set_vertex_buffer(0, self.vertices_buffer.slice(..));
        rpass.set_index_buffer(self.indices_buffer.slice(..), wgpu::IndexFormat::Uint16);

        rpass.draw_indexed(0 .. self.indices.len() as u32, 0, 0..1);
    }
    pub fn new(device: &Device) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let mut origin = 0;

        origin += draw_line((-0.99, -0.99), (0.99, -0.99), &mut vertices, &mut indices, 0.01, origin);
        origin += draw_line((0.99, -0.99), (0.99, 0.99), &mut vertices, &mut indices, 0.01, origin);
        origin += draw_line((0.99, 0.99), (-0.99, 0.99), &mut vertices, &mut indices, 0.01, origin);
        origin += draw_line((-0.99, 0.99), (-0.99, -0.99), &mut vertices, &mut indices, 0.01, origin);

        let vertices_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let indices_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        Self {
            vertices,
            vertices_buffer,
            indices,
            indices_buffer,
            gallery: Default::default(),
        }
    }
}
