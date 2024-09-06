use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStatus {
    Working,
    Idle
}

#[derive(Debug)]
pub enum ThreadWorkDispatcherError<T> {
    SendError(std::sync::mpsc::SendError<T>),
    NoIdleThreadsAvailable
}

pub struct ThreadWorkDispatcher<I: Send + 'static, O: Send + 'static> {
    senders: Vec<Sender<I>>,
    receivers: Vec<Receiver<O>>,
    threads_status: Vec<ThreadStatus>,
    thread_pool: rayon::ThreadPool,
}

impl<I: Send + 'static, O: Send + 'static> ThreadWorkDispatcher<I, O> {
    pub fn new<F: FnOnce(Receiver<I>, Sender<O>) + Send + 'static + Clone>(num_threads: usize, f: F) -> Self {
        let mut senders = vec![];
        let mut receivers = vec![];
        let mut threads_status = vec![];
        let thread_pool = rayon::ThreadPoolBuilder::new().num_threads(num_threads).build().unwrap();
        for _ in 0..num_threads {
            let f_clone = f.clone();
            let (input_sender, input_receiver) = channel();
            let (output_sender, output_receiver) = channel();
            thread_pool.spawn(move || { f_clone(input_receiver, output_sender) });
            senders.push(input_sender);
            receivers.push(output_receiver);
            threads_status.push(ThreadStatus::Idle);
        }

        Self { senders, receivers, threads_status, thread_pool }
    }

    #[inline]
    fn get_idle_thread_index(&self) -> Option<usize> {
        self.threads_status.iter().position(|p| *p == ThreadStatus::Idle)
    }

    #[inline]
    pub fn idle_threads(&self) -> usize {
        self.threads_status.iter().filter(|p| **p == ThreadStatus::Idle).count()
    }

    pub fn collect_outputs(&mut self) -> Box<[O]> {
        let mut outputs = vec![];
        for (i, receiver) in self.receivers.iter().enumerate() {
            if let Ok(output) = receiver.try_recv() {
                outputs.push(output);
                self.threads_status[i] = ThreadStatus::Idle;
            }
        }

        outputs.into_boxed_slice()
    }

    pub fn dispatch_work(&mut self, input: I) -> Result<(), ThreadWorkDispatcherError<I>> {
        match self.get_idle_thread_index() {
            Some(i) => {
                self.threads_status[i] = ThreadStatus::Working;
                self.senders[i].send(input).map_err(|err| ThreadWorkDispatcherError::SendError(err))
            },
            None => Err(ThreadWorkDispatcherError::NoIdleThreadsAvailable)
        }
    }
    
    pub fn iter_outputs<'a>(&'a mut self) -> impl Iterator<Item = O> + 'a {
        self.receivers
        .iter()
        .enumerate()
        .map(move |(i, f)| (i, f.try_recv()))
        .map(|(i, f)| f.and_then(|op| {
            self.threads_status[i] = ThreadStatus::Idle; Ok(op)
        }))
        .filter_map(|f| f.ok())
    }
}