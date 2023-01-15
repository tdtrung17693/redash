use super::{
    renderer::{RenderBox, TextStyle},
    Component, ComponentType, Event, EventHandler, EventType, Position,
};
use pancurses::Window;
use std::{cell::RefCell, rc::Rc};

/// Input box
/// # Examples
/// ```
/// let a = InputBox { value = String::from("asd"), width = 12 }
/// ```
pub struct SystemBar {
    label: String,
    width: i32,
}

impl SystemBar {
    pub fn new(label: &str, width: i32) -> Self {
        SystemBar {
            label: String::from(label),
            width,
        }
    }
}

impl<'a> Component<'a> for SystemBar {
    fn render(&self, renderer: &super::renderer::Renderer, position: &Position) {
        let &Position { top, left } = position;

        renderer.draw_string(
            &self.label,
            TextStyle::Normal,
            false,
            &RenderBox {
                top: top as usize,
                left: left as usize,
                width: (self.width) as usize,
                height: 1,
            },
        );
    }

    fn render_focus(&self, _: &Window, _: &Position) {}

    fn trigger(&mut self, _: &Event) -> bool {
        return false;
    }

    fn add_event_listener(&mut self, _: EventType, _: Rc<RefCell<EventHandler<'a>>>) {}

    fn toggle_focus(&mut self) {}

    fn get_component_type(&self) -> super::ComponentType {
        ComponentType::SystemBar
    }

    fn is_focusable(&self) -> bool {
        return false;
    }
}
