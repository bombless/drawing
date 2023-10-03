use app_surface::AppSurface;
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
    cursor_vertices: Vec<f32>,
    cursor_indices: Vec<u16>,
    radius: f32,
    count_segments: usize,
    cursor_buffer: wgpu::Buffer,
    cursor_index_buffer: wgpu::Buffer,
    color: color::State,
    text: text::State,
}

impl State {
    pub fn draw<'a, 'b>(&'a self, rpass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        rpass.set_vertex_buffer(0, self.vertex_buffer());
        rpass.set_index_buffer(self.index_buffer(), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..self.num_indices(), 0, 0..1);
        self.text.draw(rpass);
    }
    pub fn vertex_buffer(&self) -> wgpu::BufferSlice {
        self.cursor_buffer.slice(..)
    }
    pub fn index_buffer(&self) -> wgpu::BufferSlice {
        self.cursor_index_buffer.slice(..)
    }
    pub fn num_indices(&self) -> u32 {
        self.count_segments as u32 * 3
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
    pub fn new(app: &AppSurface) -> Self {

        let text = text::State::new(&app);

        let device = &app.device;

        let cursor = [0.0f32, 0.0];

        let radius = 0.1f32;

        let count_segments = 900;

        let mut cursor_vertices = Vec::from(cursor);

        let mut cursor_indices = vec![0];

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

        let cursor_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&cursor_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let cursor_index_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&cursor_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let color = color::State::new(device);

        Self {
            cursor,
            cursor_vertices,
            cursor_indices,
            radius,
            count_segments,
            cursor_buffer, cursor_index_buffer, text, color,
        }
    }
}

