#[cfg(feature="timers_std")]
pub mod std_noalloc;

#[cfg(feature="timers_std")]
pub mod std;

pub mod core;

#[cfg(feature="async")]
pub mod tokio;