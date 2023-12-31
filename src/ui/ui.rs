use std::ops::Index;
use app_surface::AppSurface;
use glam::{Mat4, vec3};
use wgpu::{Buffer, Device, Queue};
use wgpu::util::DeviceExt;
use crate::uniform::BufferAndBindGroupBindingVec;

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

struct Shape {
    shape: Vec<(f32, f32)>,
    fill: bool,
}

impl Shape {
    fn push(&mut self, p: (f32, f32)) {
        self.shape.push(p)
    }
    fn is_empty(&self) -> bool {
        self.shape.is_empty()
    }
    fn len(&self) -> usize {
        self.shape.len()
    }
    fn last(&self) -> Option<&(f32, f32)> {
        self.shape.last()
    }
    fn truncate(&mut self, offset: usize) {
        self.shape.truncate(offset)
    }
}

impl<'a> IntoIterator for &'a Shape {
    type Item = &'a (f32, f32);
    type IntoIter = std::slice::Iter<'a, (f32, f32)>;

    fn into_iter(self) -> Self::IntoIter {
        self.shape[..].into_iter()
    }
}

impl Index<usize> for Shape {
    type Output = (f32, f32);

    fn index(&self, index: usize) -> &Self::Output {
        self.shape.index(index)
    }
}

pub struct State {
    cursor: Option<(f32, f32)>,
    points: Vec<Shape>,
    vertices: Vec<f32>,
    indices: Vec<u16>,
    radius: f32,
    segments_count: usize,
    buffer: Buffer,
    index_buffer: Buffer,
    text: text::State,
}

impl State {
    pub fn draw<'a, 'b>(&'a self, rpass: &mut wgpu::RenderPass<'b>, color_buffers: &'a BufferAndBindGroupBindingVec) where 'a: 'b {

        rpass.set_vertex_buffer(0, self.buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        let index_buffer_len = self.index_buffer.size() as u32 / 2;
        let indices_len = self.indices.len() as u32;

        if index_buffer_len > 0 {
            let count_cursor = self.segments_count as u32 * 3;

            if self.cursor.is_some() {
                rpass.set_bind_group(1, color_buffers.bind_group(0), &[]);

                rpass.draw_indexed(0..count_cursor, 0, 0..1);
            }

            rpass.set_bind_group(1, color_buffers.bind_group(1), &[]);


            rpass.draw_indexed(count_cursor..indices_len.min(index_buffer_len), 0, 0..1);
        }

        self.text.draw(rpass);
    }
    pub fn update_color(&self, buffers: &BufferAndBindGroupBindingVec, queue: &Queue) {
        queue.write_buffer(buffers.buffer(0), 0, bytemuck::cast_slice(&[1f32, 0.0, 1.0, 1.0]));
        queue.write_buffer(buffers.buffer(1), 0, bytemuck::cast_slice(&[0f32, 1.0, 0.0, 1.0]));
    }
    pub fn update_transform(&self, buffer: &Buffer, queue: &Queue, ratio: f32) {
        let transform = Mat4::from_scale(vec3(1.0 / ratio, 1.0, 1.0));
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&transform.to_cols_array_2d()));
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
    pub fn text(&self) -> &text::State {
        &self.text
    }
    pub fn text_mut(&mut self) -> &mut text::State {
        &mut self.text
    }
    pub fn push_point(&mut self) {
        if let Some(cursor) = self.cursor {
            self.points.last_mut().unwrap().push(cursor);
            self.update_points();
        }
    }
    pub fn new_path(&mut self, fill: bool) {
        if self.points.last().unwrap().is_empty() {
            return;
        }
        self.points.last_mut().unwrap().fill = fill;
        self.points.push(Shape {
            shape: Vec::new(),
            fill: false,
        });
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

    pub fn update_cursor(&mut self, x: f32, y: f32, ratio: f32)  {
        if x.abs() > 2.0 / ratio {
            self.cursor = None;
            return;
        }
        self.cursor = Some((x, y));

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
        use super::utils::draw_line;

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


        for (i, segment) in self.points.iter().enumerate() {
            if segment.is_empty() {
                break;
            }
            let full_shape = i + 1 < self.points.len();
            for i in 1 .. segment.len() {
                let p1 = segment[i - 1];
                let p2 = segment[i];
                draw_line(p1, p2, &mut self.vertices, &mut self.indices, self.radius, count);

                count += 4;
            }
            if segment.fill {
                for &(x, y) in segment {
                    self.vertices.push(x);
                    self.vertices.push(y);
                }
                let limit = segment.len();
                for i in 0 .. limit {
                    for j in i + 1 .. limit {
                        for k in j + 1 .. limit {
                            self.indices.push(count + i as u16);
                            self.indices.push(count + j as u16);
                            self.indices.push(count + k as u16);
                        }
                    }
                }
                count += segment.len() as u16;
            }
            if full_shape {
                let p1 = *segment.last().unwrap();
                let p2 = segment[0];
                draw_line(p1, p2, &mut self.vertices, &mut self.indices, self.radius, count);

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

        let cursor = Some((0.0, 0.0));

        let radius = 0.01f32;

        let segments_count = 6;


        let buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::INDEX,
            });

        Self {
            cursor,
            points: vec![Shape { shape: vec![], fill: false }],
            vertices: Vec::new(),
            indices: Vec::new(),
            radius,
            segments_count,
            buffer,
            index_buffer,
            text,
        }
    }
}

