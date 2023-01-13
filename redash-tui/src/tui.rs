use std::{cell::RefCell, error::Error, rc::Rc};
extern crate pancurses;

use pancurses::{Input, Window};
use redash_client::client::Client;

use redash_client::Data;

use crate::app::CommandEntry;

use self::{events::*, input_box::InputBox, renderer::Renderer, selectable_list::SelectableList};

mod renderer;

pub mod events;
pub mod input_box;
pub mod selectable_list;

pub trait Component<'a> {
    fn render(&self, renderer: &Renderer, position: &Position);
    fn render_focus(&self, window: &Window, position: &Position);
    fn toggle_focus(&mut self);
    fn add_event_listener(&mut self, event: EventType, handler: Rc<RefCell<EventHandler<'a>>>);
    fn trigger(&mut self, event: &Event);
    fn get_component_type(&self) -> ComponentType;
}

pub struct Position {
    top: i32,
    left: i32,
}

impl Position {
    pub fn new(top: i32, left: i32) -> Self {
        Position { top, left }
    }
}

fn create_cell<T>(t: T) -> Rc<RefCell<T>> {
    return Rc::new(RefCell::new(t));
}

pub struct App<'a> {
    window: &'a Window,
    redis_client: &'a Client,
    history_list: Rc<RefCell<Vec<CommandEntry>>>,
    components: Vec<(Rc<RefCell<dyn Component<'a> + 'a>>, Position)>,
    renderer: Renderer<'a>,
    current_focus: usize,
}

impl<'a> App<'a> {
    pub fn new(window: &'a Window, redis_client: &'a Client) -> Self {
        return App {
            window,
            history_list: Rc::new(RefCell::new(Vec::new())),
            redis_client,
            renderer: Renderer::new(window),
            components: Vec::new(),
            current_focus: 0,
        };
    }

    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        let window = self.window;
        let redis_client = self.redis_client;
        let (screen_height, screen_width) = window.get_max_yx();

        let input_box = create_cell(InputBox::new(
            "Command",
            ((screen_width as f32) * 0.2_f32).floor() as i32,
        ));
        let command_list = create_cell(SelectableList::new(
            "History",
            ((screen_width as f32) * 0.2_f32).floor() as i32,
            screen_height - 3,
            Vec::new(),
            false,
        ));

        let result_list = create_cell(SelectableList::new(
            "Result",
            ((screen_width as f32) * 0.8_f32).floor() as i32 - 1,
            screen_height,
            Vec::new(),
            true,
        ));

        let input_box_cloned = input_box.clone();
        let command_list_cloned = command_list.clone();
        let result_list_cloned = result_list.clone();
        let history_list = self.history_list.clone();
        let input_submit_handler = move |event| match (command_list_cloned).try_borrow_mut() {
            Ok(mut a) => {
                if let Event {
                    event_data: EventData::String(command),
                    ..
                } = event
                {
                    match result_list_cloned.try_borrow_mut() {
                        Ok(mut result_lst) => {
                            let response = redis_client.send_command(&command);

                            result_lst.clear();
                            match response {
                                Ok(result) => {
                                    {
                                        match &result {
                                            Data::Array(array) => {
                                                for data in array {
                                                    result_lst.append_items(format!("{data}"))
                                                }
                                            }

                                            data => result_lst.append_items(format!("{data}")),
                                        }
                                    }
                                    let mut history_list = history_list.borrow_mut();
                                    history_list.push(CommandEntry {
                                        command: command.clone(),
                                        response: result,
                                    });
                                }
                                Err(err) => result_lst.append_items(format!("{err}")),
                            }

                            a.append_items(command);
                            a.next_item();
                        }
                        _ => {}
                    }
                }
            }
            Err(_) => (),
        };

        if let Ok(mut p) = input_box_cloned.try_borrow_mut() {
            p.add_event_listener(
                EventType::Submit,
                Rc::new(RefCell::new(input_submit_handler)),
            );
        }

