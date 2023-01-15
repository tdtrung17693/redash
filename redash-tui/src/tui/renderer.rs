use pancurses::{Window, A_BOLD, A_ITALIC, A_NORMAL, COLOR_PAIR};

use crate::constants::FOCUS_COLOR;

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
        scroll_percent: f64,
    ) {
        let window = self.window;
        let signed_top = top as i32;
        let signed_left = left as i32;
        let scroll_handler_y = (scroll_percent * ((height - 2) as f64)) as i32;

        window.mv(signed_top, signed_left + width as i32);
        window.addstr("↟");
        window.mv(signed_top + scroll_handler_y + 1, signed_left);
        window.addstr("█");
        window.mv((top + height) as i32, signed_left);
        window.addstr("↡");
    }

    pub fn draw_string(
        &self,
        string: &str,
        text_style: TextStyle,
        is_focus: bool,
        render_box: &RenderBox,
    ) {
        let &RenderBox {
            top, left, width, ..
        } = render_box;
        let window = self.window;
        let signed_top = top as i32;
        let signed_left = left as i32;
        let text_weight_attr = match text_style {
            TextStyle::Bold => A_BOLD,
            TextStyle::Normal => A_NORMAL,
            TextStyle::Italic => A_ITALIC,
        };
        let mut attr_flag = text_weight_attr;
        if is_focus {
            attr_flag |= COLOR_PAIR(FOCUS_COLOR);
        }
        let mut string = String::from(string);
        let len = string.len();
        if len > width {
            // ellipsis if width >= 1
            if width >= 1 {
                string = format!("{}{}", &string[..width], "…");
            } else {
                string = String::new();
            }
        }
        window.attron(attr_flag);
        window.mv(signed_top, signed_left);
        window.addstr(string);
        window.attroff(attr_flag);
    }
}
