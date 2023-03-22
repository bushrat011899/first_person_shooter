use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::config::UserAction;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Pod, Zeroable, Default, Serialize, Deserialize, Debug)]
pub struct ButtonInput {
    bytes: [u8; 2],
}

impl ButtonInput {
    fn get_by_index(&self, index: usize) -> bool {
        let bits = u8::BITS as usize;

        if index > self.bytes.len() * bits {
            log::error!("Attempted to access index {index} in ButtonInput");
            return false;
        }

        let byte = index / bits;
        let bit = index % bits;
        let mask = 1 << bit;

        self.bytes[byte] & mask > 0
    }

    fn set_by_index(&mut self, index: usize, value: bool) {
        let bits = u8::BITS as usize;

        if index > self.bytes.len() * bits {
            log::error!("Attempted to access index {index} in ButtonInput");
            return;
        }

        let byte = index / bits;
        let bit = index % bits;
        let mask = 1 << bit;

        if value {
            self.bytes[byte] |= mask;
        } else {
            self.bytes[byte] &= !mask;
        }
    }

    fn index_of(action: UserAction) -> usize {
        match action {
            UserAction::MoveForward => 0,
            UserAction::MoveBackward => 1,
            UserAction::MoveLeft => 2,
            UserAction::MoveRight => 3,
            UserAction::Jump => 4,
            UserAction::Crouch => 5,
            UserAction::Sprint => 6,
            UserAction::Ram => 7,
            UserAction::Pour => 8,
            UserAction::Load => 9,
            UserAction::Fire => 10,
        }
    }

    /// Updates the state of the input associated with the provided `UserAction`.
    pub fn set(&mut self, action: UserAction, value: bool) {
        let index = Self::index_of(action);
        self.set_by_index(index, value);
    }

    /// Checks if the button associated with the provided `UserAction` is currently pressed.
    pub fn get(&self, action: UserAction) -> bool {
        let index = Self::index_of(action);
        self.get_by_index(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use enum_iterator::all;

    #[test]
    fn all_user_actions_can_be_stored_and_retreived() {
        for action in all::<UserAction>() {
            let mut input = ButtonInput::default();

            assert_eq!(
                input.get(action),
                false,
                "Action: {:?}; Byte Dump: {:?}",
                action,
                input.bytes
            );

            input.set(action, true);

            assert_eq!(
                input.get(action),
                true,
                "Action: {:?}; Byte Dump: {:?}",
                action,
                input.bytes
            );

            input.set(action, false);

            assert_eq!(
                input.get(action),
                false,
                "Action: {:?}; Byte Dump: {:?}",
                action,
                input.bytes
            );
        }
    }
}
