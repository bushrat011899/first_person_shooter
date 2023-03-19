use std::time::Duration;

use bevy::{
    gltf::Gltf,
    gltf::{GltfMesh, GltfNode},
    math::Vec3Swizzles,
    prelude::*,
    window::{CursorGrabMode, WindowMode},
};

use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_fps_controller::controller::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier3d::prelude::*;
use main_menu::MainMenuPlugin;

mod main_menu;
mod player;
mod firearm;

const SPAWN_POINT: Vec3 = Vec3::new(0.0, 1.0, 0.0);

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum AppState {
    #[default]
    MainMenu,
    InGame,
}

fn main() {
    App::new()
        .add_state::<AppState>()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .insert_resource(ClearColor(Color::hex("D4F5F5").unwrap()))
        .insert_resource(RapierConfiguration::default())
        .insert_resource(SpacialAudio { max_distance: 25. })
        .add_plugins(
            DefaultPlugins
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),
        )
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(AudioPlugin)
        .add_plugin(MainMenuPlugin)
        .add_system(setup_window.on_startup())
        .add_system(load_level.in_schedule(OnEnter(AppState::InGame)))
        .add_systems(
            (
                manage_cursor,
                scene_colliders,
                display_text,
                respawn,
                fire_gun,
                fire_gun_and_check_for_hits,
                keyboard_and_mouse_input,
                choose_movement_mode,
                map_input_orientation,
                map_input_movement,
                map_camera_transform,
                player::head_bobbing,
                player::right_hand_bobbing,
                player::left_hand_bobbing,
            )
                .chain()
                .in_set(OnUpdate(AppState::InGame)),
        )
        .run();
}

/// Setup the main window during application launch.
fn setup_window(mut window: Query<&mut Window>) {
    let mut window = window.single_mut();
    window.title = String::from("Muskrats at Dawn");
    //window.mode = WindowMode::Fullscreen;
}

