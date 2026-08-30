use crate::{
    math::{IVec, UVec},
    util::Color,
};

pub enum VAlign {
    Top,
    Center,
    Bottom,
}

pub enum HAlign {
    Left,
    Center,
    Right,
}

/// Positioner to determine the position of a widget relative to another widget.
/// This is only used by `set_pos_rel` in a widget.
pub struct Positioner<'a> {
    pub rel: &'a dyn Widget,
    pub anchor: (HAlign, VAlign),
    pub align: (HAlign, VAlign),
    pub margin: (i16, i16),
}

pub enum Position<'a> {
    Absolute(UVec),
    Relative(&'a Positioner<'a>),
}

///
/// Trait for GUI-Elements that can be drawn on the E-Ink Display.
///
pub trait Widget {
    /// Position the Widget by an absolute value.
    /// This is just a 2D Point on the screen space
    fn set_pos_abs(mut self, pos: UVec) -> Self
    where
        Self: Sized,
    {
        self.set_pos(pos);
        self
    }

    /// Position the Widget by a value relative to another Widget.
    /// This positioning is composited by three values:
    /// - An anchor of another Widget: One of the 4 corners or one of the 4 centers between
    ///   the corner.
    /// - An alignment of itself: Decides how this widget is aligned relative to the anchor. A left
    ///   alignment means the widget is moved so that it is left to the anchor.
    /// - And a margin: Adds an absolute vector on this position.
    fn set_pos_rel(mut self, pos: &Positioner) -> Self
    where
        Self: Sized,
    {
        // Position of the Widget we want to position relative to
        let position = IVec::from(
            UVec::from(pos.rel.get_pos()) +
        // Add the position of another anchor of this widget
        match pos.anchor.0 {
            HAlign::Left => UVec::new(0, 0),
            HAlign::Center => UVec::new(pos.rel.get_width()  / 2, 0),
            HAlign::Right => UVec::new(pos.rel.get_width() , 0),
        } + match pos.anchor.1 {
            VAlign::Top => UVec::new(0, 0),
            VAlign::Center => UVec::new(0, pos.rel.get_height()  / 2),
            VAlign::Bottom => UVec::new(0, pos.rel.get_height() ),
        } -
        // Substract the align of self from the position
        match pos.align.0 {
            HAlign::Left => UVec::new(self.get_width() , 0),
            HAlign::Center => UVec::new(self.get_width() / 2 , 0),
            HAlign::Right => UVec::new(0, 0),
        } - match pos.align.1 {
            VAlign::Top => UVec::new(0, self.get_height()),
            VAlign::Center => UVec::new(0, self.get_height() / 2),
            VAlign::Bottom => UVec::new(0, 0),
        },
            // Add the margin to the Position
        ) + IVec::new(pos.margin.0, pos.margin.1);

        self.set_pos(UVec::from(position));
        self
    }

    fn get_width(&self) -> u16;
    fn get_height(&self) -> u16;
    fn get_pos(&self) -> UVec;
    fn set_pos(&mut self, pos: UVec);
    /* generate the pixel data to draw the widget */
    fn make(&self) -> Vec<Color>;
}

/// Our implementations of this trait will probably all have these getters, setters,
/// so make a macro to spam these automatically by using widget!() in the impl of a widget.
macro_rules! widget {
    () => {
        fn get_width(&self) -> u16 {
            self.width
        }

        fn get_height(&self) -> u16 {
            self.height
        }

        fn get_pos(&self) -> UVec {
            self.pos
        }

        fn set_pos(&mut self, pos: UVec) {
            self.pos = pos;
        }
    };
}
