use bevy::asset::{Asset, Handle, uuid_handle};
use bevy::pbr::{MaterialExtension, StandardMaterial};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

pub const SHADER_HANDLE: Handle<Shader> = uuid_handle!("ddeed264-efde-495e-9159-4ac3db07f9f8");
pub const TEXTURE_HANDLE: Handle<Image> = uuid_handle!("1af26f3e-5605-4723-a036-dc83f357c7d8");

/// The type of the material that will be inserted for you after you insert the [`WindWakerShader`] via the [`WindWakerShaderBuilder`] into an entity.
pub type ExtendedMaterial = bevy::pbr::ExtendedMaterial<StandardMaterial, WindWakerShader>;

/// Build via the [`WindWakerShaderBuilder`] and insert into an entity to give it a shader that looks
/// like the one used by characters in The Legend of Zelda: The Wind Waker.
///
/// After insertion, the shader will be moved into the [`ExtendedMaterial`] of the entity.
#[derive(Asset, AsBindGroup, PartialEq, Debug, Clone, Component, Reflect)]
#[reflect(PartialEq)]
pub struct WindWakerShader {
    #[texture(100)]
    #[sampler(101)]
    mask: Handle<Image>,
    /// The parts of the model that are facing the light source and are not in shadow.
    #[uniform(102)]
    pub highlight_color: LinearRgba,
    /// The parts of the model that are not facing the light source and are in shadow.
    #[uniform(103)]
    pub shadow_color: LinearRgba,
    /// The color of the edge of the model, which gets a slight specular highlight to make the model pop.
    #[uniform(104)]
    pub rim_color: LinearRgba,
}

impl Default for WindWakerShader {
    fn default() -> Self {
        WindWakerShaderBuilder::default().build()
    }
}

impl MaterialExtension for WindWakerShader {
    fn fragment_shader() -> ShaderRef {
        SHADER_HANDLE.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        SHADER_HANDLE.into()
    }
}

/// Builds a new [`WindWakerShader`] by setting the parameters to look like those in The Legend of Zelda: The Wind Waker.
///
/// After insertion, the shader will be moved into the [`ExtendedMaterial`] of the entity.
/// If the entity in question is a [`Scene`](bevy::prelude::Scene), this is done for all the entities inside the scene.
///
/// # Example
///
/// ```
/// use bevy::prelude::*;
/// use bevy_wind_waker_shader::prelude::*;
///
/// fn spawn_with_wind_waker_shader(mut commands: Commands, asset_server: Res<AssetServer>) {
///     commands.spawn((
///         WorldAssetRoot(asset_server.load("models/Fox.glb")),
///         WindWakerShaderBuilder::default()
///             .time_of_day(TimeOfDay::Afternoon)
///             .weather(Weather::Sunny)
///             .build(),
///    ));
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct WindWakerShaderBuilder {
    time_of_day: TimeOfDay,
    weather: Weather,
    override_highlight_color: Option<Color>,
    override_shadow_color: Option<Color>,
    override_rim_color: Option<Color>,
}

/// Palette entry helper: the hex digits of the original Wind Waker colors, as a [`Color`].
fn rgb(r: u8, g: u8, b: u8) -> Color {
    Srgba::rgb_u8(r, g, b).into()
}

impl WindWakerShaderBuilder {
    /// Uses the color palette associated with the given time of day in The Legend of Zelda: The Wind Waker.
    /// Note that the [weather](WindWakerShaderBuilder::weather) will modify the colors.
    ///
    /// The default time of day is [TimeOfDay::Day].
    #[must_use]
    pub const fn time_of_day(mut self, time: TimeOfDay) -> Self {
        self.time_of_day = time;
        self
    }

    /// Modifies the color palette associated with the [time of day](WindWakerShaderBuilder::time_of_day) by the given weather.
    ///
    /// The default weather is [Weather::Sunny].
    #[must_use]
    pub const fn weather(mut self, weather: Weather) -> Self {
        self.weather = weather;
        self
    }

    /// Overrides the highlight color with the given color. Highlights are the parts of the model that are facing the light source and are not in shadow.
    /// This overrides both the [time of day](WindWakerShaderBuilder::time_of_day) and [weather](WindWakerShaderBuilder::weather) settings.
    #[must_use]
    pub const fn override_highlight_color(mut self, color: Color) -> Self {
        self.override_highlight_color = Some(color);
        self
    }

