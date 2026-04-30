/***
 * GUI-Element that shows a Text on the E-Ink Display
 ***/
use super::widget::{Position, Widget};
use crate::math::Vec2;
use crate::{
    math::UVec,
    util::{Color, FontSetting},
};
use rusttype::{Font, Point, PositionedGlyph, Scale};

pub struct Text<'a> {
    pub pos: UVec,
    pub width: u16,
    pub height: u16,
    pub font_info: FontSetting,
    glyphs: Vec<PositionedGlyph<'a>>,
}

impl<'a> Text<'a> {
    pub fn new(
        text: &str,
        font_info: FontSetting,
        pos: Position,
        size: Vec2<u16>,
    ) -> Box<Text<'a>> {
        let width = size.x;
        let height = size.y;

        let path = std::env::current_dir()
            .unwrap()
            .join("fonts/")
            .join(&font_info.name);
        let size = Scale::uniform(font_info.size);

        let font_type = std::fs::read(&path).unwrap();

        let font = Font::try_from_vec(font_type).unwrap();
        let v_metrics = font.v_metrics(size);

        let mut glyphs: Vec<_> = font
            .layout(text, size, rusttype::point(0.0, v_metrics.ascent))
            .collect();

        let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as i32;

        let mut line_wrap = 0.0;
        let mut line = 0;
        for glyph in &mut glyphs {
            let h_metrics = glyph.unpositioned().h_metrics();
            // Use the right side of the bounding box relative to its position to check if it fits.
            // Since we set position.x to line_wrap, the max.x will be line_wrap + glyph_bounding_box_relative_max_x
            let glyph_bb = glyph
                .unpositioned()
                .clone()
                .positioned(rusttype::point(0.0, 0.0))
                .pixel_bounding_box();
            let glyph_right_edge = if let Some(g_bb) = glyph_bb {
                g_bb.max.x as f32
            } else {
                h_metrics.advance_width
            };

            if line_wrap + glyph_right_edge > width as f32 && line_wrap > 0.0 {
                line_wrap = 0.0;
                line += 1;
            }

            let x = line_wrap;
            let y = (line + 1) as f32 * glyphs_height as f32;

            glyph.set_position(Point { x, y });

            line_wrap += h_metrics.advance_width;
        }

        glyphs.retain(|g| g.position().y < height as f32 - font_info.size);

        let w = Text {
            pos: UVec { x: 0, y: 0 },
            width,
            height,
            font_info,

            glyphs,
        };

        let w = match pos {
            Position::Absolute(p) => w.set_pos_abs(p),
            Position::Relative(p) => w.set_pos_rel(p),
        };

        Box::new(w)
    }
}

impl<'a> Widget for Text<'a> {
    widget!();

    fn make(&self) -> Vec<Color> {
        let mut pixels =
            vec![Color::new(255, 255, 255); self.width as usize * self.height as usize];

        for glyph in &self.glyphs {
            if let Some(glyph_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    let c = (255.0 - 255.0 * v * self.font_info.saturation) as u8;
                    let x = glyph_box.min.x + x as i32;
                    let y = glyph_box.min.y + y as i32;
                    pixels[(x + y * self.width as i32) as usize] = Color::new(c, c, c);
                });
            }
        }
        pixels
    }
}
