use std::iter;

use app_surface::{AppSurface, SurfaceFrame};
// use PMXUtil::reader::ModelInfoStage;
use utils::framework::{Action, run};
use winit::{dpi::{PhysicalPosition, PhysicalSize}, window::WindowId};
use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};

mod base_shape;
mod ui;
pub(crate) mod uniform;



struct State {
    app: AppSurface,
    ui: ui::State,
    base_shape: base_shape::State,
    track_cursor: PhysicalPosition<f64>,
    old_pos: PhysicalPosition<f64>,
    last_track: PhysicalPosition<f64>,
    pressed: bool,
    ctrl: bool,
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
            last_track: PhysicalPosition::default(),
            pressed: false,
            ctrl: false,
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
        self.ui.resize_view(&self.app);
        self.base_shape.resize_view(&self.app.config);
        self.app.resize_surface();
    }
    fn request_redraw(&mut self) {
        self.app.view.request_redraw();
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        if let WindowEvent::ReceivedCharacter('z') = event {
            self.ui.new_path(false);
            return true;
        }
        if let WindowEvent::ReceivedCharacter('f') = event {
            self.ui.new_path(true);
            return true;
        }
        if let WindowEvent::ReceivedCharacter('d') = event {
            self.ui.delete_path();
            return true;
        }
        if let WindowEvent::CursorMoved { position: p, ..} = event {
            self.ui.update_cursor(&self.app.config, p.x as _, p.y as _);
            self.track_cursor = *p;
            if self.pressed {
                if self.ctrl {
                    self.base_shape.move_cam(&self.app.config, self.last_track, self.track_cursor);
                } else {
                    self.base_shape.translation(&self.app.config, self.last_track, self.track_cursor);
                }
            }
            self.last_track = *p;
            return true;
        }
        if let WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, ..} = event {
            self.old_pos = self.track_cursor;
            self.pressed = true;
            return true;
        }
        if let WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, ..} = event {
            if self.old_pos == self.track_cursor {
                self.ui.push_point();
            }
            self.pressed = false;
            return true;
        }
        if let WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode, state, ..}, ..} = event {
            if virtual_keycode == &Some(VirtualKeyCode::LControl) {
                self.ctrl = state == &ElementState::Pressed;
                return true;
            }
        }
        false
    }

    fn update(&mut self) {
        let info = self.base_shape.update(&self.app.queue);
        self.ui.update(&self.app, info);
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
    // let model_info_loader=PMXUtil::reader::ModelInfoStage::open("./琳妮特/【琳妮特】.pmx");
    // let (model_info, vertices_loader)=model_info_loader.unwrap().read();
    // println!("model_info {model_info:?}");
    // let (v, f) = vertices_loader.read();
    // println!("v {v:?}");
    // let (f, t) = f.read();
    // println!("face {f:?}");
    // let (v, m) = t.read();
    // println!("v {v:?}");
    // let (m, b) = m.read();
    // println!("m {m:?}");
    // let (b, pose) = b.read();
    // println!("b {b:?}");
    // let (pose, frame) = pose.read();
    // println!("pose {pose:?}");
    // let (frame, r) = frame.read();
    // println!("frame {frame:?}");
    // let (r, j) = r.read();
    // println!("r {r:?}");
    // let (j, s) = j.read();
    // println!("j {j:?}");
    // let s = if let Some(x) = s { x } else { return };
    // let s = s.read();
    // println!("s {s:?}");

}
