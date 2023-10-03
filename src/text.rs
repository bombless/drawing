use wgpu_text::{glyph_brush::{Section as TextSection, Text}, BrushBuilder, TextBrush};
use glyph_brush::ab_glyph::FontRef;

pub struct State<'a> {
    brush: TextBrush<FontRef<'a>>,
    section: TextSection<'a>,
}

impl<'a> State<'a> {
    pub fn new(app: &app_surface::AppSurface) -> Self {

        Self {
            brush: BrushBuilder::using_font_bytes(include_bytes!("C:\\Windows\\Fonts\\msyh.ttc")).unwrap()
                .build(&app.device, app.config.width, app.config.height, app.config.format),
            section: TextSection::default().add_text(Text::new("Press Z to start path")),
        }
    }

    pub fn process_queued(&mut self, app: &app_surface::AppSurface) {
        self.brush.queue(&app.device, &app.queue, vec![&self.section]).unwrap();
    }

    pub fn draw<'b>(&'a self, rpass: &mut wgpu::RenderPass<'b>) where 'a: 'b {
        self.brush.draw(rpass);
    }

    pub fn resize_view(&self, app: &app_surface::AppSurface) {
        self.brush.resize_view(app.config.width as f32, app.config.height as f32, &app.queue);
    }
}