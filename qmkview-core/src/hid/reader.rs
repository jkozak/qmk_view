use super::{Protocol, Message, ParseError, QMK_VENDOR_ID, CORNE_VENDOR_ID, RAW_HID_USAGE_PAGE, HID_PACKET_SIZE};
use crossbeam_channel::{Sender, Receiver, unbounded};
use hidapi::{HidApi, HidDevice};
use std::thread;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error, info, warn};

#[derive(Debug, Error)]
pub enum HidError {
    #[error("HID API error: {0}")]
    HidApi(#[from] hidapi::HidError),
    #[error("Device not found")]
    DeviceNotFound,
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("IO error: {0}")]
    Io(String),
}

pub struct HidReader {
    rx: Receiver<Result<Message, HidError>>,
    _handle: Option<thread::JoinHandle<()>>,
}

impl HidReader {
    pub fn new() -> Result<Self, HidError> {
        let (tx, rx) = unbounded();

        let handle = thread::spawn(move || {
            if let Err(e) = Self::read_loop(tx) {
                error!("HID reader thread error: {}", e);
            }
        });

        Ok(Self {
            rx,
            _handle: Some(handle),
        })
    }

    pub fn try_recv(&self) -> Option<Result<Message, HidError>> {
        self.rx.try_recv().ok()
    }

    pub fn recv(&self) -> Result<Message, HidError> {
        self.rx.recv().map_err(|_| HidError::Io("Channel closed".to_string()))?
    }

    fn find_device(api: &HidApi) -> Result<HidDevice, HidError> {
        info!("Searching for QMK keyboard (VID: 0x{:04X} or 0x{:04X}, usage_page: 0x{:04X})",
              QMK_VENDOR_ID, CORNE_VENDOR_ID, RAW_HID_USAGE_PAGE);

        for device_info in api.device_list() {
            debug!(
                "Found device: VID={:04X}, PID={:04X}, usage_page={:04X}, interface={}, product={}",
                device_info.vendor_id(),
                device_info.product_id(),
                device_info.usage_page(),
                device_info.interface_number(),
                device_info.product_string().unwrap_or("Unknown")
            );

            // Check for QMK keyboards by VID
            let is_qmk_vid = device_info.vendor_id() == QMK_VENDOR_ID
                || device_info.vendor_id() == CORNE_VENDOR_ID;

            // Also check for "Corne" in product name as fallback
            let is_corne = device_info.product_string()
                .map(|s| s.contains("Corne"))
                .unwrap_or(false);

            if (is_qmk_vid || is_corne) && device_info.usage_page() == RAW_HID_USAGE_PAGE {
                info!(
                    "Found QMK Raw HID device: {} (VID={:04X}, PID={:04X})",
                    device_info
                        .product_string()
                        .unwrap_or("Unknown"),
                    device_info.vendor_id(),
                    device_info.product_id()
                );

                return device_info.open_device(api).map_err(HidError::from);
            }
        }

        Err(HidError::DeviceNotFound)
    }

    fn read_loop(tx: Sender<Result<Message, HidError>>) -> Result<(), HidError> {
        let api = HidApi::new()?;
        let mut device: Option<HidDevice> = None;
        let mut buf = [0u8; HID_PACKET_SIZE];

        loop {
            if device.is_none() {
                match Self::find_device(&api) {
                    Ok(dev) => {
                        info!("Connected to QMK keyboard");
                        dev.set_blocking_mode(true).ok();
                        device = Some(dev);
                    }
                    Err(HidError::DeviceNotFound) => {
                        warn!("QMK keyboard not found, retrying in 2s...");
                        thread::sleep(Duration::from_secs(2));
                        continue;
                    }
                    Err(e) => {
                        error!("Error finding device: {}", e);
                        thread::sleep(Duration::from_secs(2));
                        continue;
                    }
                }
            }

            if let Some(ref dev) = device {
                match dev.read_timeout(&mut buf, 1000) {
                    Ok(size) if size > 0 => {
                        debug!("Received {} bytes from HID", size);
                        match Protocol::parse(&buf) {
                            Ok(msg) => {
                                debug!("Parsed message: {:?}", msg);
                                if tx.send(Ok(msg)).is_err() {
                                    info!("Receiver dropped, stopping HID reader");
                                    return Ok(());
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse HID message: {}", e);
                                if tx.send(Err(HidError::Parse(e))).is_err() {
                                    return Ok(());
                                }
                            }
                        }
                    }
                    Ok(_) => {
                        // Timeout, continue
                    }
                    Err(e) => {
                        error!("HID read error: {}, reconnecting...", e);
                        device = None;
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
        }
    }
}

pub struct MockHidSender {
    tx: Sender<Result<Message, HidError>>,
}

impl MockHidSender {
    pub fn new() -> (Self, Receiver<Result<Message, HidError>>) {
        let (tx, rx) = unbounded();
        (Self { tx }, rx)
    }

    pub fn send(&self, msg: Message) -> Result<(), HidError> {
        self.tx
            .send(Ok(msg))
            .map_err(|_| HidError::Io("Channel closed".to_string()))
    }

    pub fn into_reader(self, rx: Receiver<Result<Message, HidError>>) -> MockHidReader {
        MockHidReader { rx }
    }
}

pub struct MockHidReader {
    rx: Receiver<Result<Message, HidError>>,
}

impl MockHidReader {
    pub fn try_recv(&self) -> Option<Result<Message, HidError>> {
        self.rx.try_recv().ok()
    }

    pub fn recv(&self) -> Result<Message, HidError> {
        self.rx.recv().map_err(|_| HidError::Io("Channel closed".to_string()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hid::protocol::{KeyPosition, Modifiers};

    #[test]
    fn test_mock_sender() {
        let (sender, rx) = MockHidSender::new();

        sender.send(Message::LayerChange { layer: 1 }).unwrap();

        let reader = sender.into_reader(rx);
        let msg = reader.try_recv().unwrap().unwrap();
        assert_eq!(msg, Message::LayerChange { layer: 1 });
    }
}
