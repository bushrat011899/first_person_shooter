use bevy::{input::mouse::MouseMotion, prelude::*};
use std::f32::consts::*;

use super::FpsControllerInput;

/// Component describing Keyboard and Mouse FPS controls.
#[derive(Component)]
pub struct KeyboardAndMouseInputBindings {
    pub enabled: bool,
    pub sensitivity: f32,
    pub forward: KeyCode,
    pub back: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub up: KeyCode,
    pub down: KeyCode,
    pub sprint: KeyCode,
    pub jump: KeyCode,
    pub fly: KeyCode,
    pub crouch: KeyCode,
    pub free_look: KeyCode,
}

/// Defaults chosen based on popular games such as CSGO.
impl Default for KeyboardAndMouseInputBindings {
    fn default() -> Self {
        Self {
            enabled: true,
            sensitivity: 0.001,
            forward: KeyCode::W,
            back: KeyCode::S,
            left: KeyCode::A,
            right: KeyCode::D,
            up: KeyCode::Q,
            down: KeyCode::E,
            sprint: KeyCode::LShift,
            jump: KeyCode::Space,
            fly: KeyCode::F,
            crouch: KeyCode::LControl,
            free_look: KeyCode::LAlt,
        }
    }
}

/// Minimum angle value in radians.
///
/// TODO: Number appears arbitrary, should either be simplified or explained.
const ANGLE_EPSILON: f32 = 0.001953125;

/// A system which maps user inputs into an `AbstractInput` using a `KeyboardAndMouseInputBindings`.
pub fn keyboard_and_mouse_input(
    key_input: Res<Input<KeyCode>>,
    mut mouse_events: EventReader<MouseMotion>,
    mut query: Query<(&KeyboardAndMouseInputBindings, &mut FpsControllerInput)>,
) {
    let query = query.iter_mut().filter(|(bindings, _)| bindings.enabled);

    for (bindings, mut input) in query {
        // Map mouse motion to camera pitch and yaw
        let mouse_delta = mouse_events.iter().map(|event| event.delta).sum::<Vec2>();

        let mouse_delta = mouse_delta * bindings.sensitivity;

        input.pitch = (input.pitch - mouse_delta.y)
            .clamp(-FRAC_PI_2 + ANGLE_EPSILON, FRAC_PI_2 - ANGLE_EPSILON);
        input.yaw -= mouse_delta.x;
        if input.yaw.abs() > PI {
            input.yaw = input.yaw.rem_euclid(TAU);
        }

        // Map keyboard controls to player translation
        input.movement = Vec3::new(
            get_axis(&key_input, bindings.right, bindings.left),
            get_axis(&key_input, bindings.up, bindings.down),
            get_axis(&key_input, bindings.forward, bindings.back),
        );
        input.sprint = key_input.pressed(bindings.sprint);
        input.jump = key_input.pressed(bindings.jump);
        input.fly = key_input.just_pressed(bindings.fly);
        input.crouch = key_input.pressed(bindings.crouch);
        input.free_look = input
            .free_look
            .evolve(key_input.pressed(bindings.free_look));
    }
}

/// Maps a pair of keys onto a single floating point axis.
fn get_axis(key_input: &Res<Input<KeyCode>>, key_pos: KeyCode, key_neg: KeyCode) -> f32 {
    match (key_input.pressed(key_pos), key_input.pressed(key_neg)) {
        (true, false) => 1.0,
        (false, true) => -1.0,
        _ => 0.0,
    }
}
