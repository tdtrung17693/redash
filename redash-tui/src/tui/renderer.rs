use pancurses::{Window, A_BOLD, A_ITALIC, A_NORMAL};

pub struct Renderer<'a> {
    window: &'a Window,
}

pub struct RenderBox {
    pub top: usize,
    pub left: usize,
    pub width: usize,
    pub height: usize,
}

pub enum TextStyle {
    Normal,
    Bold,
    Italic,
}

impl<'a> Renderer<'a> {
    pub fn new(window: &'a Window) -> Self {
        Renderer { window }
    }

    pub fn clear_rect(
        &self,
        &RenderBox {
            top,
            left,
            width,
            height,
        }: &RenderBox,
    ) {
        let window = self.window;
        let signed_top = top as i32;
        let signed_left = left as i32;

        let mut clear_string = String::new();
        for _ in 0..width {
            clear_string = format!("{clear_string} ");
        }

        window.mv(signed_top, signed_left);

        for i in 0..height {
            window.mv(signed_top + (i as i32), signed_left);
            window.addstr(&clear_string);
        }
    }

    pub fn draw_box(&self, render_box: &RenderBox) {
        let window = self.window;
        let &RenderBox {
            top,
            left,
            width,
            height,
        } = render_box;
        let signed_top = top as i32;
        let signed_left = left as i32;
        let signed_width = width as i32;
        window.mv(signed_top, signed_left);
        window.addstr(box_drawing::light::DOWN_RIGHT);

        for _ in 0..(render_box.width - 2) {
            window.addstr(box_drawing::light::HORIZONTAL);
        }
        window.addstr(box_drawing::light::DOWN_LEFT);

        window.mv(signed_top + 1, signed_left);
        for cur_y in top + 1..(top + height - 1) {
            let cur_y = cur_y as i32;
            window.addstr(box_drawing::light::VERTICAL);
            window.mv(cur_y, signed_left + signed_width - 1);
            window.addstr(box_drawing::light::VERTICAL);
            window.mv(cur_y + 1, signed_left);
        }

        window.addstr(box_drawing::light::UP_RIGHT);
        for _ in 0..(width - 2) {
            window.addstr(box_drawing::light::HORIZONTAL);
        }
        window.addstr(box_drawing::light::UP_LEFT);
    }

    pub fn draw_vscrollbar(
        &self,
        &RenderBox {
            top,
            left,
            width,
            height,
        }: &RenderBox,
        scroll_top: i32,
    ) {
        let window = self.window;
        let signed_top = top as i32;
        let signed_left = left as i32;
        window.mv(signed_top, signed_left + width as i32);
        window.addstr("↟");
        window.mv(signed_top + scroll_top + 1, signed_left);
        window.addstr("█");
        window.mv(signed_top + height as i32, signed_left);
        window.addstr("↡");
    }

    pub fn draw_string(&self, string: &str, text_style: TextStyle, render_box: &RenderBox) {
        let &RenderBox { top, left, .. } = render_box;
        let window = self.window;
        let signed_top = top as i32;
        let signed_left = left as i32;
        let attr = match text_style {
            TextStyle::Bold => A_BOLD,
            TextStyle::Normal => A_NORMAL,
            TextStyle::Italic => A_ITALIC,
        };
        let old_attr = window.attrget();
        window.attrset(attr);
        window.mv(signed_top, signed_left);
        window.addstr(string);
        window.attrset(old_attr.0);
    }
}
