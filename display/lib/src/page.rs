use crate::screen::Screen;
use crate::util::Color;
use crate::widgets::widget::Widget;

///
/// A single page that can be rendered on the screen. A page is a collection of widgets.
///
pub struct Page {
    pub width: u16,
    pub height: u16,
    pub widgets: Vec<Box<dyn Widget>>,
    buffer: Vec<Color>,
}

impl Page {
    ///
    /// get a new Page that has the dimensions of the screen object.
    ///
    pub fn new_with_screen_dim(screen: &Screen) -> Page {
        let width = screen.width;
        let height = screen.height;
        let buffer = vec![Color::new(255, 255, 255); width as usize * height as usize];

        Page {
            width,
            height,
            widgets: Vec::new(),
            buffer,
        }
    }
    pub fn add(&mut self, obj: Box<dyn Widget>) {
        self.widgets.push(obj);
    }
    ///
    /// take all widgets and composite them into a single buffer with the size of a page
    ///
    pub fn composite(&mut self) -> &Vec<Color> {
        let buf = &mut self.buffer;
        for widget in &mut self.widgets {
            let pixels = widget.make();
            let pos = widget.get_pos();

            // scan each line of the widget
            for line in 0..widget.get_height() as usize {
                // using Vec.splice(), we move each pixel line of the widget
                // to the page-buffer with respect to its absolute position on the page
                let begin = (pos.y as usize + line) as usize * self.width as usize + pos.x as usize;
                let end = begin + widget.get_width() as usize;
                let it = &pixels
                    [line * widget.get_width() as usize..(line + 1) * widget.get_width() as usize];
                buf.splice(begin..end, it.iter().cloned());
            }
        }
        buf
    }
}
