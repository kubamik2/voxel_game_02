use std::sync::{mpsc::{channel, Receiver, Sender}, Arc};

#[derive()]
pub struct ThreadStatus(std::sync::atomic::AtomicU8);

impl ThreadStatus {
    #[inline]
    pub fn is_working(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Relaxed) == 1
    }

    #[inline]
    pub fn is_idle(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Relaxed) == 0
    }

    #[inline]
    pub fn set_working(&self) {
        self.0.store(1, std::sync::atomic::Ordering::Relaxed);
    }

    #[inline]
    pub fn set_idle(&self) {
        self.0.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    pub const fn idle() -> Self {
        Self(std::sync::atomic::AtomicU8::new(0))
    }

    pub fn working() -> Self {
        Self(std::sync::atomic::AtomicU8::new(1))
    }
}

#[derive(Debug)]
pub enum ThreadWorkDispatcherError<T> {
    SendError(std::sync::mpsc::SendError<T>),
    NoIdleThreadsAvailable
}

pub struct ThreadWorkDispatcher<I: Send + Sync + 'static, O: Send + Sync + 'static> {
    senders: Box<[Sender<I>]>,
    receivers: Box<[Receiver<O>]>,
    threads_status: Box<[ThreadStatus]>,
}

unsafe impl<I: Send + Sync + 'static, O: Send + Sync + 'static> Sync for ThreadWorkDispatcher<I, O> {}

impl<I: Send + 'static + Sync, O: Send + 'static + Sync> ThreadWorkDispatcher<I, O> {
    pub fn new<F: FnOnce(Receiver<I>, Sender<O>) + Send + 'static + Clone>(num_threads: usize, f: F) -> Self {
        let mut senders = vec![];
        let mut receivers = vec![];
        let mut threads_status = vec![];
        let _thread_pool = rayon::ThreadPoolBuilder::new().num_threads(num_threads).build().unwrap();
        for _ in 0..num_threads {
            let f_clone = f.clone();
            let (input_sender, input_receiver) = channel();
            let (output_sender, output_receiver) = channel();
            _thread_pool.spawn(move || { f_clone(input_receiver, output_sender) });
            senders.push(input_sender);
            receivers.push(output_receiver);
            threads_status.push(ThreadStatus::idle());
        }

        Self {
            senders: senders.into_boxed_slice(),
            receivers: receivers.into_boxed_slice(),
            threads_status: threads_status.into_boxed_slice(),
        }
    }

    #[inline]
    fn get_idle_thread_index(&self) -> Option<usize> {
        self.threads_status.iter().position(|p| p.is_idle())
    }

    #[inline]
    pub fn idle_threads(&self) -> usize {
        self.threads_status.iter().filter(|p| p.is_idle()).count()
    }

    pub fn collect_outputs(&self) -> Box<[O]> {
        let mut outputs = vec![];
        for (i, receiver) in self.receivers.iter().enumerate() {
            if let Ok(output) = receiver.try_recv() {
                outputs.push(output);
                self.threads_status[i].set_idle();
            }
        }

        outputs.into_boxed_slice()
    }

    pub fn dispatch_work(&self, input: I) -> Result<(), ThreadWorkDispatcherError<I>> {
        match self.get_idle_thread_index() {
            Some(i) => {
                self.threads_status[i].set_working();
                self.senders[i].send(input).map_err(|err| ThreadWorkDispatcherError::SendError(err))
            },
            None => Err(ThreadWorkDispatcherError::NoIdleThreadsAvailable)
        }
    }
 
    pub fn iter_outputs(&self) -> impl Iterator<Item = O> + '_ {
        self.receivers
        .iter()
        .enumerate()
        .map(move |(i, f)| (i, f.try_recv()))
        .map(|(i, f)| f.and_then(|op| {
            self.threads_status[i].set_idle(); Ok(op)
        }))
        .filter_map(|f| f.ok())
    }
}
