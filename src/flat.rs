//! Flat toon shader inspired by the art style of Sable.
//!
//! Unlike the Wind Waker shader, this shader preserves the mesh's original texture
//! colors and simply multiplies a shadow/highlight tint on top, producing a flat
//! cel-shaded look without replacing the color palette.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_wind_waker_shader::flat::*;
//!
//! fn main() {
//!     // Apply the flat shader to every StandardMaterial mesh in the app:
//!     App::new()
//!         .add_plugins((DefaultPlugins, FlatShaderPlugin::global()))
//!         .run();
//! }
//! ```
//!
//! Or apply per-entity:
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_wind_waker_shader::flat::*;
//!
//! fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     commands.spawn((
//!         SceneRoot(asset_server.load("models/Fox.glb")),
//!         FlatShaderBuilder::default().shadow_darkness(0.6).build(),
//!     ));
//! }
//! ```

use bevy::asset::{Asset, Assets, Handle, RenderAssetUsages, load_internal_asset, uuid_handle};
use bevy::ecs::observer::On;
use bevy::image::{CompressedImageFormats, Image, ImageSampler, ImageType};
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialPlugin, MeshMaterial3d, StandardMaterial,
};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::scene::{SceneInstance, SceneRoot};
use bevy::shader::ShaderRef;

use crate::components::TEXTURE_HANDLE;

pub(crate) const FLAT_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("8f4d1a2b-3c5e-4f6d-9a8b-7c6e5d4f3a2b");

/// Type alias for the extended material produced by [`FlatShader`].
pub type FlatExtendedMaterial = ExtendedMaterial<StandardMaterial, FlatShader>;

/// A flat toon shader that preserves the mesh's original texture colors.
///
/// Insert this component (via [`FlatShaderBuilder`]) on a mesh entity or a
/// [`SceneRoot`]. The plugin will replace the [`StandardMaterial`] with an
/// [`ExtendedMaterial`] and remove this component once done.
///
/// For game-wide application see [`FlatShaderPlugin::global`].
#[derive(Asset, AsBindGroup, PartialEq, Debug, Clone, Component, Reflect)]
#[reflect(PartialEq)]
pub struct FlatShader {
    #[texture(100)]
    #[sampler(101)]
    mask: Handle<Image>,
    /// Tint multiplied onto shadowed areas. Default: 50% gray.
    #[uniform(102)]
    pub shadow_tint: LinearRgba,
    /// Tint multiplied onto lit areas. Default: white (no change).
    #[uniform(103)]
    pub highlight_tint: LinearRgba,
    /// Number of discrete color levels per channel for palette quantization.
    ///
    /// `0.0` disables posterization (continuous color). `6.0` gives a 216-color palette
    /// reminiscent of Sable's art style. Higher values (e.g. `16.0`) are subtler.
    #[uniform(104)]
    pub color_levels: f32,
}

impl Default for FlatShader {
    fn default() -> Self {
        FlatShaderBuilder::default().build()
    }
}

impl MaterialExtension for FlatShader {
    fn fragment_shader() -> ShaderRef {
        FLAT_SHADER_HANDLE.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        FLAT_SHADER_HANDLE.into()
    }
}

/// Builds a [`FlatShader`] component.
///
/// # Example
///
/// ```
/// use bevy_wind_waker_shader::flat::FlatShaderBuilder;
///
/// let shader = FlatShaderBuilder::default()
///     .shadow_darkness(0.6)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct FlatShaderBuilder {
    shadow_tint: Color,
    highlight_tint: Color,
    color_levels: f32,
}

impl Default for FlatShaderBuilder {
    fn default() -> Self {
        Self {
            shadow_tint: Color::srgb(0.9, 0.9, 0.9),
            highlight_tint: Color::WHITE,
            color_levels: 6.0,
        }
    }
}

impl FlatShaderBuilder {
    /// Sets how dark shadow areas appear. `0.0` = black shadow, `1.0` = no shadow.
    ///
    /// This is a convenience wrapper around [`shadow_tint`](Self::shadow_tint) using a
    /// neutral gray at the given brightness.
    pub fn shadow_darkness(self, brightness: f32) -> Self {
        self.shadow_tint(Color::srgb(brightness, brightness, brightness))
    }

    /// Sets an explicit color to multiply onto shadowed areas.
    pub fn shadow_tint(mut self, color: Color) -> Self {
        self.shadow_tint = color;
        self
    }

    /// Sets the color multiplied onto lit areas. Defaults to white (original color unchanged).
    pub fn highlight_tint(mut self, color: Color) -> Self {
        self.highlight_tint = color;
        self
    }

    /// Sets the number of discrete color levels per RGB channel used for palette quantization.
    ///
    /// `0.0` disables posterization. `6.0` (default) gives a 216-color palette. Higher values
    /// are subtler; lower values give a more extreme poster-art look.
    pub fn color_levels(mut self, levels: f32) -> Self {
        self.color_levels = levels;
        self
    }

