
#[macro_export]
macro_rules! fsm_event_unit {
    ($event_name:ident) => {
        #[derive(Copy, Clone, PartialEq, Default, Debug)]
        pub struct $event_name;
        impl FsmEvent for $event_name {}
    }
}
