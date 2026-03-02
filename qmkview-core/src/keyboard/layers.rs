use super::{KeyDef, KeyType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    keys: HashMap<(u8, u8, bool), KeyDef>,
}

impl Layer {
    pub fn new(name: &str, keys: Vec<KeyDef>) -> Self {
        let mut key_map = HashMap::new();
        for key in keys {
            key_map.insert((key.row, key.col, key.is_left), key);
        }

        Self {
            name: name.to_string(),
            keys: key_map,
        }
    }

    pub fn from_parsed_layers(parsed_layers: Vec<Vec<KeyDef>>, layer_names: Vec<String>) -> Vec<Layer> {
        parsed_layers
            .into_iter()
            .enumerate()
            .map(|(i, keys)| {
                let name = layer_names.get(i)
                    .cloned()
                    .unwrap_or_else(|| format!("Layer {}", i));
                Layer::new(&name, keys)
            })
            .collect()
    }

    pub fn get_key(&self, row: u8, col: u8, is_left: bool) -> Option<&KeyDef> {
        self.keys.get(&(row, col, is_left))
    }

    pub fn all_keys(&self) -> Vec<&KeyDef> {
        self.keys.values().collect()
    }

    pub fn create_default_layers() -> Vec<Layer> {
        vec![
            Self::create_base_layer(),
            Self::create_lower_layer(),
            Self::create_raise_layer(),
            Self::create_adjust_layer(),
        ]
    }

    fn create_base_layer() -> Layer {
        let keys = vec![
            // Left hand - row 0
            KeyDef { row: 0, col: 0, is_left: true, keycode: "Tab".into(), key_type: KeyType::Navigation },
            KeyDef { row: 0, col: 1, is_left: true, keycode: "Q".into(), key_type: KeyType::Letter },
            KeyDef { row: 0, col: 2, is_left: true, keycode: "W".into(), key_type: KeyType::Letter },
            KeyDef { row: 0, col: 3, is_left: true, keycode: "E".into(), key_type: KeyType::Letter },
            KeyDef { row: 0, col: 4, is_left: true, keycode: "R".into(), key_type: KeyType::Letter },
            KeyDef { row: 0, col: 5, is_left: true, keycode: "T".into(), key_type: KeyType::Letter },
            // Left hand - row 1
            KeyDef { row: 1, col: 0, is_left: true, keycode: "Esc".into(), key_type: KeyType::Function },
            KeyDef { row: 1, col: 1, is_left: true, keycode: "A".into(), key_type: KeyType::Letter },
            KeyDef { row: 1, col: 2, is_left: true, keycode: "S".into(), key_type: KeyType::Letter },
            KeyDef { row: 1, col: 3, is_left: true, keycode: "D".into(), key_type: KeyType::Letter },
            KeyDef { row: 1, col: 4, is_left: true, keycode: "F".into(), key_type: KeyType::Letter },
            KeyDef { row: 1, col: 5, is_left: true, keycode: "G".into(), key_type: KeyType::Letter },
            // Left hand - row 2
            KeyDef { row: 2, col: 0, is_left: true, keycode: "Shift".into(), key_type: KeyType::Modifier },
            KeyDef { row: 2, col: 1, is_left: true, keycode: "Z".into(), key_type: KeyType::Letter },
            KeyDef { row: 2, col: 2, is_left: true, keycode: "X".into(), key_type: KeyType::Letter },
            KeyDef { row: 2, col: 3, is_left: true, keycode: "C".into(), key_type: KeyType::Letter },
            KeyDef { row: 2, col: 4, is_left: true, keycode: "V".into(), key_type: KeyType::Letter },
            KeyDef { row: 2, col: 5, is_left: true, keycode: "B".into(), key_type: KeyType::Letter },

            // Right hand - row 0
            KeyDef { row: 0, col: 0, is_left: false, keycode: "Y".into(), key_type: KeyType::Letter },
            KeyDef { row: 0, col: 1, is_left: false, keycode: "U".into(), key_type: KeyType::Letter },
            KeyDef { row: 0, col: 2, is_left: false, keycode: "I".into(), key_type: KeyType::Letter },
            KeyDef { row: 0, col: 3, is_left: false, keycode: "O".into(), key_type: KeyType::Letter },
            KeyDef { row: 0, col: 4, is_left: false, keycode: "P".into(), key_type: KeyType::Letter },
            KeyDef { row: 0, col: 5, is_left: false, keycode: "Bksp".into(), key_type: KeyType::Navigation },
            // Right hand - row 1
            KeyDef { row: 1, col: 0, is_left: false, keycode: "H".into(), key_type: KeyType::Letter },
            KeyDef { row: 1, col: 1, is_left: false, keycode: "J".into(), key_type: KeyType::Letter },
            KeyDef { row: 1, col: 2, is_left: false, keycode: "K".into(), key_type: KeyType::Letter },
            KeyDef { row: 1, col: 3, is_left: false, keycode: "L".into(), key_type: KeyType::Letter },
            KeyDef { row: 1, col: 4, is_left: false, keycode: ";".into(), key_type: KeyType::Symbol },
            KeyDef { row: 1, col: 5, is_left: false, keycode: "'".into(), key_type: KeyType::Symbol },
            // Right hand - row 2
            KeyDef { row: 2, col: 0, is_left: false, keycode: "N".into(), key_type: KeyType::Letter },
            KeyDef { row: 2, col: 1, is_left: false, keycode: "M".into(), key_type: KeyType::Letter },
            KeyDef { row: 2, col: 2, is_left: false, keycode: ",".into(), key_type: KeyType::Symbol },
            KeyDef { row: 2, col: 3, is_left: false, keycode: ".".into(), key_type: KeyType::Symbol },
            KeyDef { row: 2, col: 4, is_left: false, keycode: "/".into(), key_type: KeyType::Symbol },
            KeyDef { row: 2, col: 5, is_left: false, keycode: "Shift".into(), key_type: KeyType::Modifier },
        ];

        Layer::new("Base", keys)
    }

