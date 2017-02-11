# Hierarchical state machines for Rust

Inspired by Boost MSM. Generates the transition tables using procedural macros. Requires at least Rust 1.16.

Consider this a preview, undocumented, alpha-level release.

[![Build Status](https://travis-ci.org/hashmismatch/fsm.rs.svg?branch=master)](https://travis-ci.org/hashmismatch/fsm.rs)

## Features

- Static states
- Submachines with optional state preserving events
- Orthogonal regions
- Interrupt states
- State guards
- Queued events
- Enum generation for event and state types
- Internal state transitions that don't trigger the entry and exit events
- Helpers for multiple entry states	
- Graphviz visualisation, the codegen generates a test that saves the graph file to the filesystem
- Inspection trait for custom debugging	

## Example

```rust
extern crate fsm;
#[macro_use]
extern crate fsm_codegen;

use fsm::*;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Event1 { v: usize }
impl FsmEvent for Event1 {}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct EventGo;
impl FsmEvent for EventGo {}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct EventRestart;
impl FsmEvent for EventRestart {}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct EventStart;
impl FsmEvent for EventStart {}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct EventStop;
impl FsmEvent for EventStop {}

#[derive(Clone, PartialEq, Default)]
pub struct StateA { a: usize }
impl FsmState<FsmMinTwo> for StateA { }

#[derive(Clone, PartialEq, Default)]
pub struct StateB { b: usize }
impl FsmState<FsmMinTwo> for StateB { }

#[derive(Clone, PartialEq, Default)]
pub struct StateC { c: usize }
impl FsmState<FsmMinTwo> for StateC { }

#[derive(Fsm)]
struct FsmMinTwoDefinition(
	InitialState<FsmMinTwo, StateA>,

    Transition         < FsmMinTwo,  StateA,        EventStart,        StateB,    NoAction >,
    Transition         < FsmMinTwo,  StateB,        EventStop,         StateA,    NoAction >,
    Transition         < FsmMinTwo,  StateB,        EventGo,           StateC,    NoAction >,

    Transition         < FsmMinTwo, (StateA, StateB, StateC),
                                                    EventRestart,      StateA,    NoAction >,

    TransitionInternal < FsmMinTwo,  StateA,        Event1,                       NoAction >,

    InterruptState     < FsmMinTwo,  StateB,        EventStop >
);

#[cfg(test)]
#[test]
fn test_fsm_min2() {
    let mut fsm = FsmMinTwo::new(());
    fsm.start();
    assert_eq!(FsmMinTwoStates::StateA, fsm.get_current_state());

    fsm.process_event(FsmMinTwoEvents::EventStart(EventStart)).unwrap();
    assert_eq!(FsmMinTwoStates::StateB, fsm.get_current_state());    
}
```