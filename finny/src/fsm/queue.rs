use lib::*;

use crate::FsmResult;


pub trait FsmEventQueue<T> {
    fn enqueue(&mut self, event: T) -> FsmResult<()>;
    fn dequeue(&mut self) -> Option<T>;
}

#[cfg(feature = "std")]
pub struct FsmEventQueueVec<T> {
    queue: VecDeque<T>
}

#[cfg(feature = "std")]
impl<T> FsmEventQueue<T> for FsmEventQueueVec<T> {
    fn enqueue(&mut self, event: T) -> FsmResult<()> {
        self.queue.push_back(event);
        Ok(())
    }

    fn dequeue(&mut self) -> Option<T> {
        self.queue.pop_front()
    }
}

#[cfg(feature = "std")]
impl<T> FsmEventQueueVec<T> {
    pub fn new() -> Self {
        FsmEventQueueVec {
            queue: VecDeque::new()
        }
    }
}