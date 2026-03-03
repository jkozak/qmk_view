use crate::hid::{Message, KeyPosition, Modifiers};
use super::{Layout, Layer, KeyDef};
use std::collections::HashSet;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct KeyboardState {
    layout: Layout,
    layers: Vec<Layer>,
    current_layer: u8,
    modifiers: Modifiers,
    pressed_keys: HashSet<(u8, u8, bool)>,
}

impl KeyboardState {
    pub fn new(layers: Vec<Layer>) -> Self {
        Self {
            layout: Layout::new(),
            layers,
            current_layer: 0,
            modifiers: Modifiers {
                shift: false,
                ctrl: false,
                alt: false,
                gui: false,
            },
            pressed_keys: HashSet::new(),
        }
    }

    pub fn apply_message(&mut self, msg: Message) {
        match msg {
            Message::LayerChange { layer } => {
                debug!("Layer changed to {}", layer);
                self.current_layer = layer;
            }
            Message::KeyPress(pos) => {
                debug!("Key pressed: row={}, col={}, is_left={}", pos.row, pos.col, pos.is_left);
                self.pressed_keys.insert((pos.row, pos.col, pos.is_left));
            }
            Message::KeyRelease(pos) => {
                debug!("Key released: row={}, col={}, is_left={}", pos.row, pos.col, pos.is_left);
                self.pressed_keys.remove(&(pos.row, pos.col, pos.is_left));
            }
            Message::ModifierState(mods) => {
                debug!("Modifiers changed: {:?}", mods);
                self.modifiers = mods;
            }
            Message::FullState {
                layer,
                modifiers,
                pressed_keys,
            } => {
                debug!("Full state update: layer={}, mods={:?}, keys={}", layer, modifiers, pressed_keys.len());
                self.current_layer = layer;
                self.modifiers = modifiers;
                self.pressed_keys.clear();
                for key in pressed_keys {
                    self.pressed_keys.insert((key.row, key.col, key.is_left));
                }
            }
        }
    }

    pub fn current_layer(&self) -> u8 {
        self.current_layer
    }

    pub fn modifiers(&self) -> &Modifiers {
        &self.modifiers
    }

    pub fn is_key_pressed(&self, row: u8, col: u8, is_left: bool) -> bool {
        self.pressed_keys.contains(&(row, col, is_left))
    }

    pub fn get_key_at(&self, row: u8, col: u8, is_left: bool) -> Option<&KeyDef> {
        if (self.current_layer as usize) < self.layers.len() {
            self.layers[self.current_layer as usize].get_key(row, col, is_left)
        } else {
            None
        }
    }

    pub fn get_current_layer_keys(&self) -> Vec<&KeyDef> {
        if (self.current_layer as usize) < self.layers.len() {
            self.layers[self.current_layer as usize].all_keys()
        } else {
            Vec::new()
        }
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn layer_name(&self) -> &str {
        if (self.current_layer as usize) < self.layers.len() {
            &self.layers[self.current_layer as usize].name
        } else {
            "Unknown"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyboard::{KeyType, KeyDef};

    fn create_test_layer() -> Layer {
        Layer::new("Test", vec![])
    }

    fn create_test_layer_with_keys() -> Layer {
        Layer::new("Test", vec![
            // Right hand row 0: col 0=Y (inner), col 5=Bksp (pinky)
            KeyDef { row: 0, col: 0, is_left: false, keycode: "Y".into(), key_type: KeyType::Letter },
            KeyDef { row: 0, col: 5, is_left: false, keycode: "Bksp".into(), key_type: KeyType::Navigation },
        ])
    }

    #[test]
    fn test_state_layer_change() {
        let mut state = KeyboardState::new(vec![create_test_layer(), create_test_layer()]);

        assert_eq!(state.current_layer(), 0);

        state.apply_message(Message::LayerChange { layer: 1 });
        assert_eq!(state.current_layer(), 1);
    }

    #[test]
    fn test_state_key_press() {
        let mut state = KeyboardState::new(vec![create_test_layer()]);

        assert!(!state.is_key_pressed(0, 0, true));

        state.apply_message(Message::KeyPress(KeyPosition {
            row: 0,
            col: 0,
            is_left: true,
        }));
        assert!(state.is_key_pressed(0, 0, true));

        state.apply_message(Message::KeyRelease(KeyPosition {
            row: 0,
            col: 0,
            is_left: true,
        }));
        assert!(!state.is_key_pressed(0, 0, true));
    }

    #[test]
    fn test_state_modifiers() {
        let mut state = KeyboardState::new(vec![create_test_layer()]);

        assert!(state.modifiers().is_empty());

        state.apply_message(Message::ModifierState(Modifiers {
            shift: true,
            ctrl: false,
            alt: false,
            gui: false,
        }));

        assert!(state.modifiers().shift);
    }

    #[test]
    fn test_right_hand_key_lookup() {
        // Layout stores: col 0=Y (inner), col 5=Bksp (pinky)
        // This test verifies direct lookup without mirroring (mirroring is done in view layer)
        let state = KeyboardState::new(vec![create_test_layer_with_keys()]);

        let key = state.get_key_at(0, 0, false);
        assert!(key.is_some());
        assert_eq!(key.unwrap().keycode, "Y");

        let key = state.get_key_at(0, 5, false);
        assert!(key.is_some());
        assert_eq!(key.unwrap().keycode, "Bksp");
    }
}