fn load_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
    audio: Res<bevy_kira_audio::Audio>,
) {
    // Spawn a cube that plays music
    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let cube_material_handle = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: cube_handle.clone(),
            material: cube_material_handle.clone(),
            transform: Transform::from_xyz(5.0, 2.0, 1.0),
            ..default()
        },
        AudioEmitter {
            instances: vec![audio.play(assets.load("e1m1_cover.ogg")).looped().handle()],
        },
        Collider::cuboid(0.5, 0.5, 0.5),
    ));

    // Create a directional light for the environment
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 6000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 7.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Spawn the player
    let player_entities = player::spawn_player(&mut commands, 0);

    let player_handle = meshes.add(Mesh::from(shape::Capsule { radius: 0.5, rings: 8, depth: 1.0, latitudes: 8, longitudes: 8, uv_profile: default() }));
    let player_material_handle = materials.add(StandardMaterial {
        base_color: Color::rgb(0.3, 0.8, 0.3),
        ..default()
    });

    commands.entity(player_entities.torso).insert((
        player_handle,
        player_material_handle
    ));

    commands.entity(player_entities.right_hand).insert((
        assets.load::<Scene, _>("musket.glb#Scene0"),
        AudioEmitter { instances: vec![] },
        Gun {
            fire_animation: assets.load("musket.glb#Animation0"),
            fire_sound: assets.load("gun_shot.ogg"),
        },
    ));

    // Load the scene
    commands.insert_resource(MainScene {
        handle: assets.load("playground.glb"),
        is_loaded: false,
    });

    // Debug UI
    commands.spawn(
        TextBundle::from_section(
            "",
            TextStyle {
                font: assets.load("fira_mono.ttf"),
                font_size: 24.0,
                color: Color::BLACK,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
    );
}

fn respawn(mut query: Query<(&mut Transform, &mut Velocity)>) {
    for (mut transform, mut velocity) in &mut query {
        if transform.translation.y > -50.0 {
            continue;
        }

        velocity.linvel = Vec3::ZERO;
        transform.translation = SPAWN_POINT;
    }
}

#[derive(Resource)]
struct MainScene {
    handle: Handle<Gltf>,
    is_loaded: bool,
}

#[derive(Resource)]
struct SimplePerformance {
    frames: f32,
    delta_time: f32,
    frame_time: f32,
}

impl Default for SimplePerformance {
    fn default() -> Self {
        Self {
            frames: 0.0,
            delta_time: 0.0,
            frame_time: f32::INFINITY,
        }
    }
}

impl SimplePerformance {
    fn update(&mut self, delta_time: f32) {
        self.frames += 1.0;
        self.delta_time += delta_time;

        if self.delta_time > 1.0 && self.frames > 0.0 {
            self.frame_time = self.delta_time / self.frames;
            self.frames = 0.0;
            self.delta_time = 0.0;
        }
    }
}

#[derive(Component)]
struct Gun {
    fire_animation: Handle<AnimationClip>,
    fire_sound: Handle<bevy_kira_audio::AudioSource>,
}

fn fire_gun(
    input: Res<Input<MouseButton>>,
    mut gun_query: Query<(Entity, &Gun, &mut AudioEmitter), With<Gun>>,
    children: Query<&Children>,
    mut query: Query<&mut AnimationPlayer, Without<Gun>>,
    audio: Res<bevy_kira_audio::Audio>,
) {
    for (gun_entity, gun, mut audio_emitter) in gun_query.iter_mut() {
        for child in children.iter_descendants(gun_entity) {
            let Ok(mut player) = query.get_mut(child) else {
                continue;
            };

            if input.just_pressed(MouseButton::Left) {
                audio_emitter
                    .instances
                    .push(audio.play(gun.fire_sound.clone_weak()).handle());

                player
                    .set_speed(2.0)
                    .play_with_transition(
                        gun.fire_animation.clone_weak(),
                        Duration::from_millis(10),
                    )
                    .set_elapsed(0.0);
            }
        }
    }
}

fn fire_gun_and_check_for_hits(
    heads: Query<&GlobalTransform, With<player::Head>>,
    rapier_context: Res<RapierContext>,
    input: Res<Input<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !input.just_pressed(MouseButton::Left) {
        return;
    }

    for head in heads.iter() {
        let ray_dir = head.forward();
        let ray_pos = head.translation() + 2.0 * ray_dir;
        let max_toi = f32::INFINITY;
        let solid = true;
        let filter = QueryFilter::new();

        let Some((entity, toi)) = rapier_context.cast_ray(
            ray_pos, ray_dir, max_toi, solid, filter
        ) else {
            continue;
        };

        let hit_point = ray_pos + ray_dir * toi;
        println!("Entity {:?} hit at point {}", entity, hit_point);

        // Spawn sphere to represent the hit point
        let hit_handle = meshes.add(Mesh::from(shape::UVSphere {
            radius: 0.1,
            sectors: 4,
            stacks: 4,
        }));
        let hit_material_handle = materials.add(StandardMaterial {
            base_color: Color::rgb(0.1, 0.1, 0.1),
            ..default()
        });

        commands.spawn((
            PbrBundle {
                mesh: hit_handle.clone(),
                material: hit_material_handle.clone(),
                transform: Transform::from_translation(hit_point),
                ..default()
            },
            Collider::ball(0.1),
        ));
    }
}

fn scene_colliders(
    mut commands: Commands,
    mut main_scene: ResMut<MainScene>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<GltfMesh>>,
    gltf_node_assets: Res<Assets<GltfNode>>,
    mesh_assets: Res<Assets<Mesh>>,
) {
    if main_scene.is_loaded {
        return;
    }

    let Some(gltf) = gltf_assets.get(&main_scene.handle) else {
        return;
    };

    let scene = gltf.scenes.first().unwrap().clone();
    commands.spawn(SceneBundle { scene, ..default() });

    for node in &gltf.nodes {
        let node = gltf_node_assets.get(&node).unwrap();
        let Some(gltf_mesh) = node.mesh.clone() else {
            continue;
        };

        let gltf_mesh = gltf_mesh_assets.get(&gltf_mesh).unwrap();
        for mesh_primitive in &gltf_mesh.primitives {
            let mesh = mesh_assets.get(&mesh_primitive.mesh).unwrap();
            commands.spawn((
                Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap(),
                RigidBody::Fixed,
                TransformBundle::from_transform(node.transform),
            ));
        }
    }

    main_scene.is_loaded = true;
}

fn manage_cursor(
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut window_query: Query<&mut Window>,
    mut controller_query: Query<&mut KeyboardAndMouseInputBindings>,
) {
    let mut window = window_query.single_mut();
    if btn.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
        for mut controller in &mut controller_query {
            controller.enabled = true;
        }
    }
    if key.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
        for mut controller in &mut controller_query {
            controller.enabled = false;
        }
    }
}

fn display_text(
    time: Res<Time>,
    mut controller_query: Query<(&Transform, &Velocity)>,
    mut text_query: Query<&mut Text>,
    mut perf: Local<SimplePerformance>,
) {
    let dt = time.delta_seconds();

    perf.update(dt);

    let frame_time = perf.frame_time;
    let fps = 1.0 / frame_time;

    for (transform, velocity) in &mut controller_query {
        for mut text in &mut text_query {
            text.sections[0].value = format!(
                "vel: {:.2}, {:.2}, {:.2}\npos: {:.2}, {:.2}, {:.2}\nspd: {:.2}\nspf: {:.3}\nfps: {:3.0}",
                velocity.linvel.x,
                velocity.linvel.y,
                velocity.linvel.z,
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
                velocity.linvel.xz().length(),
                frame_time,
                fps
            );
        }
    }
}
