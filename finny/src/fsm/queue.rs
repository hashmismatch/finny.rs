use crate::lib::*;
use crate::{FsmBackend, FsmResult};
use super::tests_fsm::TestFsm;

/// The event queueing trait for FSMs. Can be used from outside or from within the actions of the FSM.
pub trait FsmEventQueue<F: FsmBackend> {
    /// Try to enqueue an event.
    fn enqueue<E: Into<<F as FsmBackend>::Events>>(&mut self, event: E) -> FsmResult<()>;
    /// Try to dequeue an event.
    fn dequeue(&mut self) -> Option<<F as FsmBackend>::Events>;
    /// Number of messages to be dequeued.
    fn len(&self) -> usize;
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

        fn len(&self) -> usize {
            self.queue.len()
        }
    }
}

#[cfg(feature = "std")]
pub use self::queue_vec::*;

mod queue_array {
    use arraydeque::{Array, ArrayDeque};

    use super::*;

    /// A heapless queue with a fixed size. Implemented using the `arraydequeue` crate.
    pub struct FsmEventQueueArray<F, A>
        where F: FsmBackend, A: Array<Item = <F as FsmBackend>::Events>, Self: Sized
    {
        dequeue: ArrayDeque<A>,
        _fsm: PhantomData<F>
    }

    impl<F, A> FsmEventQueueArray<F, A>
        where F: FsmBackend, A: Array<Item = <F as FsmBackend>::Events>
    {
        pub fn new() -> Self {
            Self {
                dequeue: ArrayDeque::new(),
                _fsm: PhantomData::default()
            }
        }
    }

    impl<F, A> FsmEventQueue<F> for FsmEventQueueArray<F, A> 
        where F: FsmBackend, A: Array<Item = <F as FsmBackend>::Events>
    {
        fn enqueue<E: Into<<F as FsmBackend>::Events>>(&mut self, event: E) -> FsmResult<()> {
            match self.dequeue.push_back(event.into()) {
                Ok(_) => Ok(()),
                Err(_) => Err(crate::FsmError::QueueOverCapacity)
            }
        }

        fn dequeue(&mut self) -> Option<<F as FsmBackend>::Events> {
            self.dequeue.pop_front()
        }

        fn len(&self) -> usize {
            self.dequeue.len()
        }
    }
}

pub use self::queue_array::*;

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

    fn len(&self) -> usize {
        0
    }
}

#[test]
fn test_dequeue_vec() {
    let queue = FsmEventQueueVec::<TestFsm>::new();
    test_queue(queue);
}

#[test]
fn test_array() {
    let queue = FsmEventQueueArray::<TestFsm, [_; 16]>::new();
    test_queue(queue);
}

fn test_queue<Q: FsmEventQueue<TestFsm>>(mut queue: Q) {
    use super::tests_fsm::{Events, EventA};

    // fill and drain
    {
        for i in 0..5 {
            assert_eq!(i, queue.len());
            queue.enqueue(EventA { n: i }).unwrap();            
            assert_eq!(i+1, queue.len());
        }

        for i in 0..5 {
            assert_eq!(5-i, queue.len());
            let ev = queue.dequeue().unwrap();
            assert_eq!(Events::EventA(EventA { n: i }), ev);            
            assert_eq!(5-i-1, queue.len());
        }
    }
    assert_eq!(None, queue.dequeue());

    // zipper - enqueue 2, drain 1
    {
        let mut n = 0;
        let mut x = 0;
        for _ in 0..10 {
            queue.enqueue(EventA { n }).unwrap();
            n += 1;
            queue.enqueue(EventA { n }).unwrap();
            n += 1;

            {
                let ev = queue.dequeue().unwrap();
                assert_eq!(Events::EventA(EventA { n: x }), ev);
                x += 1;
            }

            assert_eq!(queue.len(), x);
        }
    }
}