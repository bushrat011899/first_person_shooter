use config::UserAction;
use ggrs::InputStatus;
use input::LocalPlayerHandle;
use non_linear_time::{track_exact_time, ExactTime};
use player::{OwningPlayer, Head};
use simple_logger::SimpleLogger;

use bevy::{
    gltf::Gltf,
    gltf::{GltfMesh, GltfNode},
    math::Vec3Swizzles,
    prelude::*,
    window::{CursorGrabMode, PresentMode, WindowResolution},
};

use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_ggrs::{GGRSPlugin, PlayerInputs};
use bevy_hanabi::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier3d::prelude::*;

use firearm::{FirearmAction, FirearmActions, FirearmBundle, FirearmEvent, FirearmPlugin, Fired};
use main_menu::MainMenuPlugin;
use multiplayer::{GGRSConfig, MatchConfiguration};
use particles::{setup_smoke_particles, setup_sparks_particles, SmokeCloudEffect, SparksEffect};
use controller::*;

mod config;
mod firearm;
mod fog;
mod input;
mod main_menu;
mod multiplayer;
mod non_linear_time;
mod particles;
mod player;
mod controller;

const SPAWN_POINT: Vec3 = Vec3::new(0.0, 1.0, 0.0);

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}

pub fn log_user_inputs(
    inputs: Res<PlayerInputs<GGRSConfig>>,
    local_player: Res<LocalPlayerHandle>,
) {
    for (player, (input, status)) in inputs.iter().enumerate() {
        //info!("Player {} sent: {:?} with status {:?}", player, input, status);
        let tag = if local_player.0 == player { "Local" } else { "Remote" };
        if input.buttons.get(UserAction::Fire) {
            info!("[{tag}] Player {player} Fired!");
        }
        if input.buttons.get(UserAction::Jump) {
            info!("[{tag}] Player {player} Jumped!");
        }
    }
}

fn main() {
    // Load User Settings
    let config = config::Config::try_load().unwrap_or_default();
    config.try_save().expect("Must be able to save settings.");

    // Start Logging to Standard Out
    let mut logger = SimpleLogger::new().with_level(config.logging.level.into());

    for (module, level) in config.logging.overrides.iter() {
        logger = logger.with_module_level(module, (*level).into());
    }

    logger.init().expect("Unable to Initialise Logging System");

    // Create the Bevy Application
    let mut app = App::new();

    // Attach Multiplayer Controls to Bevy
    GGRSPlugin::<GGRSConfig>::new()
        // define frequency of rollback game logic update
        .with_update_frequency(config.matchmaking.tick_rate().into())
        // define system that returns inputs given a player handle, so GGRS can send the inputs around
        .with_input_system(input::capture_and_encode_user_input)
        // register types of components AND resources you want to be rolled back
        .register_rollback_component::<Transform>()
        .register_rollback_resource::<ExactTime>()
        // these systems will be executed as part of the advance frame update
        .with_rollback_schedule({
            let mut schedule = Schedule::default();

            schedule
                .add_systems((track_exact_time, log_user_inputs))
                .add_system(map_player_input_to_controller_input.in_set(FpsControllerSet::Input))
                .add_systems(
                    (
                        map_input_orientation,
                        map_input_movement,
                        map_camera_transform,
                    )
                        .chain()
                        .in_set(FpsControllerSet::Update),
                )
                .add_systems(
                    (
                        manage_cursor,
                        scene_colliders,
                        display_text,
                        respawn,
                        input_handler,
                        check_for_bullet_collisions,
                        activate_camera_of_local_player,
                    )
                        .in_set(OnUpdate(AppState::InGame)),
                );

            schedule
        })
        // make it happen in the bevy app
        .build(&mut app);

    // Configure the Rest of the Application
    app.add_state::<AppState>()
        .insert_resource(LocalPlayerHandle(0))
        .insert_resource(ExactTime {
            tick_rate: config.matchmaking.tick_rate().into(),
            tick: 0,
            seconds: 0,
        })
        .insert_resource(MatchConfiguration {
            room_id: "something_random".to_owned(),
            players: 2,
        })
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .insert_resource(ClearColor(Color::hex("D4F5F5").unwrap()))
        .insert_resource(RapierConfiguration::default())
        //.insert_resource(SpacialAudio { max_distance: 25. })
        .insert_resource::<Msaa>(config.graphics.msaa.into())
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Muskrats at Dawn".to_owned(),
                        mode: config.graphics.mode,
                        present_mode: if config.graphics.vsync {
                            PresentMode::AutoVsync
                        } else {
                            PresentMode::AutoNoVsync
                        },
                        resolution: WindowResolution::new(
                            config.graphics.width as f32,
                            config.graphics.height as f32,
                        ),
                        ..default()
                    }),
                    ..default()
                })
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),
        )
        .insert_resource(config)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(AudioPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(HanabiPlugin)
        .add_plugin(FirearmPlugin)
        .configure_set(
            FpsControllerSet::Input.before(FpsControllerSet::Update),
        )
        .add_systems(
            (
                multiplayer::start_matchbox_socket,
                setup_sparks_particles,
                setup_smoke_particles,
            )
                .on_startup(),
        )
        .add_systems(
            (
                multiplayer::watch_for_connected_peers,
                multiplayer::start_game_when_ready,
            )
                .in_set(OnUpdate(AppState::MainMenu)),
        )
        .add_systems((load_level, spawn_players, add_audio_listener_to_head_of_local_player).in_schedule(OnEnter(AppState::InGame)))
        
        .add_systems(
            (
                player::head_bobbing,
                player::right_hand_bobbing,
                player::left_hand_bobbing,
            )
                .in_set(OnUpdate(AppState::InGame))
                .after(FpsControllerSet::Update),
        )
        .add_systems(
            (fog::clear_fog_over_time, fog::increase_fog_after_shots)
                .in_set(OnUpdate(AppState::InGame)),
        )
        .run();
}

