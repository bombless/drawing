use app_surface::AppSurface;
use glam::vec4;
use wgpu::{Queue, SurfaceConfiguration};
use winit::dpi::PhysicalPosition;
use crate::texture::Texture;

mod base_shape;
mod zoom;

pub struct State {
    render_pipeline: wgpu::RenderPipeline,
    basic_shape: base_shape::State,
    zoom: zoom::State,
}

impl State {
    pub fn draw<'a, 'b>(&'a mut self, rpass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.zoom.bind_group(), &[]);

        self.basic_shape.draw(rpass);
    }
    pub fn update(&mut self, queue: &Queue) -> Vec<String> {
        self.zoom.update_proj();

        let mut ret = Vec::new();

        let f = self.zoom.get_mat4();
        for v in base_shape::VERTICES {
            let pos = vec4(v.position[0], v.position[1], v.position[2], 1.0);
            ret.push(format!("{:?} {:?}", v.color, f * pos));
        }

        queue.write_buffer(self.zoom.buffer(), 0, self.zoom.data());

        ret
    }
    pub fn translation(&mut self, config: &SurfaceConfiguration, p1: PhysicalPosition<f64>, p2: PhysicalPosition<f64>) {
        let offset_x = (p2.x - p1.x) as f32 / config.height as f32 * 2.0;
        let offset_y = -(p2.y - p1.y) as f32 / config.height as f32 * 2.0;
        self.zoom.translation(offset_x, offset_y);
    }
    pub fn move_cam(&mut self, config: &SurfaceConfiguration, p1: PhysicalPosition<f64>, p2: PhysicalPosition<f64>) {
        let offset_x = (p2.x - p1.x) as f32 / config.height as f32 * 2.0;
        let offset_y = -(p2.y - p1.y) as f32 / config.height as f32 * 2.0;
        self.zoom.move_cam(offset_x, offset_y);
    }
    pub fn resize_view(&mut self, config: &SurfaceConfiguration) {
        self.zoom.scale_x(config.width as f32 / config.height as f32);
    }
    pub fn new(app: &AppSurface) -> Self {let zoom = zoom::State::new(&app.device);
        let basic_shape = base_shape::State::new(&app.device);

        let render_basic_shape_pipeline_layout =
            app.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        zoom.layout()
                    ],
                    push_constant_ranges: &[],
                });

        let shader_basic_shape = app
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let render_basic_shape_pipeline = app
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_basic_shape_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_basic_shape,
                    entry_point: "vs_main",
                    buffers: &[base_shape::Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_basic_shape,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: app.config.format.add_srgb_suffix(),
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::default(),
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less, // 1.
                    stencil: wgpu::StencilState::default(), // 2.
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
            });
        Self { render_pipeline: render_basic_shape_pipeline, basic_shape, zoom: zoom::State::new(&app.device) }
    }
}