        let command_list_cloned = command_list.clone();
        let result_list_cloned = result_list.clone();
        let history_list = self.history_list.clone();
        let list_value_change_handler = move |event| {
            if let Event {
                event_data: EventData::Number(value),
                ..
            } = event
            {
                match result_list_cloned.try_borrow_mut() {
                    Ok(mut result_lst) => {
                        result_lst.clear();
                        let history_list = history_list.borrow();
                        let entry = history_list.get(value as usize).unwrap();
                        match &entry.response {
                            Data::Array(array) => {
                                for data in array {
                                    result_lst.append_items(format!("{data}"))
                                }
                            }

                            data => result_lst.append_items(format!("{data}")),
                        }
                    }
                    _ => {}
                }
            }
        };
        if let Ok(mut p) = command_list_cloned.try_borrow_mut() {
            p.add_event_listener(
                EventType::ValueChange,
                Rc::new(RefCell::new(list_value_change_handler)),
            );
        }

        self.add_component(input_box, Position::new(0, 0))?;
        self.add_component(command_list, Position::new(3, 0))?;
        self.add_component(
            result_list,
            Position::new(0, ((screen_width as f32) * 0.2_f32).floor() as i32 + 2),
        )?;
        Ok(())
    }

    pub fn handle_input(&mut self, input: Input) -> Result<(), Box<dyn Error>> {
        match input {
            Input::Character('\t') => self.focus_next(),
            Input::KeyBackspace | Input::KeyUp | Input::KeyDown | Input::KeyEnter => {
                self.trigger_event(&Event::new(EventType::KeyPress, EventData::Key(input)))?
            }
            Input::Character(c) => {
                self.trigger_event(&Event::new(EventType::KeyPress, EventData::Char(c)))?
            }
            _ => (),
        }

        Ok(())
    }
    fn add_component(
        &mut self,
        component: Rc<RefCell<dyn Component<'a> + 'a>>,
        position: Position,
    ) -> Result<(), Box<dyn Error>> {
        let is_empty = self.components.is_empty();
        self.components.push((component, position));
        if is_empty {
            // Focus first component by default
            let c = self.components[0].0.clone();
            let mut c = c.try_borrow_mut()?;
            c.toggle_focus();
        }
        Ok(())
    }

    pub fn render(&self) {
        self.components.iter().for_each(|p| {
            let c = p.0.clone();

            match c.try_borrow() {
                Ok(c) => c.render(&self.renderer, &p.1),
                Err(_) => (),
            };
        });
        self.components.iter().for_each(|p| {
            let c = p.0.clone();

            match c.try_borrow() {
                Ok(c) => c.render_focus(&self.window, &p.1),
                Err(_) => (),
            };
        })
    }

    pub fn trigger_event(&mut self, event: &Event) -> Result<(), Box<dyn Error>> {
        for component in &mut self.components {
            let c = component.0.clone();

            let mut c = c.try_borrow_mut()?;
            c.trigger(event)
        }

        Ok(())
    }

    pub fn focus_next(&mut self) {
        let total_component = self.components.len();

        // remove current focus
        let current_focus = self.components[self.current_focus].0.clone();
        // let mut current_focus = current_focus.borrow_mut();
        match current_focus.try_borrow_mut() {
            Ok(mut c) => c.toggle_focus(),
            Err(_) => (),
        };

        // set next component focus
        self.current_focus = (self.current_focus + 1) % total_component;
        let current_focus = self.components[self.current_focus].0.clone();
        match current_focus.try_borrow_mut() {
            Ok(mut c) => c.toggle_focus(),
            Err(_) => (),
        };
    }
}

pub fn run(redis_client: &Client, window: &Window) -> Result<(), Box<dyn Error>> {
    let mut app = App::new(window, redis_client);
    app.init()?;

    loop {
        app.render();
        match window.getch() {
            Some(Input::KeyDC) => break,
            Some(input) => app.handle_input(input)?,
            None => (),
        }
    }
    Ok(())
}
