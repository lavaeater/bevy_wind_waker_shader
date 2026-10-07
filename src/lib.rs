#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![doc = include_str!("../readme.md")]
#![recursion_limit = "256"]
pub mod prelude {
    //! Everything you need to get started.
    //!
    //!  See [`WindWakerShaderBuilder`] for Wind Waker style,
    //! [`FlatShaderPlugin`] / [`FlatShaderBuilder`] for flat Sable-style shading, and
    //! [`PixelShaderPlugin`] / [`PixelShaderBuilder`] for pixelation.
    pub use crate::{
        TimeOfDay, Weather, WindWakerShader, WindWakerShaderBuilder, WindWakerShaderPlugin,
        flat::{FlatExtendedMaterial, FlatShader, FlatShaderBuilder, FlatShaderPlugin},
        pixelate::{PixelExtendedMaterial, PixelShader, PixelShaderBuilder, PixelShaderPlugin},
    };
}

pub use components::{
    ExtendedMaterial, TimeOfDay, Weather, WindWakerShader, WindWakerShaderBuilder,
};
pub use plugin::WindWakerShaderPlugin;

pub mod flat;
pub mod pixelate;

mod components;
mod plugin;
mod systems;