    /// Builds the [`FlatShader`].
    pub fn build(self) -> FlatShader {
        FlatShader {
            mask: TEXTURE_HANDLE.clone(),
            shadow_tint: self.shadow_tint.into(),
            highlight_tint: self.highlight_tint.into(),
            color_levels: self.color_levels,
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin for the flat toon shader.
///
/// Use [`FlatShaderPlugin::default()`] for manual per-entity usage, or
/// [`FlatShaderPlugin::global()`] / [`FlatShaderPlugin::global_with()`] to
/// automatically shade every [`StandardMaterial`] mesh in the app.
#[non_exhaustive]
#[derive(Debug, Default, Clone)]
pub struct FlatShaderPlugin {
    global: Option<FlatShader>,
}

impl FlatShaderPlugin {
    /// Apply the flat shader automatically to all [`StandardMaterial`] meshes, including
    /// those that are children of loaded scenes.
    ///
    /// Uses the default [`FlatShader`] (50% shadow darkness).
    pub fn global() -> Self {
        Self {
            global: Some(FlatShader::default()),
        }
    }

    /// Like [`global`](Self::global) but lets you supply a custom [`FlatShader`].
    pub fn global_with(shader: FlatShader) -> Self {
        Self {
            global: Some(shader),
        }
    }
}

#[derive(Resource, Clone)]
struct GlobalFlatShader(FlatShader);

impl Plugin for FlatShaderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            FLAT_SHADER_HANDLE,
            "assets/flat_shader.wgsl",
            Shader::from_wgsl
        );

        // Load ZAtoon texture. If WindWakerShaderPlugin is also registered, both plugins
        // insert the same image under the same UUID handle, which is idempotent.
        let img = Image::from_buffer(
            include_bytes!("assets/ZAtoon.png"),
            ImageType::Extension("png"),
            CompressedImageFormats::default(),
            false,
            ImageSampler::default(),
            RenderAssetUsages::RENDER_WORLD,
        )
        .expect("Failed to load internal image.");

        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .insert(TEXTURE_HANDLE.id(), img)
            .unwrap();

        app.add_plugins(MaterialPlugin::<FlatExtendedMaterial>::default())
            .add_systems(
                PreUpdate,
                (customize_scene_materials, customize_standard_materials),
            )
            .register_type::<FlatShader>();

        if let Some(shader) = &self.global {
            app.insert_resource(GlobalFlatShader(shader.clone()))
                .add_observer(apply_globally_on_material_add);
        }
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn customize_scene_materials(
    unloaded_instances: Query<(Entity, Option<&SceneInstance>, &FlatShader), With<SceneRoot>>,
    handles: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    pbr_materials: Res<Assets<StandardMaterial>>,
    scene_manager: Res<SceneSpawner>,
    mut materials: ResMut<Assets<FlatExtendedMaterial>>,
    mut cmds: Commands,
) {
    for (entity, instance, shader) in unloaded_instances.iter() {
        let Some(instance) = instance else { continue };
        if scene_manager.instance_is_ready(**instance) {
            cmds.entity(entity).remove::<FlatShader>();
        }
        let handles = handles.iter_many(scene_manager.iter_instance_entities(**instance));
        for (entity, material_handle) in handles {
            let Some(material) = pbr_materials.get(material_handle) else {
                continue;
            };
            let toon_material = materials.add(ExtendedMaterial {
                base: material.clone(),
                extension: shader.clone(),
            });
            cmds.entity(entity)
                .insert(MeshMaterial3d(toon_material))
                .remove::<MeshMaterial3d<StandardMaterial>>();
        }
    }
}

fn customize_standard_materials(
    with_material: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>, &FlatShader),
        Without<SceneRoot>,
    >,
    mut materials: ResMut<Assets<FlatExtendedMaterial>>,
    pbr_materials: Res<Assets<StandardMaterial>>,
    mut cmds: Commands,
) {
    for (entity, material_handle, shader) in with_material.iter() {
        let Some(material) = pbr_materials.get(material_handle) else {
            continue;
        };
        let toon_material = materials.add(ExtendedMaterial {
            base: material.clone(),
            extension: shader.clone(),
        });
        cmds.entity(entity)
            .insert(MeshMaterial3d(toon_material))
            .remove::<MeshMaterial3d<StandardMaterial>>();
    }
}

/// Observer registered when global mode is active. Fires whenever any entity
/// receives a `MeshMaterial3d<StandardMaterial>` and inserts the default
/// [`FlatShader`] so the swap systems pick it up next `PreUpdate`.
fn apply_globally_on_material_add(
    trigger: On<Add, MeshMaterial3d<StandardMaterial>>,
    without_shader: Query<(), Without<FlatShader>>,
    global: Res<GlobalFlatShader>,
    mut cmds: Commands,
) {
    if without_shader.contains(trigger.event_target()) {
        cmds.entity(trigger.event_target()).insert(global.0.clone());
    }
}
