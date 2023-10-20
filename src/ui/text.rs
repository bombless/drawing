use wgpu_text::{glyph_brush::{Section as TextSection, Text}, BrushBuilder, TextBrush};
use glyph_brush::ab_glyph::FontRef;
use glyph_brush::Layout;

pub struct State {
    brush: TextBrush<FontRef<'static>>,
    section: TextSection<'static>,
}

impl State {
    pub fn new(app: &app_surface::AppSurface) -> Self {
        let text = "Press Z to start path, F to fill and start path, D to delete path";

        Self {
            brush: BrushBuilder::using_font_bytes(include_bytes!("C:\\Windows\\Fonts\\msyh.ttc")).unwrap()
                .build(&app.device, app.config.width, app.config.height, app.config.format),
            section: TextSection::default().add_text(Text::new(text)),
        }
    }

    pub fn process_queued(&mut self, app: &app_surface::AppSurface, info: &[&str]) {
        let mut sections = vec![&self.section];
        let mut offset = 0.0;
        let info = info.iter().map(|x| {
            offset += 20.0;
            TextSection::default()
                .add_text(Text::new(*x))
                .with_screen_position((0.0, offset))
        }).collect::<Vec<_>>();
        sections.extend(&info);
        self.brush.queue(&app.device, &app.queue, sections).unwrap();
    }

    pub fn draw<'a, 'b>(&'a self, rpass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        self.brush.draw(rpass);
    }

    pub fn resize_view(&self, app: &app_surface::AppSurface) {
        self.brush.resize_view(app.config.width as f32, app.config.height as f32, &app.queue);
    }
}