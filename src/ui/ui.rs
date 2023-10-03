use app_surface::AppSurface;
use wgpu::Device;
use wgpu::util::DeviceExt;

use super::color;
use super::text;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct State {
    cursor: [f32; 2],
    points: Vec<[f32; 2]>,
    vertices: Vec<f32>,
    indices: Vec<u16>,
    radius: f32,
    count_segments: usize,
    buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    color: color::State,
    text: text::State,
}

impl State {
    pub fn draw<'a, 'b>(&'a self, rpass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now();
        let timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let bind_group = if timestamp % 2 == 0 {
            self.color().red_bind_group()
        } else {
            self.color().green_bind_group()
        };
        rpass.set_bind_group(0, bind_group, &[]);
        rpass.set_vertex_buffer(0, self.buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..self.num_indices(), 0, 0..1);
        self.text.draw(rpass);
    }
    pub fn cursor_vertices(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }
    pub fn cursor_indices(&self) -> &[u8] {
        bytemuck::cast_slice(&self.indices)
    }

    pub fn cursor_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn cursor_index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn check_cursor_buffer(&mut self, device: &Device) {
        if self.buffer.size() == self.cursor_vertices().len() as _ {
            return
        }

        let cursor_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: self.cursor_vertices(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        self.buffer = cursor_buffer;
    }
    pub fn check_cursor_index_buffer(&mut self, device: &Device) {
        if self.index_buffer.size() == self.cursor_indices().len() as _ {
            return
        }

        let cursor_index_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: self.cursor_indices(),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });
        self.index_buffer = cursor_index_buffer;
    }
    pub fn write_vertices_buffer(&mut self, app: &AppSurface) {
        self.check_cursor_buffer(&app.device);
        let cursor_vertices = self.cursor_vertices();
    }
    pub fn num_indices(&self) -> u32 {
        if self.indices.is_empty() { 0 } else { self.count_segments as u32 * 3 }
    }
    pub fn color(&self) -> &color::State {
        &self.color
    }
    pub fn text(&self) -> &text::State {
        &self.text
    }
    pub fn text_mut(&mut self) -> &mut text::State {
        &mut self.text
    }
    pub fn push_point(&mut self) {
        self.points.push(self.cursor);
        println!("{:?}", self.points);
    }
    pub fn update_cursor(&mut self, cursor: [f32; 2])  {
        let mut cursor_vertices = Vec::from(cursor);

        let mut cursor_indices = vec![0];

        let count_segments = self.count_segments;
        let radius = self.radius;

        for i in 0 .. count_segments {
            let p1 = i as f32 / count_segments as f32 * 2.0 * std::f32::consts::PI;
            cursor_indices.push(0);
            cursor_indices.push(i as u16 * 2);
            cursor_indices.push(i as u16 * 2 + 1);
            let offset_x1 = p1.sin() * radius;
            let offset_y1 = p1.cos() * radius;
            cursor_vertices.push(offset_x1 + cursor[0]);
            cursor_vertices.push(offset_y1 + cursor[1]);
            let p2 = i as f32 + 1.0 / count_segments as f32 * 2.0 * std::f32::consts::PI;
            let offset_x2 = p2.sin() * radius;
            let offset_y2 = p2.cos() * radius;
            cursor_vertices.push(offset_x2 + cursor[0]);
            cursor_vertices.push(offset_y2 + cursor[1]);
        }

        if cursor_indices.len() % 2 == 1 {
            cursor_indices.push(0);
        }

        self.cursor = cursor;
        self.vertices = cursor_vertices;
        self.indices = cursor_indices;
    }
    pub fn new(app: &AppSurface) -> Self {

        let text = text::State::new(&app);

        let device = &app.device;

        let cursor = [0.0f32, 0.0];

        let radius = 0.1f32;

        let count_segments = 900;



        let cursor_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::VERTEX,
            });
        let cursor_index_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::INDEX,
            });

        let color = color::State::new(device);

        Self {
            cursor,
            points: Vec::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            radius,
            count_segments,
            buffer: cursor_buffer,
            index_buffer: cursor_index_buffer, text, color,
        }
    }
}

