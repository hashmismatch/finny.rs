todo
=====

- ~~~generate methods to call the current machine's state on_ methods, for use in sub machines. generate code that matches the current state enum.~~~
- ~~~history? skip start() for transitions that have shallow history?~~~
- ~~~stop()?~~~
- ~~~internal events - Event + State + Action, doesn't invoke on_exit & on_enter for the same state~~~
- stringify of field names for state types. PreviousSong -> previous_song
- ~~~dotty transition graph~~~
   - ~~~fix initial transitions (check fsm1.rs)~~~
   - ~~~mark internal transitions (they don't call on_exit & on_enter)~~~
- ~~~big todo: orthogonal regions~~~
- ~~~state reachability test~~~
- ~~~guards~~~ (non-mutable, is this ok?)
- ~~~event queuing from actions (and states?)~~~
- any event
- list of states as the source... tuple?
- ~~~fsm.get_state::<State1>()~~~
- terminal state?
- deferred events (related to queue)
- ~~~interrupt state~~~
   - todo: graph check for reachability of resume events (need to be in the same region!)

- FsmStateFactory should be generic on the trait type, not on the trait method - state could be reused in multiple contexts
- rewrite the boost tests so that states store their field in themselves, not in the context



potential
=========
- idea: for Fsm<I = Inspection> ... just generate type FsmImpl<I> and then typedef Fsm = FsmImpl<NullInspection> as a default?
    - kinda messy... currently resolving with a codegen type


- new_state_timeout_transitition::<StateFrom, StateTo, _>(|| timeout settings)
 - creates a new numbered event, etc...