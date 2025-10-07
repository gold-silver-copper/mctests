use bevy::{
    animation::{AnimationTarget, AnimationTargetId},
    gltf::GltfAssetLabel,
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    platform::collections::HashSet,
    prelude::*,
};

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
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
