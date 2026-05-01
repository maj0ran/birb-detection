use bird_display_lib::error::Result;
use bird_display_lib::math::{UVec, Vec2};
use bird_display_lib::page::Page;
use bird_display_lib::screen::Screen;
use bird_display_lib::util::FontSetting;
use bird_display_lib::widgets::image::Image;
use bird_display_lib::widgets::text::Text;
use bird_display_lib::widgets::widget::{HAlign, Position, Positioner, VAlign};
use std::fs::File;
use std::io::{self, BufRead, Read};

#[allow(unused)]
const FONT_MED: FontSetting = FontSetting {
    name: "LucidaTypewriterRegular.ttf",
    size: 96.0,
    saturation: 1.0,
};

#[allow(unused)]
const FONT_SMALLMED: FontSetting = FontSetting {
    name: "LucidaTypewriterRegular.ttf",
    size: 64.0,
    saturation: 1.0,
};

#[allow(unused)]
const FONT_SMALL: FontSetting = FontSetting {
    name: "LucidaTypewriterRegular.ttf",
    size: 48.0,
    saturation: 1.0,
};

const UPPER_BORDER: u16 = 150;

pub fn main() -> Result<()> {
    let screen = Screen::new()?;

    println!("Screen Dimension: {}x{}", screen.width, screen.height);

    let mut page = Page::new_with_screen_dim(&screen);

    let image = Image::new(
        "bird.jpg",
        1.0,
        Position::Absolute(UVec::new(screen.width - 600 - 20, UPPER_BORDER)),
    );

    let f = File::open("desc.txt").unwrap();
    let mut reader = io::BufReader::new(f);

    let mut latin_name = String::new();
    let _ = reader.read_line(&mut latin_name);

    let mut common_name = String::new();
    let _ = reader.read_line(&mut common_name);

    let mut desc = String::new();
    let _ = reader.read_to_string(&mut desc);

    // strip trailing '\n'
    latin_name = latin_name[..latin_name.len() - 1].to_owned();
    common_name = common_name[..common_name.len() - 1].to_owned();
    desc = desc[..desc.len() - 1].to_owned();

    let latin_name_item = Text::new(
        &latin_name,
        FONT_SMALLMED,
        Position::Absolute(UVec::new(20, UPPER_BORDER)),
        Vec2::new(700, 200),
    );

    let common_name_item = Text::new(
        &(String::from("(") + &common_name + &String::from(")")),
        FONT_SMALLMED,
        Position::Relative(&Positioner {
            rel: latin_name_item.as_ref(),
            anchor: (HAlign::Left, VAlign::Bottom),
            align: (HAlign::Right, VAlign::Bottom),
            margin: (0, 48),
        }),
        Vec2::new(700, 200),
    );

    let desc_item = Text::new(
        &desc,
        FONT_SMALL,
        Position::Absolute(UVec::new(20, UPPER_BORDER + 600 + 48)),
        Vec2::new(1300, 900),
    );

    println!("text w: {} h: {}", desc_item.width, desc_item.height);

    page.add(image);
    page.add(latin_name_item);
    page.add(common_name_item);
    page.add(desc_item);

    screen.add_page(page);

    screen.clear();
    screen.render();
    screen.update();

    Ok(())
}
