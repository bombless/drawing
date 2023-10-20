use wgpu::util::DeviceExt;
use rand::random;


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
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
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct State {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

macro_rules! vertices {
    ($($x:expr, $y:expr),+) => {
        [$(
            Vertex { position: [$x, $y, -0.25], color: [($x + $y) * ($x + $y), ($x + $y) * ($x + $y), ($x + $y) * ($x + $y)] },
            Vertex { position: [$x, $y, 0.25], color: [($x + $y) * ($x + $y), ($x + $y) * ($x + $y), ($x + $y) * ($x + $y)] },
        )+]
    };
}

pub const VERTICES: &[Vertex] = &vertices![
        -0.5, 0.75,
        -0.25, 0.75,
        0.5, 0.75,
        0.5, 0.5,
        0.5, 0.25,
        0.5, 0.0,
        -0.25, 0.5,
        -0.25, 0.25,
        -0.25, 0.0,
        -0.5, -0.5,
        -0.25, -0.5
];


macro_rules! indices {
    ($($a:tt $b:tt $c:tt),+) => {
        [$($a, $b, $c),+]
    }
}

const INDICES: &[u16] = &indices![2 4 22, 2 22 20, 2 20 19, 2 19 1, 3 1 19, 3 19 21, 20 22 21, 20 21 19, 13 15 16, 13 16 14, 17 21 22, 17 22 18, 1 5 6, 1 6 2, 4 6 8, 4 8 14, 14 8 7, 14 7 13, 7 5 3, 7 3 13, 5 7 8, 5 8 6, 15 9 10, 15 10 16, 16 10 12, 16 12 18, 18 12 11, 18 11 17, 11 9 15, 11 15 17, 10 9 11, 10 11 12];




impl State {
    pub fn draw<'a, 'b>(&'a self, rpass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        rpass.set_vertex_buffer(0, self.vertex_buffer());
        rpass.set_index_buffer(self.index_buffer(), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..self.num_indices(), 0, 0..1);
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
    pub fn new(device: &wgpu::Device) -> Self {


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
            vertex_buffer, index_buffer, num_indices
        }
    }
}

