#[cfg(all(feature = "lib", feature = "custom_events"))]
compile_error!("feature ragout/custom_events and ragout/lib can not e enabled concurrently");

#[cfg(feature = "custom_events")]
pub use ragout_input_macro::ragout_input;

#[cfg(feature = "lib")]
pub mod ragout;

#[cfg(feature = "lib")]
pub use ragout::*;
