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
    cursor: (f32, f32),
    points: Vec<Vec<(f32, f32)>>,
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
        if self.buffer.size() > 0 {
            rpass.draw_indexed(0..self.num_indices_cursor(), 0, 0..1);
            let from = self.num_indices_cursor();
            let to = self.num_indices_paths() + self.num_indices_cursor();
            rpass.draw_indexed(from ..to, 0, 0..1);
        }
        self.text.draw(rpass);
    }
    pub fn vertices(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }
    pub fn indices(&self) -> &[u8] {
        bytemuck::cast_slice(&self.indices)
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn check_buffer(&mut self, device: &Device) {
        if self.buffer.size() == self.vertices().len() as _ {
            return
        }

        let cursor_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: self.vertices(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        self.buffer = cursor_buffer;
    }
    pub fn check_index_buffer(&mut self, device: &Device) {
        if self.index_buffer.size() == self.indices().len() as _ {
            return
        }

        let index_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: self.indices(),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });
        self.index_buffer = index_buffer;
    }
    pub fn num_indices_cursor(&self) -> u32 {
        self.count_segments as u32 * 3
    }
    pub fn num_indices_paths(&self) -> u32 {
        self.indices.len() as u32 - self.num_indices_cursor()
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
        self.points.last_mut().unwrap().push(self.cursor);
    }
    pub fn new_path(&mut self) {
        if self.points.last().unwrap().is_empty() {
            return;
        }
        self.points.push(Vec::new());
    }
    pub fn delete_path(&mut self) {
        std::mem::take(self.points.last_mut().unwrap());
    }
    pub fn update_cursor(&mut self, x: f32, y: f32)  {
        let mut vertices = vec![];

        let mut indices = vec![];

        let count_segments = self.count_segments as _;
        let radius = self.radius;

        fn fill_buffer(vertices: &mut Vec<f32>, indices: &mut Vec<u16>, origin: u16,
                       factor: f32, radius: f32, x: f32, y: f32, count_segments: u32)
        {
            vertices.push(x);
            vertices.push(y);
            for i in 0 .. count_segments {
                let p1 = i as f32 * factor;
                indices.push(origin);
                indices.push(origin + i as u16 * 2 + 1);
                indices.push(origin + i as u16 * 2 + 2);
                let offset_x1 = p1.sin() * radius;
                let offset_y1 = p1.cos() * radius;
                vertices.push(offset_x1 + x);
                vertices.push(offset_y1 + y);
                let p2 = factor * (i + 1) as f32;
                let offset_x2 = p2.sin() * radius;
                let offset_y2 = p2.cos() * radius;
                vertices.push(offset_x2 + x);
                vertices.push(offset_y2 + y);
            }
        }

        let mut count = 0;
        fill_buffer(&mut vertices, &mut indices, count,
                    2.0 * std::f32::consts::PI / count_segments as f32, radius, x, y, count_segments);


        for segment in &self.points {
            for &(x, y) in segment {
                count += count_segments as u16 * 2 + 1;
                fill_buffer(&mut vertices, &mut indices, count,
                            2.0 * std::f32::consts::PI / count_segments as f32, radius, x, y, count_segments);
            }
        }

        if indices.len() % 2 == 1 {
            indices.push(0);
        }

        self.cursor = (x, y);
        self.vertices = vertices;
        self.indices = indices;
    }
    pub fn new(app: &AppSurface) -> Self {

        let text = text::State::new(&app);

        let device = &app.device;

        let cursor = (0.0, 0.0);

        let radius = 0.1f32;

        let count_segments = 30;


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
            points: vec![vec![]],
            vertices: Vec::new(),
            indices: Vec::new(),
            radius,
            count_segments,
            buffer: cursor_buffer,
            index_buffer: cursor_index_buffer, text, color,
        }
    }
}

