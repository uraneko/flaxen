#[cfg(all(feature = "lib", feature = "custom_events"))]
compile_error!("feature ragout/custom_events and ragout/lib can not e enabled concurrently");

#[cfg(feature = "custom_events")]
pub use ragout_custom_events_macro::ragout_custom_events;

#[cfg(feature = "lib")]
pub mod ragout;
pub use ragout::*;