fn activate_camera_of_local_player(
    local_player: Res<LocalPlayerHandle>,
    mut query: Query<(&OwningPlayer, &mut Camera)>
) {
    for (OwningPlayer(player), mut camera) in query.iter_mut() {
        camera.is_active = *player == local_player.0;
    }
}

fn add_audio_listener_to_head_of_local_player(
    mut commands: Commands,
    local_player: Res<LocalPlayerHandle>,
    mut query: Query<(Entity, &OwningPlayer), (With<Head>, Without<bevy_kira_audio::prelude::AudioReceiver>)>
) {
    for (entity, OwningPlayer(player)) in query.iter_mut() {
        let Some(mut entity) = commands.get_entity(entity) else {
            continue;
        };

        if *player == local_player.0 {
            entity.insert(bevy_kira_audio::prelude::AudioReceiver);
        }
    }
}

fn spawn_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rip: ResMut<bevy_ggrs::RollbackIdProvider>,
    assets: Res<AssetServer>,
    match_settings: Res<MatchConfiguration>
) {
    for player_id in 0..match_settings.players {
        // Spawn the player
        let player_entities = player::spawn_player(&mut commands, player_id);

        let player_handle = meshes.add(Mesh::from(shape::Capsule {
            radius: 0.5,
            rings: 8,
            depth: 1.0,
            latitudes: 8,
            longitudes: 8,
            uv_profile: default(),
        }));
        let player_material_handle = materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.8, 0.3),
            ..default()
        });

        commands.entity(player_entities.head).insert((FogSettings {
            color: Color::rgba(0.1, 0.1, 0.1, 1.0),
            falloff: FogFalloff::Exponential { density: 0.1 },
            ..default()
        }, rip.next()));

        commands
            .entity(player_entities.torso)
            .insert((player_handle, player_material_handle));

        commands
            .entity(player_entities.right_hand)
            .insert((FirearmBundle {
                model: assets.load("musket.glb#Scene0"),
                actions: FirearmActions {
                    fire: FirearmAction {
                        animation: assets.load("musket.glb#Animation0"),
                        sound: assets.load("gun_shot.ogg"),
                        cooldown: 1.0,
                    },
                },
                audio_emitter: AudioEmitter { instances: vec![] },
            }, rip.next()));

        commands.entity(player_entities.feet).insert(rip.next());
        commands.entity(player_entities.left_hand).insert(rip.next());
        commands.entity(player_entities.torso).insert(rip.next());
    }
}

fn load_level(
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
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

fn input_handler(
    inputs: Res<PlayerInputs<GGRSConfig>>,
    hands: Query<(Entity, &OwningPlayer), (With<player::RightHand>, With<firearm::FirearmActions>)>,
    mut fire_events: EventWriter<firearm::FirearmEvent<firearm::Fire>>,
) {
    for (entity, OwningPlayer(player)) in hands.iter() {
        let Some((input, status)) = inputs.get(*player) else {
            continue;
        };

        if let InputStatus::Disconnected = status {
            continue;
        }

        if !input.buttons.get(UserAction::Fire) {
            continue;
        }

        fire_events.send(firearm::FirearmEvent {
            details: firearm::Fire,
            entity,
        });
    }
}

fn check_for_bullet_collisions(
    mut fired_events: EventReader<FirearmEvent<Fired>>,
    hands: Query<&Parent, With<player::RightHand>>,
    heads: Query<&GlobalTransform, With<player::Head>>,
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    impact_effect: Res<SparksEffect>,
    smoke_effect: Res<SmokeCloudEffect>,
) {
    for fired_event in fired_events.iter() {
        let Ok(parent) = hands.get(fired_event.entity) else {
            continue;
        };

        let Ok(head) = heads.get(parent.get()) else {
            continue;
        };

        let ray_dir = head.forward();
        let ray_pos = head.translation() + 2.0 * ray_dir;
        let max_toi = f32::INFINITY;
        let solid = true;
        let filter = QueryFilter::new();

        commands.spawn(ParticleEffectBundle {
            effect: ParticleEffect::new(smoke_effect.effect.clone_weak()),
            transform: Transform::from_translation(ray_pos),
            ..default()
        });

        let Some((entity, toi)) = rapier_context.cast_ray(
            ray_pos, ray_dir, max_toi, solid, filter
        ) else {
            continue;
        };

        let hit_point = ray_pos + ray_dir * toi;
        println!("Entity {:?} hit at point {}", entity, hit_point);

        // Spawn sphere to represent the hit point
        let hit_handle = meshes.add(Mesh::from(shape::UVSphere {
            radius: 0.01,
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
            Collider::ball(0.01),
            ParticleEffect::new(impact_effect.effect.clone_weak()),
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
        let node = gltf_node_assets.get(node).unwrap();
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
) {
    let mut window = window_query.single_mut();
    if btn.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }
    if key.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
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
