mod registry;
mod trait_def;
mod input_semantics;

#[cfg(feature = "local-candle")]
pub mod local_candle;

pub use registry::{global_registry, ProviderRegistry};
pub use trait_def::{Provider, ProviderFeature};
pub use input_semantics::InputSemantics;

#[cfg(feature = "local-candle")]
pub use local_candle::LocalCandleProvider;
