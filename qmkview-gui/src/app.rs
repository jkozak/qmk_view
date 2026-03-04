use crate::keyboard_view::KeyboardView;
use crate::overlay;
use eframe::egui;
use qmkview_core::{Config, HidReader, KeyboardState, Layer, QmkKeymapParser};
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{debug, error, info, warn};

pub struct QmkViewApp {
    state: Arc<Mutex<KeyboardState>>,
    keyboard_view: KeyboardView,
}

impl QmkViewApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, config: &Config) -> Self {
        let layers = Self::load_layers(config);
        let state = Arc::new(Mutex::new(KeyboardState::new(layers)));

        let state_clone = state.clone();
        let config_clone = config.clone();
        thread::spawn(move || {
            Self::hid_reader_thread(state_clone, config_clone);
        });

        Self {
            state: state.clone(),
            keyboard_view: KeyboardView::new(state),
        }
    }

    fn load_layers(config: &Config) -> Vec<Layer> {
        // Try to load from keymap file first
        if let Some(ref keymap_path) = config.keymap.keymap_path {
            info!("Loading keymap from: {}", keymap_path);
            match QmkKeymapParser::parse_keymap_file(keymap_path) {
                Ok(parsed_layers) => {
                    info!("Successfully parsed {} layers from keymap", parsed_layers.len());
                    return Layer::from_parsed_layers(parsed_layers, config.keymap.layer_names.clone());
                }
                Err(e) => {
                    warn!("Failed to parse keymap file: {}. Using default layers.", e);
                }
            }
        }

        // Fall back to default layers
        info!("Using default keymap");
        Layer::create_default_layers()
    }

    fn hid_reader_thread(state: Arc<Mutex<KeyboardState>>, config: Config) {
        info!("Starting HID reader thread");

        match HidReader::new() {
            Ok(reader) => {
                info!("HID reader initialized successfully");
                loop {
                    match reader.try_recv() {
                        Some(Ok(msg)) => {
                            debug!("Received HID message: {:?}", msg);
                            if let Ok(mut state) = state.lock() {
                                if matches!(msg, qmkview_core::Message::DeviceReconnected) {
                                    info!("Device reconnected, reloading keymap");
                                    let new_layers = Self::load_layers(&config);
                                    state.reload_layers(new_layers);
                                } else {
                                    state.apply_message(msg);
                                }
                            } else {
                                error!("Failed to lock state");
                            }
                        }
                        Some(Err(e)) => {
                            warn!("HID error: {}", e);
                        }
                        None => {
                            thread::sleep(std::time::Duration::from_millis(10));
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to initialize HID reader: {}", e);
                warn!("Running without HID input");
            }
        }
    }
}

impl eframe::App for QmkViewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                self.keyboard_view.draw(ui);
            });

        ctx.request_repaint();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting QMKView");

    let config = qmkview_core::Config::load()?;
    info!("Config loaded from {:?}", std::env::var("HOME").map(|h| format!("{}/.config/qmkview/config.json", h)));

    let native_options = overlay::create_overlay_options(&config.window);
    let config_clone = config.clone();

    eframe::run_native(
        "QMKView",
        native_options,
        Box::new(move |cc| Ok(Box::new(QmkViewApp::new(cc, &config_clone)))),
    )
    .map_err(|e| {
        error!("Failed to run eframe: {}", e);
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            as Box<dyn std::error::Error>
    })
}
