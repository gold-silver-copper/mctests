use bevy::{
    gltf::GltfAssetLabel,
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};
use std::f32::consts::*;

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_light_direction, apply_minecraft_skin))
        .run();
}

#[derive(Component)]
struct MinecraftModel;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Msaa::Off,
        Transform::from_xyz(0.7, 3.0, -2.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 250.0,
            ..default()
        },
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

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_secs() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}
