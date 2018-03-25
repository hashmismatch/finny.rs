



#[derive(Copy, Clone, Debug)]
pub struct TimerSettings<E> {
	pub timeout: TimerDuration,
	pub cancel_on_state_exit: bool,
	pub event_on_timeout: E
}

#[derive(Copy, Clone, Debug)]
pub struct TransitionTimerSettings {
	pub timeout: TimerDuration
}
impl TransitionTimerSettings {
	pub fn new(timeout: TimerDuration) -> Self {
		TransitionTimerSettings {
			timeout: timeout
		}
	}
}



#[derive(Copy, Clone, Debug)]
pub struct TimerDuration {
	pub ms: u64
}

impl TimerDuration {
	pub fn from_millis(ms: u64) -> Self {
		TimerDuration { ms: ms }
	}
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TimerId(pub u32);

pub trait FsmTimers: Clone {
	fn implemented() -> bool { true }
	fn create_timeout_timer(&mut self, id: TimerId, duration: TimerDuration);
	fn cancel_timer(&mut self, id: TimerId);
	
	fn receive_events(&mut self) -> Vec<FsmTimerEvent>;
}

#[derive(Copy, Clone, Debug)]
pub struct FsmTimersNull;
impl FsmTimers for FsmTimersNull {
	fn implemented() -> bool { false }

	fn create_timeout_timer(&mut self, id: TimerId, duration: TimerDuration) { }
	fn cancel_timer(&mut self, id: TimerId) { }
	
	fn receive_events(&mut self) -> Vec<FsmTimerEvent> { vec![] }
}

#[derive(Copy, Clone, Debug)]
pub enum FsmTimerEvent {
	TimedOut(FsmTimerTimedOut)
}

#[derive(Copy, Clone, Debug)]
pub struct FsmTimerTimedOut {
	pub timer_id: TimerId
}


