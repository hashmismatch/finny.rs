use lib::*;

use crate::FsmResult;


/// The event queueing trait for FSMs. Can be used from outside from the inside of the FSM.
pub trait FsmEventQueue<T> {
    /// Try to enqueue an event.
    fn enqueue(&mut self, event: T) -> FsmResult<()>;
    /// Try to dequeue an event.
    fn dequeue(&mut self) -> Option<T>;
}

#[cfg(feature = "std")]
mod queue_vec {
    use super::*;

    /// An unbound event queue that uses `VecDeque`.
    pub struct FsmEventQueueVec<T> {
        queue: VecDeque<T>
    }

    
    impl<T> FsmEventQueueVec<T> {
        pub fn new() -> Self {
            FsmEventQueueVec {
                queue: VecDeque::new()
            }
        }
    }

    impl<T> FsmEventQueue<T> for FsmEventQueueVec<T> {
        fn enqueue(&mut self, event: T) -> FsmResult<()> {
            self.queue.push_back(event);
            Ok(())
        }

        fn dequeue(&mut self) -> Option<T> {
            self.queue.pop_front()
        }
    }
}

#[cfg(feature = "std")]
pub use self::queue_vec::*;

mod queue_heapless {
    use super::*;

    /// A heapless queue with a fixed size. Implemented using the `heapless` crate.
    pub struct FsmEventQueueHeapless<T, N>
        where N: heapless::ArrayLength<T>
    {
        vec: heapless::Vec<T, N>
    }

    impl<T, N> FsmEventQueueHeapless<T, N>
        where N: heapless::ArrayLength<T>
    {
        pub fn new() -> Self {
            FsmEventQueueHeapless {
                vec: heapless::Vec::new()
            }
        }
    }

    impl<T, N> FsmEventQueue<T> for FsmEventQueueHeapless<T, N> 
        where N: heapless::ArrayLength<T>
    {
        fn enqueue(&mut self, event: T) -> FsmResult<()> {
            todo!()
        }

        fn dequeue(&mut self) -> Option<T> {
            todo!()
        }
    }
}

pub use self::queue_heapless::*;