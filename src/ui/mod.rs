use app_surface::AppSurface;
use glam::Mat4;
use wgpu::{ShaderStages, SurfaceConfiguration};
use crate::uniform::Proxy as Uniform;


mod ui;
pub(super) mod text;
#[allow(unused)]
mod dialog;
mod utils;

pub struct State {
    render_pipeline: wgpu::RenderPipeline,
    ui: ui::State,
    transform: Uniform,
    color: Uniform,
    dialog: dialog::State,
    ratio: f32,
}

impl State {
    pub fn resize_view(&mut self, app: &AppSurface) {
        self.ui.text().resize_view(app);
        self.ratio = app.config.width as f32 / app.config.height as f32;
    }
    pub fn update(&mut self, app: &AppSurface) {
        self.ui.check_buffer(&app.device);
        app.queue.write_buffer(self.ui.buffer(), 0, self.ui.vertices());
        self.ui.check_index_buffer(&app.device);
        app.queue.write_buffer(self.ui.index_buffer(), 0, self.ui.indices());
        self.ui.update_color(self.color.buffers(0), &app.queue);
        self.ui.update_transform(self.transform.buffer(0, 0), &app.queue, self.ratio);
        self.ui.text_mut().process_queued(app);
        self.dialog.update_transform(&app.queue, self.transform.buffer(1, 0), self.ratio);
        self.dialog.update_color(&app.queue, self.color.buffer(1, 0));
    }
    pub fn update_cursor(&mut self, config: &SurfaceConfiguration, x: f32, y: f32) {
        let arg_x = (x / config.width as f32 * 2.0 - 1.0) * self.ratio;
        let arg_y = 1.0 - y / config.height as f32 * 2.0;
        self.ui.update_cursor(arg_x, arg_y, self.ratio);
    }
    pub fn new_path(&mut self, fill: bool) {
        self.ui.new_path(fill);
        self.ui.update_points();
    }
    pub fn delete_path(&mut self) {
        self.ui.delete_path();
    }
    pub fn push_point(&mut self) {
        self.ui.push_point();
    }
    pub fn draw<'a, 'b>(&'a mut self, rpass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        rpass.set_pipeline(&self.render_pipeline);

        rpass.set_bind_group(0, self.transform.bind_group(1, 0), &[]);
        rpass.set_bind_group(1, self.color.bind_group(1, 0), &[]);
        self.dialog.draw(rpass);

        rpass.set_bind_group(0, self.transform.bind_group(0, 0), &[]);
        self.ui.draw(rpass, self.color.buffers(0));
    }
    pub fn new(app: &AppSurface) -> Self {

        let shader_ui = app
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });


        let ratio = 16.0 / 9.0;
        let ui = ui::State::new(&app);
        let transform = Uniform::new(bytemuck::cast_slice(&Mat4::IDENTITY.to_cols_array_2d()), &[1, 1],  &app.device, ShaderStages::VERTEX);
        let color = Uniform::new(bytemuck::cast_slice(&[1.0f32; 4]), &[2, 1], &app.device, ShaderStages::FRAGMENT);
        let dialog = dialog::State::new(&app.device);

        let render_ui_pipeline_layout =
            app.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        transform.layout(), color.layout()
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline = app
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
                    topology: wgpu::PrimitiveTopology::default(),
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
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

        Self { render_pipeline, ui, transform, color, dialog, ratio }
    }
}