    /// Overrides the shadow color with the given color. Shadows are the parts of the model that are not facing the light source.
    /// This overrides both the [time of day](WindWakerShaderBuilder::time_of_day) and [weather](WindWakerShaderBuilder::weather) settings.
    #[must_use]
    pub const fn override_shadow_color(mut self, color: Color) -> Self {
        self.override_shadow_color = Some(color);
        self
    }

    /// Overrides the rim color with the given color. The rim is the edge of the model, which gets a slight specular highlight to make the model pop.
    /// This overrides both the [time of day](WindWakerShaderBuilder::time_of_day) and [weather](WindWakerShaderBuilder::weather) settings.
    #[must_use]
    pub const fn override_rim_color(mut self, color: Color) -> Self {
        self.override_rim_color = Some(color);
        self
    }

    /// Builds the [`WindWakerShader`] with the given settings. Note that after insertion, the shader will be moved into the [`ExtendedMaterial`] of the entity.
    pub fn build(self) -> WindWakerShader {
        let (highlight, shadow) = match (self.time_of_day, self.weather) {
            (TimeOfDay::Dusk, Weather::Sunny) => (rgb(0xA1, 0x9A, 0xA3), rgb(0x74, 0x66, 0x76)),
            (TimeOfDay::Dusk, Weather::Rainy) => (rgb(0x90, 0x88, 0x7A), rgb(0x74, 0x66, 0x76)),
            (TimeOfDay::Morning, Weather::Sunny) => (rgb(0xF0, 0xEA, 0xE3), rgb(0xBC, 0xB7, 0xCB)),
            (TimeOfDay::Morning, Weather::Rainy) => (rgb(0xB8, 0xBD, 0xB8), rgb(0x9A, 0xA4, 0x94)),
            (TimeOfDay::Day, Weather::Sunny) => (rgb(0xFF, 0xFF, 0xFF), rgb(0xA3, 0x98, 0x92)),
            (TimeOfDay::Day, Weather::Rainy) => (rgb(0xAD, 0xBB, 0xB7), rgb(0x8E, 0x97, 0x8D)),
            (TimeOfDay::Afternoon, Weather::Sunny) => {
                (rgb(0xD8, 0xC3, 0x7F), rgb(0xB0, 0x90, 0x70))
            }
            (TimeOfDay::Afternoon, Weather::Rainy) => {
                (rgb(0x99, 0x91, 0x87), rgb(0x88, 0x81, 0x77))
            }
            (TimeOfDay::Evening, Weather::Sunny) => (rgb(0x8D, 0x8C, 0x9A), rgb(0x7E, 0x78, 0x85)),
            (TimeOfDay::Evening, Weather::Rainy) => (rgb(0x8E, 0x87, 0x7D), rgb(0x7A, 0x73, 0x68)),
            (TimeOfDay::Night, Weather::Sunny) => (rgb(0x87, 0x9E, 0xB5), rgb(0x5D, 0x6E, 0x99)),
            (TimeOfDay::Night, Weather::Rainy) => (rgb(0x4B, 0x66, 0x90), rgb(0x4C, 0x59, 0x5A)),
        };
        let highlight_color = self.override_highlight_color.unwrap_or(highlight);
        let shadow_color = self.override_shadow_color.unwrap_or(shadow);
        let rim_color = self.override_rim_color.unwrap_or(Color::WHITE);
        WindWakerShader {
            mask: TEXTURE_HANDLE.clone(),
            highlight_color: highlight_color.into(),
            shadow_color: shadow_color.into(),
            rim_color: rim_color.into(),
        }
    }
}

/// The time of day used for the color palette in the [`WindWakerShaderBuilder`].
///
/// Note that this does not have to correspond to any actual time settings in your game.
/// Rather, think of this as "mood categories" that you can use to set the color palette.
#[derive(Debug, Clone, Copy, Default)]
#[allow(missing_docs)]
pub enum TimeOfDay {
    Dusk,
    Morning,
    #[default]
    Day,
    Afternoon,
    Evening,
    Night,
}

impl TimeOfDay {
    /// Returns the next time of day in the cycle.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Dusk => Self::Morning,
            Self::Morning => Self::Day,
            Self::Day => Self::Afternoon,
            Self::Afternoon => Self::Evening,
            Self::Evening => Self::Night,
            Self::Night => Self::Dusk,
        }
    }
}

/// The weather used for the color palette in the [`WindWakerShaderBuilder`].
///
/// Note that this does not have to correspond to any actual weather settings in your game.
/// Rather, think of this as "mood categories" that you can use to set the color palette.
#[derive(Debug, Clone, Copy, Default)]
#[allow(missing_docs)]
pub enum Weather {
    #[default]
    Sunny,
    Rainy,
}
