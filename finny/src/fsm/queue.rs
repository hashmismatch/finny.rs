use lib::*;

use crate::{FsmBackend, FsmResult};


/// The event queueing trait for FSMs. Can be used from outside or from within the actions of the FSM.
pub trait FsmEventQueue<F: FsmBackend> {
    /// Try to enqueue an event.
    fn enqueue<E: Into<<F as FsmBackend>::Events>>(&mut self, event: E) -> FsmResult<()>;
    /// Try to dequeue an event.
    fn dequeue(&mut self) -> Option<<F as FsmBackend>::Events>;
}

#[cfg(feature = "std")]
mod queue_vec {
    use super::*;

    /// An unbound event queue that uses `VecDeque`.
    pub struct FsmEventQueueVec<F: FsmBackend> {
        queue: VecDeque<<F as FsmBackend>::Events>
    }

    
    impl<F: FsmBackend> FsmEventQueueVec<F> {
        pub fn new() -> Self {
            FsmEventQueueVec {
                queue: VecDeque::new()
            }
        }
    }

    impl<F: FsmBackend> FsmEventQueue<F> for FsmEventQueueVec<F> {
        fn enqueue<E: Into<<F as FsmBackend>::Events>>(&mut self, event: E) -> FsmResult<()> {
            self.queue.push_back(event.into());
            Ok(())
        }

        fn dequeue(&mut self) -> Option<<F as FsmBackend>::Events> {
            self.queue.pop_front()
        }
    }
}

#[cfg(feature = "std")]
pub use self::queue_vec::*;

mod queue_heapless {
    use super::*;

    /// A heapless queue with a fixed size. Implemented using the `heapless` crate.
    pub struct FsmEventQueueHeapless<F: FsmBackend, N>
        where N: heapless::ArrayLength<<F as FsmBackend>::Events>
    {
        vec: heapless::Vec<<F as FsmBackend>::Events, N>
    }

    impl<F, N> FsmEventQueueHeapless<F, N>
        where F: FsmBackend, N: heapless::ArrayLength<<F as FsmBackend>::Events>
    {
        pub fn new() -> Self {
            FsmEventQueueHeapless {
                vec: heapless::Vec::new()
            }
        }
    }

    impl<F, N> FsmEventQueue<F> for FsmEventQueueHeapless<F, N> 
        where F: FsmBackend, N: heapless::ArrayLength<<F as FsmBackend>::Events>
    {
        fn enqueue<E: Into<<F as FsmBackend>::Events>>(&mut self, event: E) -> FsmResult<()> {
            todo!()
        }

        fn dequeue(&mut self) -> Option<<F as FsmBackend>::Events> {
            todo!()
        }
    }
}

pub use self::queue_heapless::*;

pub struct FsmEventQueueNull<F> {
    _ty: PhantomData<F>
}

impl<F> FsmEventQueueNull<F> {
    pub fn new() -> Self {
        FsmEventQueueNull { _ty: PhantomData::default() }
    }
}

impl<F: FsmBackend> FsmEventQueue<F> for FsmEventQueueNull<F> {
    fn enqueue<E: Into<<F as FsmBackend>::Events>>(&mut self, _event: E) -> FsmResult<()> {
        Ok(())
    }

    fn dequeue(&mut self) -> Option<<F as FsmBackend>::Events> {
        None
    }
}