use crate::lib::*;
use crate::{FsmBackend, FsmResult};

/// The event queueing trait for FSMs. Can be used from outside or from within the actions of the FSM.
pub trait FsmEventQueue<F: FsmBackend>: FsmEventQueueSender<F> {
    /// Try to dequeue an event.
    fn dequeue(&mut self) -> Option<<F as FsmBackend>::Events>;
    /// Number of messages to be dequeued.
    fn len(&self) -> usize;
}

pub trait FsmEventQueueSender<F: FsmBackend> {
    /// Try to enqueue an event.
    fn enqueue<E: Into<<F as FsmBackend>::Events>>(&mut self, event: E) -> FsmResult<()>;
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
        fn dequeue(&mut self) -> Option<<F as FsmBackend>::Events> {
            self.queue.pop_front()
        }

        fn len(&self) -> usize {
            self.queue.len()
        }
    }

    impl<F: FsmBackend> FsmEventQueueSender<F> for FsmEventQueueVec<F> {
        fn enqueue<E: Into<<F as FsmBackend>::Events>>(&mut self, event: E) -> FsmResult<()> {
            self.queue.push_back(event.into());
            Ok(())
        }
    }
}

#[cfg(feature = "std")]
pub use self::queue_vec::*;

#[cfg(feature = "std")]
mod queue_vec_shared {
    use std::sync::{Arc, Mutex};

    use crate::FsmError;

    use super::*;

    /// An unbound event queue that uses `VecDeque`.
    pub struct FsmEventQueueVecShared<F: FsmBackend> {
        inner: Inner<F>
    }

    impl<F> Clone for FsmEventQueueVecShared<F> where F: FsmBackend {
        fn clone(&self) -> Self {
            Self { inner: Inner { queue: self.inner.queue.clone() } }
        }
    }

    struct Inner<F: FsmBackend> {
        queue: Arc<Mutex<VecDeque<<F as FsmBackend>::Events>>>
    }
    
    impl<F: FsmBackend> FsmEventQueueVecShared<F> {
        pub fn new() -> Self {
            let q = VecDeque::new();
            let inner = Inner {
                queue: Arc::new(Mutex::new(q))
            };
            FsmEventQueueVecShared {
                inner
            }
        }
    }

    impl<F: FsmBackend> FsmEventQueue<F> for FsmEventQueueVecShared<F> {
        fn dequeue(&mut self) -> Option<<F as FsmBackend>::Events> {
            if let Ok(mut q) = self.inner.queue.lock() {
                q.pop_front()
            } else {
                None
            }
        }

        fn len(&self) -> usize {
            if let Ok(q) = self.inner.queue.lock() {
                q.len()
            } else {
                0
            }
        }
    }

    impl<F: FsmBackend> FsmEventQueueSender<F> for FsmEventQueueVecShared<F> {
        fn enqueue<E: Into<<F as FsmBackend>::Events>>(&mut self, event: E) -> FsmResult<()> {
            if let Ok(mut q) = self.inner.queue.lock() {
                q.push_back(event.into());
                Ok(())
            } else {
                Err(FsmError::QueueOverCapacity)
            }
        }
    }
}

#[cfg(feature = "std")]
pub use self::queue_vec_shared::*;

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
        fn dequeue(&mut self) -> Option<<F as FsmBackend>::Events> {
            self.dequeue.pop_front()
        }

        fn len(&self) -> usize {
            self.dequeue.len()
        }
    }

    impl<F, A> FsmEventQueueSender<F> for FsmEventQueueArray<F, A> 
        where F: FsmBackend, A: Array<Item = <F as FsmBackend>::Events>
    {
        fn enqueue<E: Into<<F as FsmBackend>::Events>>(&mut self, event: E) -> FsmResult<()> {
            match self.dequeue.push_back(event.into()) {
                Ok(_) => Ok(()),
                Err(_) => Err(crate::FsmError::QueueOverCapacity)
            }
        }
    }
}

pub use self::queue_array::*;


pub mod heapless_shared {
    //! A heapless queue with Clone and Arc support.
    
    use core::sync::atomic::{AtomicUsize, Ordering};

    use crate::FsmError;

    use super::*;

    extern crate alloc;
    use alloc::sync::Arc;
    use heapless::mpmc::Q64;

/// An unbound event queue that uses `VecDeque`.
    pub struct FsmEventQueueHeaplessShared<F: FsmBackend> {
        inner: Arc<Inner<F>>
    }

