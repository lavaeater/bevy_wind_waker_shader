//! Pixelation shader — makes models appear rendered at a lower resolution.
//!
//! Quantizes texture UV coordinates to a coarse grid and posterizes output
//! colors, producing a pixel-art look on any 3D model.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_wind_waker_shader::pixelate::*;
//!
//! fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     commands.spawn((
//!         SceneRoot(asset_server.load("models/Fox.glb")),
//!         PixelShaderBuilder::default().build(),
//!     ));
//! }
//! ```

use bevy::asset::{Asset, Assets, Handle, load_internal_asset, uuid_handle};
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialPlugin, MeshMaterial3d, StandardMaterial,
};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::scene::{SceneInstance, SceneRoot};
use bevy::shader::ShaderRef;

pub(crate) const PIXELATE_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("3a1f7b2c-4d6e-5f8a-9b0c-1d2e3f4a5b6c");

/// Type alias for the extended material produced by [`PixelShader`].
pub type PixelExtendedMaterial = ExtendedMaterial<StandardMaterial, PixelShader>;

/// A pixelation shader that makes models appear rendered at a lower resolution.
///
/// Insert this component (via [`PixelShaderBuilder`]) on a mesh entity or a
/// [`SceneRoot`]. The plugin replaces the [`StandardMaterial`] with an
/// [`ExtendedMaterial`] and removes this component once done.
#[derive(Asset, AsBindGroup, PartialEq, Debug, Clone, Component, Reflect)]
#[reflect(PartialEq)]
pub struct PixelShader {
    /// Number of UV grid divisions per axis. Lower = larger blocks = more pixelated.
    /// `0.0` disables UV snapping. Default: `16.0`.
    #[uniform(100)]
    pub pixel_density: f32,
    /// Discrete color levels per RGB channel. Lower = fewer colors.
    /// `0.0` disables posterization. Default: `8.0`.
    #[uniform(101)]
    pub color_levels: f32,
}

impl Default for PixelShader {
    fn default() -> Self {
        PixelShaderBuilder::default().build()
    }
}

impl MaterialExtension for PixelShader {
    fn fragment_shader() -> ShaderRef {
        PIXELATE_SHADER_HANDLE.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        PIXELATE_SHADER_HANDLE.into()
    }
}

/// Builds a [`PixelShader`] component.
///
/// # Example
///
/// ```
/// use bevy_wind_waker_shader::pixelate::PixelShaderBuilder;
///
/// let shader = PixelShaderBuilder::default()
///     .pixel_density(8.0)
///     .color_levels(4.0)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct PixelShaderBuilder {
    pixel_density: f32,
    color_levels: f32,
}

impl Default for PixelShaderBuilder {
    fn default() -> Self {
        Self {
            pixel_density: 16.0,
            color_levels: 8.0,
        }
    }
}

impl PixelShaderBuilder {
    /// Sets the UV grid density. Lower values produce larger pixel blocks.
    ///
    /// `8.0` gives a very coarse look; `32.0` is subtle. `0.0` disables UV snapping.
    pub fn pixel_density(mut self, density: f32) -> Self {
        self.pixel_density = density;
        self
    }

    /// Sets the number of discrete color levels per RGB channel.
    ///
    /// `4.0` gives a harsh poster-art look; `16.0` is subtle. `0.0` disables posterization.
    pub fn color_levels(mut self, levels: f32) -> Self {
        self.color_levels = levels;
        self
    }

    /// Builds the [`PixelShader`].
    pub fn build(self) -> PixelShader {
        PixelShader {
            pixel_density: self.pixel_density,
            color_levels: self.color_levels,
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin for the pixelation shader.
///
/// Add this plugin once, then insert [`PixelShader`] (via [`PixelShaderBuilder`])
/// on any entity with a [`SceneRoot`] or a bare mesh to make it appear pixelated.
#[non_exhaustive]
#[derive(Debug, Default, Clone)]
pub struct PixelShaderPlugin;

impl Plugin for PixelShaderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            PIXELATE_SHADER_HANDLE,
            "assets/pixelate_shader.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<PixelExtendedMaterial>::default())
            .add_systems(
                PreUpdate,
                (customize_scene_materials, customize_standard_materials),
            )
            .register_type::<PixelShader>();
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn customize_scene_materials(
    unloaded_instances: Query<(Entity, Option<&SceneInstance>, &PixelShader), With<SceneRoot>>,
    handles: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    pbr_materials: Res<Assets<StandardMaterial>>,
    scene_manager: Res<SceneSpawner>,
    mut materials: ResMut<Assets<PixelExtendedMaterial>>,
    mut cmds: Commands,
) {
    for (entity, instance, shader) in unloaded_instances.iter() {
        let Some(instance) = instance else { continue };
        if scene_manager.instance_is_ready(**instance) {
            cmds.entity(entity).remove::<PixelShader>();
        }
        let handles = handles.iter_many(scene_manager.iter_instance_entities(**instance));
        for (entity, material_handle) in handles {
            let Some(material) = pbr_materials.get(material_handle) else {
                continue;
            };
            let pixel_material = materials.add(ExtendedMaterial {
                base: material.clone(),
                extension: shader.clone(),
            });
            cmds.entity(entity)
                .insert(MeshMaterial3d(pixel_material))
                .remove::<MeshMaterial3d<StandardMaterial>>();
        }
    }
}

fn customize_standard_materials(
    with_material: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>, &PixelShader),
        Without<SceneRoot>,
    >,
    mut materials: ResMut<Assets<PixelExtendedMaterial>>,
    pbr_materials: Res<Assets<StandardMaterial>>,
    mut cmds: Commands,
) {
    for (entity, material_handle, shader) in with_material.iter() {
        let Some(material) = pbr_materials.get(material_handle) else {
            continue;
        };
        let pixel_material = materials.add(ExtendedMaterial {
            base: material.clone(),
            extension: shader.clone(),
        });
        cmds.entity(entity)
            .insert(MeshMaterial3d(pixel_material))
            .remove::<MeshMaterial3d<StandardMaterial>>();
    }
}
