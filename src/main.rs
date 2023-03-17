use std::time::Duration;

use bevy::{
    gltf::Gltf,
    gltf::{GltfMesh, GltfNode},
    math::Vec3Swizzles,
    prelude::*,
    window::{CursorGrabMode, WindowMode},
};

use bevy_kira_audio::prelude::*;
use bevy_rapier3d::prelude::*;

use bevy_fps_controller::controller::*;

mod player;

const SPAWN_POINT: Vec3 = Vec3::new(0.0, 1.0, 0.0);

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .insert_resource(ClearColor(Color::hex("D4F5F5").unwrap()))
        .insert_resource(RapierConfiguration::default())
        .insert_resource(SpacialAudio { max_distance: 25. })
        .insert_resource(SimplePerformance {
            frames: 0.0,
            delta_time: 0.0,
            frame_time: f32::INFINITY,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(AudioPlugin)
        // .add_plugin(RapierDebugRenderPlugin::default())
        // .add_plugin(FpsControllerPlugin)
        .add_systems((
            setup.on_startup(),
            manage_cursor,
            scene_colliders,
            display_text,
            respawn,
            fire_gun,
        ))
        .add_systems(
            (
                // Handle Player Inputs
                keyboard_and_mouse_input,
                choose_movement_mode,
                // Update the Controller
                map_input_orientation,
                map_input_movement,
                // Update the Camera
                map_camera_transform,
                player::head_bobbing,
            )
                .chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    let mut window = window.single_mut();
    window.title = String::from("Minimal FPS Controller Example");
    //window.mode = WindowMode::Fullscreen;

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 6000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 7.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    let player_entities = player::spawn_player(&mut commands, 0);

    commands.entity(player_entities.right_hand).insert((
        assets.load::<Scene, _>("L1A1_aussie.glb#Scene0"),
        AudioEmitter { instances: vec![] },
        Gun {
            fire_animation: assets.load("L1A1_aussie.glb#Animation0"),
            fire_sound: assets.load("gun_shot.ogg"),
        },
    ));

    commands.insert_resource(MainScene {
        handle: assets.load("playground.glb"),
        is_loaded: false,
    });

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
    mut perf: ResMut<SimplePerformance>,
    mut controller_query: Query<(&Transform, &Velocity)>,
    mut text_query: Query<&mut Text>,
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
