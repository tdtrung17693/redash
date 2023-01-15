use super::{
    events::EventData,
    renderer::{RenderBox, TextStyle},
    Component, ComponentType, Event, EventHandler, EventType, Position,
};
use pancurses::{curs_set, Input, Window};
use std::{cell::RefCell, cmp::min, rc::Rc};
use std::{cmp::max, collections::HashMap, fmt::Display};

/// Input box
/// # Examples
/// ```
/// let a = InputBox { value = String::from("asd"), width = 12 }
/// ```
pub struct SelectableList<'a, T: Display> {
    label: String,
    width: i32,
    height: i32,
    viewport_top: i32,
    value: i32,
    items: Vec<T>,
    is_focus: bool,
    is_disabled: bool,
    event_listeners: HashMap<EventType, Vec<Rc<RefCell<EventHandler<'a>>>>>,
}

impl<'a, T: Display> SelectableList<'a, T> {
    pub fn new(label: &str, width: i32, height: i32, items: Vec<T>, is_disabled: bool) -> Self {
        SelectableList {
            label: String::from(label),
            width,
            height,
            viewport_top: 0,
            value: 0,
            items,
            is_focus: false,
            is_disabled,
            event_listeners: HashMap::new(),
        }
    }

    fn is_overflow(&self) -> bool {
        let sz = self.items.len();
        sz > (self.height - 2) as usize
    }

    pub fn append_items(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn clear(&mut self) {
        self.items = Vec::new()
    }

    pub fn set_scroll(&mut self, scroll_top: i32) {
        self.viewport_top = scroll_top;
    }

    pub fn next_item(&mut self) {
        // if disabled, just scroll through page instead of through item
        let delta = if self.is_disabled { self.height - 1 } else { 1 };
        let len = self.items.len() as i32;
        self.value = min(self.value + delta, len - 1);

        if self.value >= self.viewport_top + self.height - 2 {
            self.set_scroll(self.viewport_top + 1);
        }

        if !self.is_disabled {
            self.value_changed();
        }
    }

    pub fn prev_item(&mut self) {
        let delta = if self.is_disabled { self.height - 1 } else { 1 };
        self.value = max(0, self.value - delta);

        if self.value < self.viewport_top {
            self.set_scroll(self.viewport_top - 1);
        }

        if !self.is_disabled {
            self.value_changed();
        }
    }

    fn value_changed(&mut self) {
        let submit_handlers = self.event_listeners.get_mut(&EventType::ValueChange);
        if let Some(handlers) = submit_handlers {
            for handler in handlers {
                let p = handler.clone();

                let mut f = p.borrow_mut();
                f(Event::new(
                    EventType::ValueChange,
                    EventData::Number(self.value),
                ))
            }
        }
    }
}

impl<'a, T: Display> Component<'a> for SelectableList<'a, T> {
    fn render(&self, renderer: &super::renderer::Renderer, position: &Position) {
        let &Position { top, left } = position;
        renderer.draw_box(&RenderBox {
            top: top as usize,
            left: left as usize,
            width: self.width as usize,
            height: self.height as usize,
        });
        renderer.draw_string(
            &self.label,
            TextStyle::Normal,
            self.is_focus,
            &RenderBox {
                top: top as usize,
                left: left as usize + 2,
                width: (self.width - 2) as usize,
                height: 0,
            },
        );

        renderer.clear_rect(&RenderBox {
            top: (top + 1) as usize,
            left: (left + 1) as usize,
            width: (self.width - 2) as usize,
            height: (self.height - 2) as usize,
        });

        // render new value
        let rendering_items = if self.is_overflow() {
            let viewport_top = self.viewport_top as usize;
            let height = self.height as usize;
            let value_sz = self.items.len();
            let start = max(0, self.viewport_top as usize);
            let end = if value_sz - viewport_top > height - 2 {
                viewport_top + height - 2
            } else {
                value_sz
            };

            &self.items[start..end]
        } else {
            &self.items[..]
        };
        if self.is_overflow() {
            let len = self.items.len() as i32;
            renderer.draw_vscrollbar(
                &RenderBox {
                    top: (top + 1) as usize,
                    left: (left + self.width - 1) as usize,
                    width: 0,
                    height: (self.height - 3) as usize,
                },
                ((self.viewport_top) as f64) / (((len as i32) - self.height + 2) as f64),
            )
        }
        for (index, item) in rendering_items.iter().enumerate() {
            let index = index as i32;
            let style = if !self.is_disabled && index + self.viewport_top == self.value {
                TextStyle::Bold
            } else {
                TextStyle::Normal
            };

            renderer.draw_string(
                &format!("{item}"),
                style,
                false,
                &RenderBox {
                    top: (top + 1 + index) as usize,
                    left: (left + 1) as usize,
                    width: (self.width - 3) as usize,
                    height: 0,
                },
            );
        }
    }

    fn render_focus(&self, window: &Window, position: &Position) {
        if !self.is_focus {
            return;
        }
        let &Position { top, left } = position;

        curs_set(0);
        window.mv(top + 1 + self.value - self.viewport_top, left + 1);
    }

    fn trigger(&mut self, event: &Event) -> bool {
        if !self.is_focus {
            return false;
        }

        if let Event {
            event_type: EventType::KeyPress,
            event_data,
        } = event
        {
            match event_data {
                EventData::Key(Input::KeyUp) => {
                    self.prev_item();
                    return true;
                }
                EventData::Key(Input::KeyDown) => {
                    self.next_item();
                    return true;
                }
                _ => return false,
            }
        } else if let Event {
            event_type: EventType::Focus,
            ..
        } = event
        {
            self.is_focus = true;
            return true;
        }

        return false;
    }

    fn add_event_listener(
        &mut self,
        event_type: EventType,
        handler: Rc<RefCell<EventHandler<'a>>>,
    ) {
        let event_handlers = self.event_listeners.entry(event_type).or_insert(Vec::new());
        event_handlers.push(handler);
    }

    fn toggle_focus(&mut self) {
        self.is_focus = !self.is_focus
    }

    fn get_component_type(&self) -> super::ComponentType {
        ComponentType::SelectableList
    }

    fn is_focusable(&self) -> bool {
        return true;
    }
}
