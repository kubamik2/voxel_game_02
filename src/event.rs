use std::{any::Any, collections::VecDeque};

pub trait Event {
    fn is_handled(&self) -> bool;
    fn set_is_handled(&mut self, value: bool);
    fn event_type(&self) -> EventType;
    fn as_any(&self) -> &dyn Any;
}

#[repr(u8)] 
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventType {
    Window,
    KeyboardInput,
}

pub struct EventQueue {
    events: [VecDeque<Box<dyn Event>>; std::mem::variant_count::<EventType>()]
}

impl EventQueue {
    pub fn new() -> Self {
        Self { events: std::array::from_fn(|_| VecDeque::new()) }
    }

    pub fn push_event(&mut self, event_type: &EventType, event: Box<dyn Event>) {
        self.events[*event_type as u8 as usize].push_back(event);
    }

    pub fn pop_event(&mut self, event_type: &EventType) -> Option<Box<dyn Event>> {
        self.events[*event_type as u8 as usize].pop_front()
    }
}