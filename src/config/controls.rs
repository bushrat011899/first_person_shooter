use bevy::prelude::{KeyCode, MouseButton};
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ControlBindings {
    pub forward: UserInput,
    pub backward: UserInput,
    pub left: UserInput,
    pub right: UserInput,
    pub jump: UserInput,
    pub crouch: UserInput,
    pub sprint: UserInput,
    pub ram: UserInput,
    pub pour: UserInput,
    pub load: UserInput,
    pub fire: UserInput,
    pub pointer_sensitivity: f32,
}

#[derive(Clone, Copy, Sequence, Debug)]
pub enum UserAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Jump,
    Crouch,
    Sprint,
    Ram,
    Pour,
    Load,
    Fire,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum UserInput {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

impl From<KeyCode> for UserInput {
    fn from(value: KeyCode) -> Self {
        Self::Keyboard(value)
    }
}

impl From<MouseButton> for UserInput {
    fn from(value: MouseButton) -> Self {
        Self::Mouse(value)
    }
}

impl Default for ControlBindings {
    fn default() -> Self {
        Self {
            forward: KeyCode::W.into(),
            backward: KeyCode::S.into(),
            left: KeyCode::A.into(),
            right: KeyCode::D.into(),
            jump: KeyCode::Space.into(),
            crouch: KeyCode::LControl.into(),
            sprint: KeyCode::LShift.into(),
            ram: KeyCode::R.into(),
            pour: KeyCode::F.into(),
            load: KeyCode::V.into(),
            fire: MouseButton::Left.into(),
            pointer_sensitivity: 0.5,
        }
    }
}

impl ControlBindings {
    /// Returns a `UserInput` bindings in the provided `UserAction`.
    pub fn input_for<'a>(&'a self, action: UserAction) -> &'a UserInput {
        match action {
            UserAction::MoveForward => &self.forward,
            UserAction::MoveBackward => &self.backward,
            UserAction::MoveLeft => &self.left,
            UserAction::MoveRight => &self.right,
            UserAction::Jump => &self.jump,
            UserAction::Crouch => &self.crouch,
            UserAction::Sprint => &self.sprint,
            UserAction::Ram => &self.ram,
            UserAction::Pour => &self.pour,
            UserAction::Load => &self.load,
            UserAction::Fire => &self.fire,
        }
    }
}
