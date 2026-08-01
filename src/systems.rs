use crate::WindWakerShader;
use bevy::asset::Assets;
use bevy::pbr::{ExtendedMaterial, MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Commands, Entity, Query, Res, ResMut, WorldInstanceSpawner, With, Without};
use bevy::world_serialization::{WorldInstance, WorldAssetRoot};

/// Source: https://github.com/bevyengine/bevy/discussions/8533#discussioncomment-5787519
pub(crate) fn customize_scene_materials(
    unloaded_instances: Query<(Entity, Option<&WorldInstance>, &WindWakerShader), With<WorldAssetRoot>>,
    handles: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    pbr_materials: Res<Assets<StandardMaterial>>,
    scene_manager: Res<WorldInstanceSpawner>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, WindWakerShader>>>,
    mut cmds: Commands,
) {
    for (entity, instance, shader) in unloaded_instances.iter() {
        if let Some(instance) = instance {
            if scene_manager.instance_is_ready(**instance) {
                cmds.entity(entity).remove::<WindWakerShader>();
            }
            // Iterate over all entities in scene (once it's loaded)
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
}

pub(crate) fn customize_standard_materials(
    with_material: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>, &WindWakerShader),
        Without<WorldAssetRoot>,
    >,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, WindWakerShader>>>,
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
