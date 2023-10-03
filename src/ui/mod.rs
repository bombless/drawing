use app_surface::AppSurface;

mod color;
mod ui;
pub(super) mod text;

pub struct State {
    render_pipeline: wgpu::RenderPipeline,
    ui: ui::State,
}

impl State {
    pub fn resize_view(&self, app: &AppSurface) {
        self.ui.text().resize_view(app);
    }
    pub fn update(&mut self, app: &AppSurface) {
        app.queue.write_buffer(self.ui.color().buffer(), 0, self.ui.color().data());
        self.ui.text_mut().process_queued(app);
    }
    pub fn draw<'a, 'b>(&'a mut self, rpass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.ui.color().bind_group(), &[]);

        self.ui.draw(rpass);
    }
    pub fn new(app: &AppSurface) -> Self {

        let shader_ui = app
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let ui = ui::State::new(&app);

        let render_ui_pipeline_layout =
            app.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        ui.color().layout()
                    ],
                    push_constant_ranges: &[],
                });

        let render_ui_pipeline = app
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render UI Pipeline"),
                layout: Some(&render_ui_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_ui,
                    entry_point: "vs_main",
                    buffers: &[ui::Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_ui,
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
                    topology: wgpu::PrimitiveTopology::TriangleList,
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
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
            });
        Self { render_pipeline: render_ui_pipeline, ui: ui::State::new(&app) }
    }
}