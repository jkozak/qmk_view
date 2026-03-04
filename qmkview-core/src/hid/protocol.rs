use thiserror::Error;

pub const MSG_LAYER_CHANGE: u8 = 0x01;
pub const MSG_KEY_PRESS: u8 = 0x02;
pub const MSG_KEY_RELEASE: u8 = 0x03;
pub const MSG_MODIFIER_STATE: u8 = 0x04;
pub const MSG_FULL_STATE: u8 = 0x05;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    LayerChange,
    KeyPress,
    KeyRelease,
    ModifierState,
    FullState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPosition {
    pub row: u8,
    pub col: u8,
    pub is_left: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub gui: bool,
}

impl Modifiers {
    pub fn from_mask(mask: u8) -> Self {
        Self {
            shift: (mask & 0x01) != 0,
            ctrl: (mask & 0x02) != 0,
            alt: (mask & 0x04) != 0,
            gui: (mask & 0x08) != 0,
        }
    }

    pub fn to_mask(&self) -> u8 {
        let mut mask = 0u8;
        if self.shift { mask |= 0x01; }
        if self.ctrl { mask |= 0x02; }
        if self.alt { mask |= 0x04; }
        if self.gui { mask |= 0x08; }
        mask
    }

    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.gui
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    LayerChange { layer: u8 },
    KeyPress(KeyPosition),
    KeyRelease(KeyPosition),
    ModifierState(Modifiers),
    FullState {
        layer: u8,
        modifiers: Modifiers,
        pressed_keys: Vec<KeyPosition>,
    },
    DeviceReconnected,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid packet size: expected 32 bytes, got {0}")]
    InvalidSize(usize),
    #[error("Unknown message type: {0}")]
    UnknownMessageType(u8),
    #[error("Invalid data in message")]
    InvalidData,
}

pub struct Protocol;

impl Protocol {
    pub fn parse(data: &[u8]) -> Result<Message, ParseError> {
        if data.len() != 32 {
            return Err(ParseError::InvalidSize(data.len()));
        }

        match data[0] {
            MSG_LAYER_CHANGE => {
                Ok(Message::LayerChange { layer: data[1] })
            }
            MSG_KEY_PRESS => {
                Ok(Message::KeyPress(KeyPosition {
                    row: data[1],
                    col: data[2],
                    is_left: data[3] != 0,
                }))
            }
            MSG_KEY_RELEASE => {
                Ok(Message::KeyRelease(KeyPosition {
                    row: data[1],
                    col: data[2],
                    is_left: data[3] != 0,
                }))
            }
            MSG_MODIFIER_STATE => {
                Ok(Message::ModifierState(Modifiers::from_mask(data[1])))
            }
            MSG_FULL_STATE => {
                let layer = data[1];
                let modifiers = Modifiers::from_mask(data[2]);
                let key_count = data[3] as usize;

                if key_count > 10 {
                    return Err(ParseError::InvalidData);
                }

                let mut pressed_keys = Vec::with_capacity(key_count);
                for i in 0..key_count {
                    let offset = 4 + (i * 3);
                    if offset + 2 >= data.len() {
                        return Err(ParseError::InvalidData);
                    }
                    pressed_keys.push(KeyPosition {
                        row: data[offset],
                        col: data[offset + 1],
                        is_left: data[offset + 2] != 0,
                    });
                }

                Ok(Message::FullState {
                    layer,
                    modifiers,
                    pressed_keys,
                })
            }
            msg_type => Err(ParseError::UnknownMessageType(msg_type)),
        }
    }

    pub fn encode(msg: &Message) -> [u8; 32] {
        let mut data = [0u8; 32];

        match msg {
            Message::LayerChange { layer } => {
                data[0] = MSG_LAYER_CHANGE;
                data[1] = *layer;
            }
            Message::KeyPress(pos) => {
                data[0] = MSG_KEY_PRESS;
                data[1] = pos.row;
                data[2] = pos.col;
                data[3] = if pos.is_left { 1 } else { 0 };
            }
            Message::KeyRelease(pos) => {
                data[0] = MSG_KEY_RELEASE;
                data[1] = pos.row;
                data[2] = pos.col;
                data[3] = if pos.is_left { 1 } else { 0 };
            }
            Message::ModifierState(mods) => {
                data[0] = MSG_MODIFIER_STATE;
                data[1] = mods.to_mask();
            }
            Message::FullState {
                layer,
                modifiers,
                pressed_keys,
            } => {
                data[0] = MSG_FULL_STATE;
                data[1] = *layer;
                data[2] = modifiers.to_mask();
                data[3] = pressed_keys.len().min(10) as u8;

                for (i, key) in pressed_keys.iter().take(10).enumerate() {
                    let offset = 4 + (i * 3);
                    data[offset] = key.row;
                    data[offset + 1] = key.col;
                    data[offset + 2] = if key.is_left { 1 } else { 0 };
                }
            }
            Message::DeviceReconnected => {
                // Internal message, not sent over HID
            }
        }

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_layer_change() {
        let mut data = [0u8; 32];
        data[0] = MSG_LAYER_CHANGE;
        data[1] = 2;

        let msg = Protocol::parse(&data).unwrap();
        assert_eq!(msg, Message::LayerChange { layer: 2 });
    }

    #[test]
    fn test_parse_key_press() {
        let mut data = [0u8; 32];
        data[0] = MSG_KEY_PRESS;
        data[1] = 1;
        data[2] = 3;
        data[3] = 1;

        let msg = Protocol::parse(&data).unwrap();
        assert_eq!(
            msg,
            Message::KeyPress(KeyPosition {
                row: 1,
                col: 3,
                is_left: true
            })
        );
    }

    #[test]
    fn test_modifiers() {
        let mods = Modifiers {
            shift: true,
            ctrl: false,
            alt: true,
            gui: false,
        };

        let mask = mods.to_mask();
        assert_eq!(mask, 0x01 | 0x04);

        let parsed = Modifiers::from_mask(mask);
        assert_eq!(parsed, mods);
    }

    #[test]
    fn test_roundtrip() {
        let msg = Message::FullState {
            layer: 1,
            modifiers: Modifiers {
                shift: true,
                ctrl: false,
                alt: false,
                gui: false,
            },
            pressed_keys: vec![
                KeyPosition {
                    row: 0,
                    col: 0,
                    is_left: true,
                },
                KeyPosition {
                    row: 2,
                    col: 5,
                    is_left: false,
                },
            ],
        };

        let encoded = Protocol::encode(&msg);
        let decoded = Protocol::parse(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }
}
