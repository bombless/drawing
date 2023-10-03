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
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    color: color::State,
    text: text::State,
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];

impl State {
    pub fn draw<'a, 'b>(&'a self, rpass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        rpass.set_vertex_buffer(0, self.vertex_buffer());
        rpass.set_index_buffer(self.index_buffer(), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..self.num_indices(), 0, 0..1);
        self.text.draw(rpass);
    }
    pub fn vertex_buffer(&self) -> wgpu::BufferSlice {
        self.vertex_buffer.slice(..)
    }
    pub fn index_buffer(&self) -> wgpu::BufferSlice {
        self.index_buffer.slice(..)
    }
    pub fn num_indices(&self) -> u32 {
        self.num_indices
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

        let vertex_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });
        let num_indices = INDICES.len() as u32;

        Self {
            vertex_buffer, index_buffer, num_indices, text, color: color::State::new(device),
        }
    }
}

