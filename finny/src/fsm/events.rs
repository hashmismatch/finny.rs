use crate::{FsmBackend, FsmEventQueue, FsmEventQueueSender, TimerId, lib::*};

/// The internal event type that also allows stopping or starting the machine.
pub enum FsmEvent<E> {
    Start,
    Stop,
    Timer(TimerId),
    Event(E)
}

impl<E> From<E> for FsmEvent<E> {
    fn from(event: E) -> Self {
        FsmEvent::Event(event)
    }
}

impl<E> Debug for FsmEvent<E> where E: Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsmEvent::Start => f.write_str("Fsm::Start"),
            FsmEvent::Stop => f.write_str("Fsm::Stop"),
            FsmEvent::Timer(t) => f.write_fmt(format_args!("Fsm::Timer({})", t)),
            FsmEvent::Event(ev) => ev.fmt(f)
        }
    }
}

impl<E> AsRef<str> for FsmEvent<E> where E: AsRef<str> {
    fn as_ref(&self) -> &str {
        match self {
            FsmEvent::Start => "Fsm::Start",
            FsmEvent::Stop => "Fsm::Stop",
            FsmEvent::Timer(_) => "Fsm::Timer",
            FsmEvent::Event(e) => e.as_ref()
        }
    }
}

pub type FsmRegionId = usize;

/// The context that is given to all of the guards and actions.
pub struct EventContext<'a, TFsm, Q> where TFsm: FsmBackend, Q: FsmEventQueueSender<TFsm> {
    pub context: &'a mut TFsm::Context,
    pub queue: &'a mut Q,
    pub region: FsmRegionId
}

impl<'a, TFsm, Q> Deref for EventContext<'a, TFsm, Q> where TFsm: FsmBackend, Q: FsmEventQueueSender<TFsm>
{
    type Target = <TFsm as FsmBackend>::Context;

    fn deref(&self) -> &Self::Target {
        self.context
    }
}

impl<'a, TFsm: FsmBackend, Q> DerefMut for EventContext<'a, TFsm, Q> where TFsm: FsmBackend, Q: FsmEventQueueSender<TFsm> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.context
    }
}