    fn create_lower_layer() -> Layer {
        let keys = vec![
            // Numbers and symbols layer
            // Left hand - row 0
            KeyDef { row: 0, col: 1, is_left: true, keycode: "1".into(), key_type: KeyType::Number },
            KeyDef { row: 0, col: 2, is_left: true, keycode: "2".into(), key_type: KeyType::Number },
            KeyDef { row: 0, col: 3, is_left: true, keycode: "3".into(), key_type: KeyType::Number },
            KeyDef { row: 0, col: 4, is_left: true, keycode: "4".into(), key_type: KeyType::Number },
            KeyDef { row: 0, col: 5, is_left: true, keycode: "5".into(), key_type: KeyType::Number },
            // Right hand - row 0
            KeyDef { row: 0, col: 0, is_left: false, keycode: "6".into(), key_type: KeyType::Number },
            KeyDef { row: 0, col: 1, is_left: false, keycode: "7".into(), key_type: KeyType::Number },
            KeyDef { row: 0, col: 2, is_left: false, keycode: "8".into(), key_type: KeyType::Number },
            KeyDef { row: 0, col: 3, is_left: false, keycode: "9".into(), key_type: KeyType::Number },
            KeyDef { row: 0, col: 4, is_left: false, keycode: "0".into(), key_type: KeyType::Number },
        ];

        Layer::new("Lower", keys)
    }

    fn create_raise_layer() -> Layer {
        let keys = vec![
            // Symbols layer
            KeyDef { row: 0, col: 1, is_left: true, keycode: "!".into(), key_type: KeyType::Symbol },
            KeyDef { row: 0, col: 2, is_left: true, keycode: "@".into(), key_type: KeyType::Symbol },
            KeyDef { row: 0, col: 3, is_left: true, keycode: "#".into(), key_type: KeyType::Symbol },
            KeyDef { row: 0, col: 4, is_left: true, keycode: "$".into(), key_type: KeyType::Symbol },
            KeyDef { row: 0, col: 5, is_left: true, keycode: "%".into(), key_type: KeyType::Symbol },
        ];

        Layer::new("Raise", keys)
    }

    fn create_adjust_layer() -> Layer {
        let keys = vec![
            // Function keys and special functions
            KeyDef { row: 0, col: 1, is_left: true, keycode: "F1".into(), key_type: KeyType::Function },
            KeyDef { row: 0, col: 2, is_left: true, keycode: "F2".into(), key_type: KeyType::Function },
            KeyDef { row: 0, col: 3, is_left: true, keycode: "F3".into(), key_type: KeyType::Function },
            KeyDef { row: 0, col: 4, is_left: true, keycode: "F4".into(), key_type: KeyType::Function },
        ];

        Layer::new("Adjust", keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_layers() {
        let layers = Layer::create_default_layers();
        assert_eq!(layers.len(), 4);
        assert_eq!(layers[0].name, "Base");
        assert_eq!(layers[1].name, "Lower");
        assert_eq!(layers[2].name, "Raise");
        assert_eq!(layers[3].name, "Adjust");
    }

    #[test]
    fn test_layer_get_key() {
        let layer = Layer::create_base_layer();
        let key = layer.get_key(0, 1, true);
        assert!(key.is_some());
        assert_eq!(key.unwrap().keycode, "Q");
    }
}
