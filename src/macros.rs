
#[macro_export]
macro_rules! fsm_event_unit {
    ($event_name:ident) => {
        #[derive(Copy, Clone, PartialEq, Default, Debug)]
        pub struct $event_name;
        impl FsmEvent for $event_name {}

                
        impl ::serde::Serialize for $event_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: ::serde::Serializer
            {
                use ::serde::ser::SerializeStruct;
                serializer.serialize_unit()
            }
        }
    }
}
