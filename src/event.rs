use crate::typemap::TypeMap;

pub struct EventQueue<E: 'static> {
    queue_old: Vec<E>,
    queue_old_state_event_count: u32,
    queue_new: Vec<E>,
    queue_new_state_event_count: u32,
    event_count: u32,
}

impl<E: 'static> EventQueue<E> {
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

pub struct EventReader<E: 'static> {
    last_event_counter: u32,
    _generic: std::marker::PhantomData<E>,
}

impl<E: 'static> EventReader<E> {
    pub fn new(events: &Events) -> Self {
        let last_event_counter = events.get_event_queue::<E>().expect(std::any::type_name::<E>()).event_count;

        Self {
            last_event_counter,
            _generic: std::marker::PhantomData
        }
    }

    pub fn read<'a>(&'a mut self, events: &'a Events) -> EventIterator<'a, E> {
        let event_queue = events.get_event_queue::<E>().unwrap();
        EventIterator::new(event_queue, &mut self.last_event_counter)
    }
}

pub struct EventIterator<'a, E: 'static> {
    inner: std::iter::Chain<std::slice::Iter<'a, E>, std::slice::Iter<'a, E>>,
    last_event_counter: &'a mut u32,
}

impl<'a, E: 'static> EventIterator<'a, E> {
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

impl<'a, E: 'static> Iterator for EventIterator<'a, E> {
    type Item = &'a E;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.inner.next() {
            *self.last_event_counter += 1;
            return Some(item);
        }
        None
    }
}

pub struct Events {
    events: TypeMap,
    event_queue_updates: Vec<Box<dyn FnMut(&mut TypeMap)>>,
}

impl Events {
    pub fn new() -> Self {
        Self { events: TypeMap::new(), event_queue_updates: vec![] }
    }

    pub fn get_event_queue<E: 'static>(&self) -> Option<&EventQueue<E>> {
        self.events.get::<EventQueue<E>>()
    }

    pub fn get_mut_event_queue<E: 'static>(&mut self) -> Option<&mut EventQueue<E>> {
        self.events.get_mut::<EventQueue<E>>()
    }

    fn insert_new_event_queue<E: 'static>(&mut self) {
        self.events.insert::<EventQueue<E>>(EventQueue::new());
    }

    #[inline]
    pub fn register_event_type<E: 'static>(&mut self) {
        fn update_event_queue<E: 'static>(events: &mut TypeMap) {
            let event_queue = events.get_mut::<EventQueue<E>>().unwrap();
            event_queue.update();
        }
        if self.get_event_queue::<E>().is_some() { return; }
        self.insert_new_event_queue::<E>();
        self.event_queue_updates.push(Box::new(|events| { update_event_queue::<E>(events) }))
    }

    #[inline]
    pub fn send<E: 'static>(&mut self, event: E) {
        let event_queue = self.get_mut_event_queue().expect(std::any::type_name::<E>());
        event_queue.send(event);
    }

    pub fn update(&mut self) {
        for update in self.event_queue_updates.iter_mut() {
            update(&mut self.events);
        }
    }

}