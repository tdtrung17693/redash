use pancurses::{Input, Window};
use std::{cell::RefCell, cmp::min, collections::HashMap, rc::Rc};

use super::{
    events::EventData,
    renderer::{RenderBox, TextStyle},
    Component, ComponentType, Event, EventHandler, EventType, Position,
};

/// Input box
/// # Examples
/// ```
/// let a = InputBox { value = String::from("asd"), width = 12 }
/// ```
pub struct InputBox<'a> {
    label: String,
    width: i32,
    value: String,
    is_focus: bool,
    event_listeners: HashMap<EventType, Vec<Rc<RefCell<EventHandler<'a>>>>>,
}

impl<'a> InputBox<'a> {
    pub fn new(label: &str, width: i32) -> Self {
        InputBox {
            label: String::from(label),
            width,
            value: String::new(),
            is_focus: false,
            event_listeners: HashMap::new(),
        }
    }
    fn is_overflow(&self) -> bool {
        return self.width - 2 < self.value.len() as i32;
    }
    pub fn focus(&mut self) {
        self.is_focus = true;
    }

    pub fn clear(&mut self) {
        self.value = String::new();
    }
}

impl<'a> Component<'a> for InputBox<'a> {
    fn render(&self, renderer: &super::renderer::Renderer, position: &Position) {
        let &Position { top, left } = position;
        renderer.draw_box(&RenderBox {
            top: top as usize,
            left: left as usize,
            width: self.width as usize,
            height: 3,
        });
        renderer.draw_string(
            &self.label,
            TextStyle::Normal,
            &RenderBox {
                top: top as usize,
                left: left as usize + 2,
                width: 0,
                height: 0,
            },
        );

        // clear old line
        renderer.clear_rect(&RenderBox {
            top: (top + 1) as usize,
            left: (left + 1) as usize,
            width: (self.width - 2) as usize,
            height: 1,
        });

        // render new value
        let value_text_render_box = RenderBox {
            top: (top + 1) as usize,
            left: (left + 1) as usize,
            width: 0,
            height: 0,
        };

        let rendering_text = if self.is_overflow() {
            let value_sz = self.value.len();
            let start = value_sz - (self.width as usize - 3);

            &self.value[start..]
        } else {
            &self.value[..]
        };
        renderer.draw_string(&rendering_text, TextStyle::Normal, &value_text_render_box);
    }

    fn render_focus(&self, window: &Window, position: &Position) {
        if !self.is_focus {
            return;
        }
        let &Position { top, left, .. } = position;

        let sz = self.value.len() as i32;
        let cursor_x = min(self.width - 3, sz);
        window.mv(top + 1, left + 1 + cursor_x);
    }

    fn trigger(&mut self, event: &Event) {
        if !self.is_focus {
            return;
        }
        if let Event {
            event_type: EventType::KeyPress,
            event_data,
        } = event
        {
            match event_data {
                EventData::Char(c) => match c {
                    '\n' => {
                        let submit_handlers = self.event_listeners.get_mut(&EventType::Submit);
                        if let Some(handlers) = submit_handlers {
                            for handler in handlers {
                                let p = handler.clone();

                                let mut f = p.borrow_mut();
                                f(Event::new(
                                    EventType::Submit,
                                    EventData::String(self.value.clone()),
                                ))
                            }
                        } else {
                            println!("no handler");
                        }
                        self.clear();
                    }
                    '\r' => return,
                    a if !a.is_ascii_control() || a == &'\n' || a == &'\r' => {
                        self.value = format!("{}{a}", self.value)
                    }
                    _ => return,
                },
                EventData::Key(Input::KeyBackspace) => {
                    if self.value.is_empty() {
                        return;
                    }
                    let value_sz = self.value.len();
                    self.value = String::from(&self.value[..(value_sz - 1)]);
                }
                _ => return,
            }
        } else if let Event {
            event_type: EventType::Focus,
            ..
        } = event
        {
            self.is_focus = true;
        }
    }

    fn toggle_focus(&mut self) {
        self.is_focus = !self.is_focus
    }

    fn add_event_listener(
        &mut self,
        event_type: EventType,
        handler: Rc<RefCell<EventHandler<'a>>>,
    ) {
        let event_handlers = self.event_listeners.entry(event_type).or_insert(Vec::new());
        event_handlers.push(handler);
    }

    fn get_component_type(&self) -> super::ComponentType {
        ComponentType::InputBox
    }
}
