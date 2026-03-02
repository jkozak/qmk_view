mod protocol;
mod reader;

pub use protocol::{Message, MessageType, Protocol, ParseError, KeyPosition, Modifiers};
pub use reader::HidReader;

pub const QMK_VENDOR_ID: u16 = 0xFEED;
pub const CORNE_VENDOR_ID: u16 = 0x4653;  // foostan Corne
pub const RAW_HID_USAGE_PAGE: u16 = 0xFF60;
pub const HID_PACKET_SIZE: usize = 32;
