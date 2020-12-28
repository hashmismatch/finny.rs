/*
use crate::{EventContext, FsmCore, FsmCoreDispatch, FsmCoreImpl, FsmEventQueue, FsmEventQueueVec, FsmResult, FsmStates};

// provided

pub struct SampleContext {
    val: usize
}

#[derive(Debug, Default)]
pub struct StateA { a: usize }
#[derive(Debug, Default)]
pub struct StateB { b: usize }

pub struct EventA;
pub struct EventB;

// generated
#[derive(Default)]
pub struct SampleStates {
    state_a: StateA,
    state_b: StateB
}

pub enum SamplesStatesEnum {
    StateA,
    StateB
}

impl FsmStates for SampleStates {
    type StateKind = SamplesStatesEnum;
}

pub enum SampleEvents { 
    EventA(EventA),
    EventB(EventB)
}

impl<Q> FsmCoreDispatch for FsmCoreImpl<SampleContext, SampleStates, SampleEvents, Q>
    where Self: FsmCore<Context = SampleContext>
{
    fn dispatch(&mut self, event: &Self::Events) -> FsmResult<()> {

        let ev_ctx: EventContext<Self> = EventContext {
            context: &mut self.context
        };

        todo!("foo")
    }
}

pub struct SampleFsmBuilder;
impl SampleFsmBuilder {
    pub fn new<Q>(context: SampleContext, queue: Q) -> FsmResult<FsmCoreImpl<SampleContext, SampleStates, SampleEvents, Q>>
        where Q: FsmEventQueue<SampleEvents>
    {
        FsmCoreImpl::new_all(context, SampleStates::default(), queue)
    }
}



#[test]
fn test_mocked_codegen() -> FsmResult<()> {

    let mut fsm = SampleFsmBuilder::new(SampleContext { val: 0}, FsmEventQueueVec::new())?;

    fsm.dispatch(&SampleEvents::EventA(EventA));
    

    Ok(())
}
*/