use arc_swap::{access::{Access, DynAccess}, ArcSwap};

use crate::typemap::TypeMap;
use std::sync::{mpsc::{channel, Sender, Receiver}, Arc};


pub trait Event = 'static + Send + Clone + Sync;

#[derive(Clone)]
pub struct EventQueue<E: Event> {
    queue_old: Vec<E>,
    queue_old_state_event_count: u32,
    queue_new: Vec<E>,
    queue_new_state_event_count: u32,
    event_count: u32,
}

impl<E: Event> EventQueue<E> {
    pub fn new() -> Self {
        Self {
            queue_old: vec![],
            queue_new: vec![],
            queue_old_state_event_count: 0,
            queue_new_state_event_count: 0,
            event_count: 0,
        }
    }

    pub fn send(&mut self, event: E) {
        self.queue_new.push(event);
        self.event_count += 1;
    }

    pub fn update(&mut self) {
        std::mem::swap(&mut self.queue_new, &mut self.queue_old);
        self.queue_new.clear();
        self.queue_old_state_event_count = self.queue_new_state_event_count;
        self.queue_new_state_event_count += self.queue_old.len() as u32;
    }
}

pub struct EventReader<E: Event> {
    last_event_counter: u32,
    event_queue: Arc<EventQueue<E>>,
    event_manager: EventManager,
}

impl<E: Event> EventReader<E> {
    pub fn new(event_manager: &EventManager) -> Self {
        let last_event_counter = event_manager
            .get_event_queue::<E>()
            .unwrap_or_else(|| panic!("get_event_queue() failed, {} not found", std::any::type_name::<E>()))
            .load()
            .event_count;
        let event_queue = event_manager.get_event_queue::<E>().unwrap().load_full();

        Self {
            last_event_counter,
            event_queue,
            event_manager: event_manager.clone(),
        }
    }

    #[inline]
    pub fn read(&mut self) -> EventIterator<E> {
        self.event_queue = self.event_manager.get_event_queue::<E>().unwrap().load_full();
        EventIterator::new(&self.event_queue, &mut self.last_event_counter)
    }
}

pub struct EventIterator<'a, E: Event> {
    inner: std::iter::Chain<std::slice::Iter<'a, E>, std::slice::Iter<'a, E>>,
    last_event_counter: &'a mut u32,
}

impl<'a, E: Event> EventIterator<'a, E> {
    pub fn new(event_queue: &'a EventQueue<E>, last_event_counter: &'a mut u32) -> Self {
        *last_event_counter = (*last_event_counter).max(event_queue.queue_old_state_event_count);
        let queue_old_sliced_index = ((*last_event_counter - event_queue.queue_old_state_event_count) as usize).min(event_queue.queue_old.len());
        let queue_new_sliced_index = (last_event_counter.saturating_sub(event_queue.queue_new_state_event_count) as usize).min(event_queue.queue_new.len());

        let queue_old_sliced = &event_queue.queue_old[queue_old_sliced_index..];
        let queue_new_sliced = &event_queue.queue_new[queue_new_sliced_index..];

        Self {
            inner: queue_old_sliced.iter().chain(queue_new_sliced.iter()),
            last_event_counter,
        }
    }
}

impl<'a, E: Event> Iterator for EventIterator<'a, E> {
    type Item = &'a E;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.inner.next() {
            *self.last_event_counter += 1;
            return Some(item);
        }
        None
    }
}

type EventUpdateFunction = Box<dyn Fn(&TypeMap) + Send + Sync>;
type EventsItem<E> = ArcSwap<EventQueue<E>>;

#[derive(Clone)]
pub struct EventManager(Arc<EventManagerBuilder>);

impl EventManager {
    #[inline]
    pub fn get_event_queue<E: Event>(&self) -> Option<&EventsItem<E>> {
        self.0.get_event_queue()
    }

    #[inline]
    pub fn send<E: Event>(&self, event: E) {
        let mut event_queue: EventQueue<E> = self.get_event_queue::<E>().unwrap().load().as_ref().clone();
        event_queue.send(event);
        self.get_event_queue::<E>().unwrap().store(Arc::new(event_queue));
    }

    pub fn update(&self) {
        for update in self.0.event_queue_updates.iter() {
            update(&self.0.events);
        }
    }

    pub fn create_reader<E: Event>(&self) -> EventReader<E> {
        EventReader::new(self)
    }
}

#[derive(Default)]
pub struct EventManagerBuilder {
    events: TypeMap,
    event_queue_updates: Vec<EventUpdateFunction>,
}

impl EventManagerBuilder {
    #[inline]
    pub fn get_event_queue<E: Event>(&self) -> Option<&EventsItem<E>> {
        self.events.get::<EventsItem<E>>()
    }

    #[inline]
    fn insert_new_event_queue<E: Event>(&mut self) {
        self.events.insert::<EventsItem<E>>(ArcSwap::from(Arc::new(EventQueue::new())));
    }

    pub fn register_event_type<E: Event>(mut self) -> Self {
        fn update_event_queue<E: Event>(events: &TypeMap) {
            let mut event_queue: EventQueue<E> = events.get::<EventsItem<E>>().unwrap().load().as_ref().clone();
            event_queue.update();
            events.get::<EventsItem<E>>().unwrap().store(Arc::new(event_queue));
        }
        if self.get_event_queue::<E>().is_some() { return self; }
        self.insert_new_event_queue::<E>();
        self.event_queue_updates.push(Box::new(|events| { update_event_queue::<E>(events) }));
        self
    }

    pub fn build(self) -> EventManager {
        EventManager(Arc::new(self))
    }
}
