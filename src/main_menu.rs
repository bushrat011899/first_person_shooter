use bevy::prelude::*;
use bevy_kira_audio::prelude::AudioReceiver;

use crate::AppState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_main_menu.in_schedule(OnEnter(AppState::MainMenu)))
            .add_systems((start_game, spin_main_menu_cube).in_set(OnUpdate(AppState::MainMenu)))
            .add_system(setdown_main_menu.in_schedule(OnExit(AppState::MainMenu)));
    }
}

/// Marks an entity as only relevant for the Main Menu state.
#[derive(Component)]
struct MainMenuEntity;

#[derive(Component)]
struct MainMenuCube;

/// Construct the Main Menu
fn setup_main_menu(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    // Spawn a cube
    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let cube_material_handle = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: cube_handle.clone(),
            material: cube_material_handle.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        MainMenuCube,
        MainMenuEntity,
    ));

    // Spawn a camera to stare at the cube
    commands.spawn((
        Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                fov: std::f32::consts::TAU / 5.0,
                ..default()
            }),
            transform: Transform::from_translation(Vec3 {
                x: 0.0,
                y: 2.0,
                z: 4.0,
            })
            .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        AudioReceiver,
        MainMenuEntity,
    ));

    // Create some prompt text
    commands.spawn((
        TextBundle::from_section(
            "Press Any Key to Start!",
            TextStyle {
                font: assets.load("fira_mono.ttf"),
                font_size: 48.0,
                color: Color::BLACK,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Percent(5.0),
                left: Val::Percent(5.0),
                ..default()
            },
            ..default()
        }),
        MainMenuEntity,
    ));
}

/// Animates a cube on the main menu
fn spin_main_menu_cube(time: Res<Time>, mut query: Query<&mut Transform, With<MainMenuCube>>) {
    let t = time.elapsed_seconds();

    for mut transform in query.iter_mut() {
        transform.translation = Vec3 {
            x: 0.0,
            y: 0.2 * (3. * t).sin(),
            z: 0.0,
        };
        transform.rotation = Quat::from_euler(EulerRot::YXZ, 3. * t, 4. * t, 5. * t);
    }
}

/// Clean-Up assets from the main-menu
fn setdown_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuEntity>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// React to player input to start the game.
fn start_game(key: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    let any_key_pressed = key.get_just_pressed().next().is_some();

    if any_key_pressed {
        next_state.set(AppState::InGame);
    }
}
