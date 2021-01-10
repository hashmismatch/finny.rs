use crate::{EventContext, FsmBackend, FsmBackendImpl, FsmEventQueue, FsmRegionId, Inspect};

pub struct DispatchContext<'a, 'b, 'c, F, Q, I>
    where F: FsmBackend,
    Q: FsmEventQueue<F>,
    I: Inspect
{
    pub queue: &'a mut Q,
    pub inspect: &'b mut I,
    pub backend: &'c mut FsmBackendImpl<F>
}

impl<'a, 'b, 'c, F, Q, I> DispatchContext<'a, 'b, 'c, F, Q, I>
where F: FsmBackend,
    Q: FsmEventQueue<F>,
    I: Inspect
{

    pub fn to_event_context(&'a mut self, region: FsmRegionId) -> EventContext<'a, F, Q>
    {
        EventContext {
            context: &mut self.backend.context,
            queue: self.queue,
            region
        }
    }
}