/// Demonstrates the flat (Sable-style) toon shader with game-wide automatic application
/// and fat black outlines via `bevy_mod_outline`.
///
/// Run with:
///   cargo run --example flat_scene
use bevy::anti_alias::fxaa::Fxaa;
use bevy::prelude::*;
use bevy_mod_outline::{AsyncWorldInheritOutline, AutoGenerateOutlineNormalsPlugin, OutlinePlugin, OutlineVolume};
use bevy_wind_waker_shader::prelude::*;

fn main() {
    App::new()
        // FlatShaderPlugin::global() automatically shades every StandardMaterial
        // mesh in the app, including all children of loaded scenes.
        .add_plugins((
            DefaultPlugins,
            FlatShaderPlugin::global(),
            OutlinePlugin::EXTRUDE_VERTEX,
            AutoGenerateOutlineNormalsPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_light)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let outline = OutlineVolume {
        visible: true,
        width: 4.0,
        colour: Color::BLACK,
    };

    // No per-entity shader component needed — FlatShaderPlugin::global() handles shading.
    // AsyncWorldInheritOutline propagates OutlineVolume to all scene children automatically.
    commands.spawn((
        WorldAssetRoot(asset_server.load("FlightHelmet/FlightHelmet.gltf#Scene0")),
        Transform {
            translation: Vec3::new(-1.5, -1.0, 0.0),
            scale: Vec3::splat(4.0),
            ..default()
        },
        outline.clone(),
        AsyncWorldInheritOutline::default(),
    ));
    commands.spawn((
        WorldAssetRoot(asset_server.load("Fox.glb#Scene0")),
        Transform {
            translation: Vec3::new(1.5, -1.0, 0.0),
            scale: Vec3::splat(0.03),
            ..default()
        }
        .looking_at(Vec3::new(2.0, -2.5, -5.0), Vec3::Y),
        outline,
        AsyncWorldInheritOutline::default(),
    ));

    // light
    commands.spawn(PointLight::default());

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Fxaa::default(),
    ));
}

fn rotate_light(mut q: Query<&mut Transform, With<PointLight>>, time: Res<Time>) {
    for mut t in q.iter_mut() {
        t.translation = Vec3::new(time.elapsed_secs().sin(), 0.5, time.elapsed_secs().cos()) * 0.5;
    }
}
