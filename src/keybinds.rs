use hashbrown::HashMap;
use winit::keyboard::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Forward,
    Backward,
    Left,
    Right,
    Jump,
    Crouch,
    Sprint,
}

pub struct Keybinds {
    inner: HashMap<KeyCode, KeyAction>
}

impl Keybinds {
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }

    #[inline]
    pub fn insert(&mut self, key_code: KeyCode, key_action: KeyAction) {
        self.inner.insert(key_code, key_action);
    }

    #[inline]
    pub fn get(&self, key_code: &KeyCode) -> Option<KeyAction> {
        self.inner.get(key_code).cloned()
    }
}

impl Default for Keybinds {
    fn default() -> Self {
        let mut keybinds = Self::new();
        keybinds.insert(KeyCode::KeyW, KeyAction::Forward);
        keybinds.insert(KeyCode::KeyS, KeyAction::Backward);
        keybinds.insert(KeyCode::KeyA, KeyAction::Left);
        keybinds.insert(KeyCode::KeyD, KeyAction::Right);
        keybinds.insert(KeyCode::Space, KeyAction::Jump);
        keybinds.insert(KeyCode::ShiftLeft, KeyAction::Crouch);
        keybinds.insert(KeyCode::ControlLeft, KeyAction::Sprint);
        keybinds
    }
}