    impl<F> Clone for FsmEventQueueHeaplessShared<F> where F: FsmBackend {
        fn clone(&self) -> Self {
            Self { inner: self.inner.clone() }
        }
    }

    struct Inner<F: FsmBackend> {
        queue: Q64<<F as FsmBackend>::Events>,
        len: AtomicUsize
    }
    
    impl<F: FsmBackend> FsmEventQueueHeaplessShared<F> {
        pub fn new() -> Self {
            let q = Q64::new();
            let inner = Inner {
                queue: q,
                len: AtomicUsize::new(0)
            };
            FsmEventQueueHeaplessShared {
                inner: Arc::new(inner)
            }
        }
    }

    impl<F: FsmBackend> FsmEventQueue<F> for FsmEventQueueHeaplessShared<F> {
        fn dequeue(&mut self) -> Option<<F as FsmBackend>::Events> {
            match self.inner.queue.dequeue() {
                Some(e) => {
                    self.inner.len.fetch_sub(1, Ordering::SeqCst);
                    Some(e)
                },
                None => None
            }
        }

        fn len(&self) -> usize {
            self.inner.len.load(Ordering::SeqCst)
        }
    }

    impl<F: FsmBackend> FsmEventQueueSender<F> for FsmEventQueueHeaplessShared<F> {
        fn enqueue<E: Into<<F as FsmBackend>::Events>>(&mut self, event: E) -> FsmResult<()> {
            match self.inner.queue.enqueue(event.into()) {
                Ok(_) => {
                    self.inner.len.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                },
                Err(_) => Err(FsmError::QueueOverCapacity) 
            }
        }
    }

}

pub struct FsmEventQueueNull<F> {
    _ty: PhantomData<F>
}

impl<F> FsmEventQueueNull<F> {
    pub fn new() -> Self {
        FsmEventQueueNull { _ty: PhantomData::default() }
    }
}

impl<F: FsmBackend> FsmEventQueue<F> for FsmEventQueueNull<F> {
    fn dequeue(&mut self) -> Option<<F as FsmBackend>::Events> {
        None
    }

    fn len(&self) -> usize {
        0
    }
}

impl<F: FsmBackend> FsmEventQueueSender<F> for FsmEventQueueNull<F> {
    fn enqueue<E: Into<<F as FsmBackend>::Events>>(&mut self, _event: E) -> FsmResult<()> {
        Ok(())
    }
}

pub struct FsmEventQueueSub<'a, Q, F, FSub>
    where 
        F: FsmBackend,
        Q: FsmEventQueueSender<F>
{
    pub parent: &'a mut Q,
    pub _parent_fsm: PhantomData<F>,
    pub _sub_fsm: PhantomData<FSub>
}

impl<'a, Q, F, FSub> FsmEventQueue<FSub> for FsmEventQueueSub<'a, Q, F, FSub>
    where 
        F: FsmBackend,
        Q: FsmEventQueueSender<F>,
        FSub: FsmBackend,
        <F as FsmBackend>::Events: From<<FSub as FsmBackend>::Events>
{
    fn dequeue(&mut self) -> Option<<FSub as FsmBackend>::Events> {
        None
    }

    fn len(&self) -> usize {
        0
    }
}

impl<'a, Q, F, FSub> FsmEventQueueSender<FSub> for FsmEventQueueSub<'a, Q, F, FSub>
    where 
        F: FsmBackend,
        Q: FsmEventQueueSender<F>,
        FSub: FsmBackend,
        <F as FsmBackend>::Events: From<<FSub as FsmBackend>::Events>
{
    fn enqueue<E: Into<<FSub as FsmBackend>::Events>>(&mut self, event: E) -> FsmResult<()>
    {
        self.parent.enqueue(event.into())
    }
}



#[cfg(test)]
use super::tests_fsm::TestFsm;

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

#[test]
fn test_dequeue_vec_shared() {
    let queue = FsmEventQueueVecShared::<TestFsm>::new();
    test_queue(queue);
}

#[test]
fn test_heapless_shared() {
    use self::heapless_shared::FsmEventQueueHeaplessShared;
    let queue = FsmEventQueueHeaplessShared::<TestFsm>::new();
    test_queue(queue);
}

#[cfg(test)]
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