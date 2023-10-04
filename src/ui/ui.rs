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
    segments_count: usize,
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
        rpass.set_vertex_buffer(0, self.buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        let index_buffer_len = self.index_buffer.size() as u32 / 2;
        let indices_len = self.indices.len() as u32;

        if index_buffer_len > 0 {
            let count_cursor = self.segments_count as u32 * 3;

            rpass.set_bind_group(0, self.color().red_bind_group(), &[]);

            rpass.draw_indexed(0..count_cursor, 0, 0..1);

            rpass.set_bind_group(0, self.color().green_bind_group(), &[]);

            rpass.draw_indexed(count_cursor..indices_len.min(index_buffer_len), 0, 0..1);
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
        if self.buffer.size() >= self.vertices().len() as u64 * 4 {
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
        if self.index_buffer.size() >= self.indices().len() as u64 * 2 {
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
        self.update_points();
    }
    pub fn new_path(&mut self) {
        if self.points.last().unwrap().is_empty() {
            return;
        }
        self.points.push(Vec::new());
    }
    pub fn delete_path(&mut self) {
        if self.points.len() == 1 {
            self.points[0].truncate(0);
        } else {
            self.points.pop();
        }
        self.update_points();
    }

    fn fill_buffer_with_a_point(vertices: &mut Vec<f32>, indices: &mut Vec<u16>, origin: u16,
                                factor: f32, radius: f32, x: f32, y: f32, count_segments: u32)
    {
        vertices.push(x);
        vertices.push(y);
        let offset_x = 0.0f32.sin() * radius;
        let offset_y = 0.0f32.cos() * radius;
        vertices.push(offset_x + x);
        vertices.push(offset_y + y);
        for i in 1 .. count_segments {
            let p = i as f32 * factor;
            indices.push(origin);
            indices.push(origin + i as u16);
            indices.push(origin + i as u16 + 1);
            let offset_x = p.sin() * radius;
            let offset_y = p.cos() * radius;
            vertices.push(offset_x + x);
            vertices.push(offset_y + y);
        }
        indices.push(origin);
        indices.push(origin + count_segments as u16);
        indices.push(origin + 1);
    }

    pub fn update_cursor(&mut self, x: f32, y: f32)  {
        self.cursor = (x, y);

        let count_segments = self.segments_count as _;
        let radius = self.radius;

        if self.indices.is_empty() {
            Self::fill_buffer_with_a_point(&mut self.vertices, &mut self.indices, 0,
                                           2.0 * std::f32::consts::PI / count_segments as f32, radius, x, y, count_segments);

        } else {

            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            Self::fill_buffer_with_a_point(&mut vertices, &mut indices, 0,
                                           2.0 * std::f32::consts::PI / count_segments as f32, radius, x, y, count_segments);
            self.vertices[..vertices.len()].copy_from_slice(&vertices);
            self.indices[..indices.len()].copy_from_slice(&indices);
        }

        if self.indices.len() % 2 == 1 {
            self.indices.push(0);
        }
    }

    pub fn update_points(&mut self)  {
        if self.indices.is_empty() {
            return;
        }

        let segments_count = self.segments_count as _;
        let radius = self.radius;

        let mut count = segments_count as u16 + 1;

        self.vertices.truncate(self.segments_count * 2 + 2);
        self.indices.truncate(self.segments_count * 3);

        for segment in &self.points {
            for &(x, y) in segment {
                Self::fill_buffer_with_a_point(&mut self.vertices, &mut self.indices, count,
                                               2.0 * std::f32::consts::PI / segments_count as f32, radius, x, y, segments_count);
                count += segments_count as u16 + 1;
            }
        }

        fn get_radian(p1: (f32, f32), p2: (f32, f32)) -> f32 {
            ((p2.0 - p1.0) / (p2.1 - p1.1)).atan()
        }

        for segment in &self.points {
            for i in 1 .. segment.len() {
                let pre = segment[i - 1];
                let cur = segment[i];

                let radian = get_radian(pre, cur);
                let offset_x1 = (radian + std::f32::consts::PI / 2.0).sin() * self.radius;
                let offset_x2 = (radian - std::f32::consts::PI / 2.0).sin() * self.radius;
                let offset_y1 = (radian + std::f32::consts::PI / 2.0).cos() * self.radius;
                let offset_y2 = (radian - std::f32::consts::PI / 2.0).cos() * self.radius;

                self.vertices.push(offset_x1 + pre.0);
                self.vertices.push(offset_y1 + pre.1);
                self.vertices.push(offset_x2 + pre.0);
                self.vertices.push(offset_y2 + pre.1);
                self.vertices.push(offset_x1 + cur.0);
                self.vertices.push(offset_y1 + cur.1);
                self.vertices.push(offset_x2 + cur.0);
                self.vertices.push(offset_y2 + cur.1);

                self.indices.push(count);
                self.indices.push(count + 1);
                self.indices.push(count + 2);

                self.indices.push(count + 1);
                self.indices.push(count + 2);
                self.indices.push(count + 3);

                count += 4;
            }
        }

        if self.indices.len() % 2 == 1 {
            self.indices.push(0);
        }
    }
    pub fn new(app: &AppSurface) -> Self {

        let text = text::State::new(&app);

        let device = &app.device;

        let cursor = (0.0, 0.0);

        let radius = 0.01f32;

        let segments_count = 6;


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
            segments_count,
            buffer: cursor_buffer,
            index_buffer: cursor_index_buffer, text, color,
        }
    }
}

