use std::iter;

use app_surface::{AppSurface, SurfaceFrame};
use utils::framework::{Action, run};
use winit::{dpi::{PhysicalPosition, PhysicalSize}, window::WindowId};
use winit::event::{ElementState, MouseButton, WindowEvent};

mod base_shape;
mod ui;




struct State {
    app: AppSurface,
    ui: ui::State,
    base_shape: base_shape::State,
    track_cursor: PhysicalPosition<f64>,
    old_pos: PhysicalPosition<f64>,
}

impl Action for State {
    fn new(app: AppSurface) -> Self {

        let ui = ui::State::new(&app);
        let base_shape = base_shape::State::new(&app);

        Self {
            app,
            ui,
            base_shape,
            track_cursor: PhysicalPosition::default(),
            old_pos: PhysicalPosition::default(),
        }
    }
    fn get_adapter_info(&self) -> wgpu::AdapterInfo {
        self.app.adapter.get_info()
    }

    fn current_window_id(&self) -> WindowId {
        self.app.view.id()
    }
    fn resize(&mut self, size: &PhysicalSize<u32>) {
        if self.app.config.width == size.width && self.app.config.height == size.height {
            return;
        }
        self.app.resize_surface();
        self.ui.resize_view(&self.app);
    }
    fn request_redraw(&mut self) {
        self.app.view.request_redraw();
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        if let WindowEvent::ReceivedCharacter('z') = event {
            self.ui.new_path(false);
        }
        if let WindowEvent::ReceivedCharacter('f') = event {
            self.ui.new_path(true);
        }
        if let WindowEvent::ReceivedCharacter('d') = event {
            self.ui.delete_path();
        }
        if let WindowEvent::CursorMoved { position: p, ..} = event {
            self.ui.update_cursor(&self.app.config, p.x as _, p.y as _);
            self.track_cursor = *p;
        }
        if let WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, ..} = event {
            self.old_pos = self.track_cursor;
        }
        if let WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, ..} = event {
            if self.old_pos == self.track_cursor {
                self.ui.push_point();
            } else {
                self.base_shape.change_zoom(&self.app.config, self.old_pos, self.track_cursor);
            }
        }
        false
    }

    fn update(&mut self) {
        self.base_shape.update(&self.app.queue);
        self.ui.update(&self.app);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let (output, view) = self.app.get_current_frame_view(None);


        let mut encoder = self
            .app
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.base_shape.draw(&mut render_pass);

            self.ui.draw(&mut render_pass);
        }

        self.app.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    run::<State>(None, None);
}
