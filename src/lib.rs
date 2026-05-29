#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![doc = include_str!("../readme.md")]

pub mod prelude {
    //! Everything you need to get started. See [`WindWakerShaderBuilder`] for Wind Waker style,
    //! [`FlatShaderPlugin`] / [`FlatShaderBuilder`] for flat Sable-style shading, and
    //! [`PixelShaderPlugin`] / [`PixelShaderBuilder`] for pixelation.
    pub use crate::{
        flat::{FlatExtendedMaterial, FlatShader, FlatShaderBuilder, FlatShaderPlugin},
        pixelate::{PixelExtendedMaterial, PixelShader, PixelShaderBuilder, PixelShaderPlugin},
        TimeOfDay, Weather, WindWakerShader, WindWakerShaderBuilder, WindWakerShaderPlugin,
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
