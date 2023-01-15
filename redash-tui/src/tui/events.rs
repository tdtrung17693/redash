use pancurses::Input;

#[derive(Debug, PartialEq, PartialOrd, Hash, Eq, Clone)]
pub enum EventType {
    Focus,
    KeyPress,
    Submit,
    ValueChange,
    ChangeInputMode,
}

pub enum ComponentType {
    InputBox,
    SelectableList,
    SystemBar,
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub enum EventData {
    Char(char),
    String(String),
    Number(i32),
    Bool(bool),
    Key(Input),
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub event_data: EventData,
}

// impl EventTrait for Event {
//     type EventData<'a>
//     where
//         Self: 'a;

//     fn get_data<'a>(&'a self) -> &Self::EventData<'a> {
//         todo!()
//     }
// }

impl Event {
    pub fn new(event_type: EventType, event_value: EventData) -> Self {
        Event {
            event_type,
            event_data: event_value,
        }
    }
}

pub type EventHandler<'a> = dyn FnMut(Event) + 'a;
