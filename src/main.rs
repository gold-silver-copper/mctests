use bevy::{
    animation::{AnimationTarget, AnimationTargetId},
    gltf::GltfAssetLabel,
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    platform::collections::HashSet,
    prelude::*,
};
use std::f32::consts::*;
const MASK_GROUP_PATHS: &[(&str, &str)] = &[
    // Head and hat (mask group 0)
    ("Body", "Head"),
    // Right arm upper (mask group 1)
    ("Body", "Right Arm Upper"),
    // Right arm lower (mask group 2)
    ("Body/Right Arm Upper", "Right Arm Lower"),
    // Left arm upper (mask group 3)
    ("Body", "Left Arm Upper"),
    // Left arm lower (mask group 4)
    ("Body/Left Arm Upper", "Left Arm Lower"),
    // Right leg upper (mask group 5)
    ("Body/Body Lower", "Right Leg Upper"),
    // Right leg lower (mask group 6)
    ("Body/Body Lower/Right Leg Upper", "Right Leg Lower"),
    // Left leg upper (mask group 7)
    ("Body/Body Lower", "Left Leg Upper"),
    // Left leg lower (mask group 8)
    ("Body/Body Lower/Left Leg Upper", "Left Leg Lower"),
    // Body (mask group 9)
    ("Body", "Body Upper"),
];
#[derive(Clone, Copy, Component)]
struct AnimationControl {
    // The ID of the mask group that this button controls.
    group_id: u32,
    label: AnimationLabel,
}

#[derive(Clone, Copy, Component, PartialEq, Debug)]
enum AnimationLabel {
    Walk = 0,
    Idle = 1,
    Lean = 2,
    Pistol = 3,
}

#[derive(Clone, Debug, Resource)]
struct AnimationNodes([AnimationNodeIndex; 4]);

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(Update, setup_animation_graph_once_loaded)
        .add_systems(Update, (apply_minecraft_skin))
        .run();
}

#[derive(Component)]
struct MinecraftModel;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Msaa::Off,
        Transform::from_xyz(0.7, 1.0, -4.0).looking_at(Vec3::new(0.0, 1.3, 0.0), Vec3::Y),
    ));

    // Spawn the light.
    commands.spawn((
        PointLight {
            intensity: 10_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-4.0, 8.0, 13.0),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .build(),
    ));

    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/splitik6.gltf"))),
        MinecraftModel,
    ));
}

fn apply_minecraft_skin(
    mut commands: Commands,
    model_query: Query<(Entity, &SceneRoot), With<MinecraftModel>>,
    children_query: Query<&Children>,
    mesh_query: Query<Entity, With<Mesh3d>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, _) in &model_query {
        // Load the texture
        let texture_handle: Handle<Image> = asset_server.load("models/splitik.png");

        // Create the material with the texture
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),

            ..default()
        });

        traverse_and_apply_skin(
            entity,
            &children_query,
            &mesh_query,
            &mut commands,
            material.clone(),
        );

        commands.entity(entity).remove::<MinecraftModel>();
    }
}

fn traverse_and_apply_skin(
    entity: Entity,
    children_query: &Query<&Children>,
    mesh_query: &Query<Entity, With<Mesh3d>>,
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
) {
    if mesh_query.contains(entity) {
        commands
            .entity(entity)
            .insert(MeshMaterial3d(material.clone()));
    }

    // Recursively check children
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            traverse_and_apply_skin(
                child,
                children_query,
                mesh_query,
                commands,
                material.clone(),
            );
        }
    }
}

// Builds up the animation graph, including the mask groups, and adds it to the
// entity with the `AnimationPlayer` that the glTF loader created.
fn setup_animation_graph_once_loaded(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    targets: Query<(Entity, &AnimationTarget)>,
) {
    for (entity, mut player) in &mut players {
        // Load the animation clip from the glTF file.
        let mut animation_graph = AnimationGraph::new();
        let blend_node = animation_graph.add_additive_blend(1.0, animation_graph.root);

        let animation_graph_nodes: [AnimationNodeIndex; 4] =
            std::array::from_fn(|animation_index| {
                let handle = asset_server.load(
                    GltfAssetLabel::Animation(animation_index).from_asset("models/splitik6.gltf"),
                );
                let mask = if animation_index == 0 { 0 } else { 0x3f };
                animation_graph.add_clip_with_mask(handle, mask, 1.0, blend_node)
            });

        // Create each mask group.
        let mut all_animation_target_ids = HashSet::new();
        for (mask_group_index, (mask_group_prefix, mask_group_suffix)) in
            MASK_GROUP_PATHS.iter().enumerate()
        {
            println!(
                "{:#?}",
                (mask_group_index, (mask_group_prefix, mask_group_suffix))
            );
            // Split up the prefix and suffix, and convert them into `Name`s.
            let prefix: Vec<_> = mask_group_prefix.split('/').map(Name::new).collect();
            let suffix: Vec<_> = mask_group_suffix.split('/').map(Name::new).collect();

            // Add each bone in the chain to the appropriate mask group.
            for chain_length in 0..=suffix.len() {
                let animation_target_id = AnimationTargetId::from_names(
                    prefix.iter().chain(suffix[0..chain_length].iter()),
                );
                animation_graph
                    .add_target_to_mask_group(animation_target_id, mask_group_index as u32);
                all_animation_target_ids.insert(animation_target_id);
            }
        }

        // We're doing constructing the animation graph. Add it as an asset.
        let animation_graph = animation_graphs.add(animation_graph);
        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animation_graph));

        // Remove animation targets that aren't in any of the mask groups. If we
        // don't do that, those bones will play all animations at once, which is
        // ugly.
        for (target_entity, target) in &targets {
            if !all_animation_target_ids.contains(&target.id) {
                commands.entity(target_entity).remove::<AnimationTarget>();
            }
        }

        // Play the animation.
        for animation_graph_node in animation_graph_nodes {
            player.play(animation_graph_node).repeat();
        }

        // Record the graph nodes.
        commands.insert_resource(AnimationNodes(animation_graph_nodes));
    }
}
