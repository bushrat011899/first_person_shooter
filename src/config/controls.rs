use bevy::prelude::{KeyCode, MouseButton};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
        }
    }
}
