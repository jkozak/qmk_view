use crate::keyboard::{KeyDef, KeyType};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

pub struct QmkKeymapParser;

impl QmkKeymapParser {
    pub fn parse_keymap_file<P: AsRef<Path>>(path: P) -> Result<Vec<Vec<KeyDef>>, ParseError> {
        let content = fs::read_to_string(path)?;
        Self::parse_keymap(&content)
    }

    pub fn parse_keymap(content: &str) -> Result<Vec<Vec<KeyDef>>, ParseError> {
        let mut layers = Vec::new();

        // Find the keymaps array
        let keymaps_start = content.find("const uint16_t PROGMEM keymaps")
            .ok_or_else(|| ParseError::Parse("Could not find keymaps array".to_string()))?;

        let keymaps_section = &content[keymaps_start..];

        // Extract each layer
        let mut layer_num = 0;
        let mut search_pos = 0;

        while let Some(layer_start) = keymaps_section[search_pos..].find(&format!("[{}] = LAYOUT", layer_num)) {
            let layer_start = search_pos + layer_start;

            // Find the opening parenthesis
            if let Some(paren_start) = keymaps_section[layer_start..].find('(') {
                let paren_start = layer_start + paren_start + 1;

                // Find the matching closing parenthesis
                if let Some(paren_end) = Self::find_matching_paren(&keymaps_section[paren_start..]) {
                    let layer_content = &keymaps_section[paren_start..paren_start + paren_end];
                    let layer_keys = Self::parse_layer(layer_content, layer_num)?;
                    layers.push(layer_keys);

                    search_pos = paren_start + paren_end;
                    layer_num += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if layers.is_empty() {
            return Err(ParseError::Parse("No layers found".to_string()));
        }

        Ok(layers)
    }

    fn find_matching_paren(s: &str) -> Option<usize> {
        let mut depth = 1;
        for (i, ch) in s.chars().enumerate() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn parse_layer(content: &str, layer_num: usize) -> Result<Vec<KeyDef>, ParseError> {
        let mut keys = Vec::new();

        // Remove comments and split by commas
        let cleaned = Self::remove_comments(content);
        let tokens: Vec<&str> = cleaned
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        // For LAYOUT_split_3x6_3: keys are INTERLEAVED by row
        // Each row has: 6 left keys, then 6 right keys
        // Then 3 left thumb keys, then 3 right thumb keys
        // Total: 42 keys

        let mut key_index = 0;

        // Main rows (0-2): each row has 6 left + 6 right keys
        for row in 0..3 {
            // Left hand keys for this row
            for col in 0..6 {
                if key_index < tokens.len() {
                    let keycode = tokens[key_index];
                    keys.push(KeyDef {
                        row: row as u8,
                        col: col as u8,
                        is_left: true,
                        keycode: Self::simplify_keycode(keycode),
                        key_type: Self::classify_keycode(keycode),
                    });
                    key_index += 1;
                }
            }

            // Right hand keys for this row
            for col in 0..6 {
                if key_index < tokens.len() {
                    let keycode = tokens[key_index];
                    keys.push(KeyDef {
                        row: row as u8,
                        col: col as u8,
                        is_left: false,
                        keycode: Self::simplify_keycode(keycode),
                        key_type: Self::classify_keycode(keycode),
                    });
                    key_index += 1;
                }
            }
        }

        // Thumb row (row 3): 3 left + 3 right keys
        for col in 0..3 {
            if key_index < tokens.len() {
                let keycode = tokens[key_index];
                keys.push(KeyDef {
                    row: 3,
                    col: col as u8,
                    is_left: true,
                    keycode: Self::simplify_keycode(keycode),
                    key_type: Self::classify_keycode(keycode),
                });
                key_index += 1;
            }
        }

        for col in 0..3 {
            if key_index < tokens.len() {
                let keycode = tokens[key_index];
                keys.push(KeyDef {
                    row: 3,
                    col: col as u8,
                    is_left: false,
                    keycode: Self::simplify_keycode(keycode),
                    key_type: Self::classify_keycode(keycode),
                });
                key_index += 1;
            }
        }

        Ok(keys)
    }

    fn remove_comments(s: &str) -> String {
        let mut result = String::new();
        let mut in_comment = false;

        for line in s.lines() {
            if let Some(comment_start) = line.find("//") {
                result.push_str(&line[..comment_start]);
            } else {
                result.push_str(line);
            }
            result.push(' ');
        }

        result
    }

    fn simplify_keycode(keycode: &str) -> String {
        let kc = keycode.trim();

        // Handle special cases
        if kc == "XXXXXXX" || kc == "_______" {
            return kc.to_string();
        }

        // Remove KC_ prefix for display
        let display = if let Some(stripped) = kc.strip_prefix("KC_") {
            stripped
        } else if let Some(mt) = Self::parse_mod_tap(kc) {
            return mt;
        } else if let Some(mo) = Self::parse_momentary(kc) {
            return mo;
        } else {
            kc
        };

        // Convert common keycodes to readable names
        match display {
            "BSPC" => "⌫",
            "ENT" | "ENTER" => "↵",
            "SPC" | "SPACE" => "␣",
            "TAB" => "⇥",
            "ESC" => "Esc",
            "LGUI" | "RGUI" => "⌘",
            "LALT" | "RALT" => "Alt",
            "LCTL" | "RCTL" => "Ctrl",
            "LSFT" | "RSFT" => "Shift",
            "UP" => "↑",
            "DOWN" => "↓",
            "LEFT" => "←",
            "RIGHT" => "→",
            "COMM" => ",",
            "DOT" => ".",
            "SLSH" => "/",
            "SCLN" => ";",
            "QUOT" => "'",
            "GRV" => "`",
            "MINS" => "-",
            "EQL" => "=",
            "LBRC" => "[",
            "RBRC" => "]",
            "BSLS" => "\\",
            "NUHS" => "#",
            "EXLM" => "!",
            "AT" => "@",
            "HASH" => "#",
            "DLR" => "$",
            "PERC" => "%",
            "CIRC" => "^",
            "AMPR" => "&",
            "ASTR" => "*",
            "LPRN" => "(",
            "RPRN" => ")",
            "UNDS" => "_",
            "PLUS" => "+",
            "LCBR" => "{",
            "RCBR" => "}",
            "PIPE" => "|",
            "TILD" => "~",
            s if s.starts_with('F') && s.len() <= 3 => s,  // F1-F12
            s if s.len() == 1 => s,  // Single letters/numbers
            s => s,
        }.to_string()
    }

    fn parse_mod_tap(keycode: &str) -> Option<String> {
        // MT(MOD_LALT, KC_A) -> A/Alt
        if keycode.starts_with("MT(") {
            if let Some(close_paren) = keycode.rfind(')') {
                let inner = &keycode[3..close_paren];
                if let Some(comma) = inner.find(',') {
                    let mod_part = inner[..comma].trim();
                    let key_part = inner[comma+1..].trim();

                    let mod_str = match mod_part {
                        "MOD_LALT" | "MOD_RALT" => "Alt",
                        "MOD_LGUI" | "MOD_RGUI" => "Gui",
                        "MOD_LCTL" | "MOD_RCTL" => "Ctl",
                        "MOD_LSFT" | "MOD_RSFT" => "Sft",
                        _ => "Mod",
                    };

                    let key_str = Self::simplify_keycode(key_part);
                    return Some(format!("{}/{}", key_str, mod_str));
                }
            }
        }

        // Handle HM_ macros (home row mods)
        if let Some(stripped) = keycode.strip_prefix("HM_") {
            // We know from the keymap that HM_A is A/Alt, etc.
            return Some(format!("{}/Mod", stripped));
        }

        None
    }

    fn parse_momentary(keycode: &str) -> Option<String> {
        // MO(1) -> ↓1
        if keycode.starts_with("MO(") {
            if let Some(close_paren) = keycode.find(')') {
                let layer = &keycode[3..close_paren];
                return Some(format!("↓{}", layer));
            }
        }
        None
    }

    fn classify_keycode(keycode: &str) -> KeyType {
        let kc = keycode.trim();

        if kc == "XXXXXXX" {
            return KeyType::None;
        }

        if kc == "_______" {
            return KeyType::Transparent;
        }

        // Mod-tap keys
        if kc.starts_with("MT(") || kc.starts_with("HM_") {
            return KeyType::Modifier;
        }

        // Layer keys
        if kc.starts_with("MO(") || kc.starts_with("LT(") || kc.starts_with("TG(") {
            return KeyType::Function;
        }

        // Check the keycode itself
        if kc.starts_with("KC_") {
            let base = &kc[3..];

            // Function keys
            if base.starts_with('F') && base.len() <= 3 {
                return KeyType::Function;
            }

            // Navigation
            if matches!(base, "UP" | "DOWN" | "LEFT" | "RIGHT" | "HOME" | "END" | "PGUP" | "PGDN") {
                return KeyType::Navigation;
            }

            // Modifiers
            if matches!(base, "LSFT" | "RSFT" | "LCTL" | "RCTL" | "LALT" | "RALT" | "LGUI" | "RGUI") {
                return KeyType::Modifier;
            }

            // Numbers
            if base.len() == 1 && base.chars().next().unwrap().is_ascii_digit() {
                return KeyType::Number;
            }

            // Letters
            if base.len() == 1 && base.chars().next().unwrap().is_ascii_alphabetic() {
                return KeyType::Letter;
            }

            // Symbols
            if matches!(base, "EXLM" | "AT" | "HASH" | "DLR" | "PERC" | "CIRC" | "AMPR" | "ASTR"
                        | "LPRN" | "RPRN" | "MINS" | "UNDS" | "EQL" | "PLUS" | "LBRC" | "RBRC"
                        | "LCBR" | "RCBR" | "BSLS" | "PIPE" | "SCLN" | "COMM" | "DOT" | "SLSH"
                        | "GRV" | "TILD" | "QUOT") {
                return KeyType::Symbol;
            }

            // Special navigation/editing
            if matches!(base, "BSPC" | "ENT" | "SPC" | "TAB" | "ESC" | "DEL") {
                return KeyType::Navigation;
            }
        }

        // UK keycodes and other custom
        if kc.starts_with("UK_") || kc.starts_with("S(") {
            return KeyType::Symbol;
        }

        // QK codes
        if kc.starts_with("QK_") || kc.starts_with("RM_") {
            return KeyType::Function;
        }

        KeyType::Symbol
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_keycode() {
        assert_eq!(QmkKeymapParser::simplify_keycode("KC_A"), "A");
        assert_eq!(QmkKeymapParser::simplify_keycode("KC_BSPC"), "⌫");
        assert_eq!(QmkKeymapParser::simplify_keycode("KC_ENT"), "↵");
        assert_eq!(QmkKeymapParser::simplify_keycode("XXXXXXX"), "XXXXXXX");
    }

    #[test]
    fn test_classify_keycode() {
        assert_eq!(QmkKeymapParser::classify_keycode("KC_A"), KeyType::Letter);
        assert_eq!(QmkKeymapParser::classify_keycode("KC_1"), KeyType::Number);
        assert_eq!(QmkKeymapParser::classify_keycode("KC_EXLM"), KeyType::Symbol);
        assert_eq!(QmkKeymapParser::classify_keycode("KC_F1"), KeyType::Function);
        assert_eq!(QmkKeymapParser::classify_keycode("XXXXXXX"), KeyType::None);
        assert_eq!(QmkKeymapParser::classify_keycode("_______"), KeyType::Transparent);
    }

    #[test]
    fn test_parse_mod_tap() {
        let result = QmkKeymapParser::parse_mod_tap("MT(MOD_LALT, KC_A)");
        assert!(result.is_some());
        assert!(result.unwrap().contains("A"));
    }
}
