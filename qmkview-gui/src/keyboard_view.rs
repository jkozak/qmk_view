use egui::{Color32, FontId, Pos2, Rect, Rounding, Sense, Stroke, Vec2};
use qmkview_core::{KeyboardState, KeyType, Layout};
use std::sync::{Arc, Mutex};

pub struct KeyboardView {
    state: Arc<Mutex<KeyboardState>>,
    key_size: f32,
    key_gap: f32,
    split_gap: f32,
}

impl KeyboardView {
    pub fn new(state: Arc<Mutex<KeyboardState>>) -> Self {
        Self {
            state,
            key_size: 50.0,
            key_gap: 5.0,
            split_gap: 40.0,
        }
    }

    // Columnar stagger offsets for Corne keyboard (in multiples of key_size)
    fn column_stagger(&self, col: u8) -> f32 {
        match col {
            0 => -0.1,   // Pinky slightly up
            1 => 0.0,    // Ring base
            2 => 0.15,   // Middle slightly down
            3 => 0.05,   // Index slightly down
            4 => -0.05,  // Index inner slightly up
            5 => -0.2,   // Inner column up
            _ => 0.0,
        }
    }

    // Thumb key positioning (returns x, y offsets)
    fn thumb_key_offset(&self, col: u8) -> (f32, f32) {
        match col {
            0 => (0.5, 0.2),   // Outer thumb key (SUPER/RAISE)
            1 => (1.5, 0.0),   // Middle thumb key (LOWER/ALT)
            2 => (2.5, 0.1),   // Inner thumb key (SPC/ENT)
            _ => (0.0, 0.0),
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let state = self.state.lock().unwrap();
        let layout = state.layout();

        ui.vertical_centered(|ui| {
            ui.add_space(10.0);

            // Layer and modifiers on the same line to prevent jumping
            ui.horizontal(|ui| {
                ui.heading(format!("Layer: {}", state.layer_name()));

                let mods = state.modifiers();
                if !mods.is_empty() {
                    let mut mod_text = String::new();
                    if mods.shift { mod_text.push_str("Shift "); }
                    if mods.ctrl { mod_text.push_str("Ctrl "); }
                    if mods.alt { mod_text.push_str("Alt "); }
                    if mods.gui { mod_text.push_str("Gui "); }
                    ui.label(format!("  |  Modifiers: {}", mod_text.trim()));
                }
            });

            ui.add_space(10.0);

            self.draw_keyboard(ui, &state, layout);
        });
    }

    fn draw_keyboard(&self, ui: &mut egui::Ui, state: &KeyboardState, layout: &Layout) {
        let total_width = (self.key_size + self.key_gap) * (layout.cols() as f32) * 2.0 + self.split_gap;
        let total_height = (self.key_size + self.key_gap) * (layout.rows() as f32);

        let (response, painter) = ui.allocate_painter(
            Vec2::new(total_width, total_height),
            Sense::hover(),
        );

        let start_pos = response.rect.left_top();

        // Draw left hand
        for row in 0..layout.rows() {
            let cols = if row == 3 { 3 } else { layout.cols() }; // Thumb row has 3 keys
            for col in 0..cols {
                self.draw_key(&painter, start_pos, row as u8, col as u8, true, state);
            }
        }

        let right_start_x = start_pos.x
            + (self.key_size + self.key_gap) * (layout.cols() as f32)
            + self.split_gap;

        let right_start_pos = Pos2::new(right_start_x, start_pos.y);

        // Draw right hand
        for row in 0..layout.rows() {
            let cols = if row == 3 { 3 } else { layout.cols() }; // Thumb row has 3 keys
            for col in 0..cols {
                self.draw_key(&painter, right_start_pos, row as u8, col as u8, false, state);
            }
        }
    }

    fn draw_key(
        &self,
        painter: &egui::Painter,
        start_pos: Pos2,
        row: u8,
        col: u8,
        is_left: bool,
        state: &KeyboardState,
    ) {
        // Mirror column positions for right hand display
        let display_col = if is_left {
            col
        } else {
            let max_col = if row == 3 { 2 } else { 5 };
            max_col - col
        };

        let (x_offset, y_offset) = if row == 3 {
            // Thumb row: use special positioning
            let (thumb_x, thumb_y) = self.thumb_key_offset(display_col);
            (
                (self.key_size + self.key_gap) * thumb_x,
                (self.key_size + self.key_gap) * thumb_y,
            )
        } else {
            // Regular rows: apply columnar stagger
            (0.0, self.column_stagger(display_col) * self.key_size)
        };

        let x = start_pos.x + (display_col as f32) * (self.key_size + self.key_gap) + x_offset;
        let y = start_pos.y + (row as f32) * (self.key_size + self.key_gap) + y_offset;

        let rect = Rect::from_min_size(
            Pos2::new(x, y),
            Vec2::new(self.key_size, self.key_size),
        );

        let is_pressed = state.is_key_pressed(row, col, is_left);
        let key_def = state.get_key_at(row, display_col, is_left);

        let (fill_color, stroke_color, text_color) = if is_pressed {
            (
                Color32::from_rgba_unmultiplied(255, 255, 0, 200),
                Color32::from_rgba_unmultiplied(255, 255, 0, 255),
                Color32::BLACK,
            )
        } else if let Some(key) = key_def {
            let base_color = self.get_key_color(&key.key_type);
            (
                Color32::from_rgba_unmultiplied(
                    base_color.r(),
                    base_color.g(),
                    base_color.b(),
                    120,
                ),
                Color32::from_rgba_unmultiplied(
                    base_color.r(),
                    base_color.g(),
                    base_color.b(),
                    200,
                ),
                Color32::WHITE,
            )
        } else {
            (
                Color32::from_rgba_unmultiplied(50, 50, 50, 50),
                Color32::from_rgba_unmultiplied(80, 80, 80, 100),
                Color32::from_rgba_unmultiplied(150, 150, 150, 150),
            )
        };

        painter.rect(
            rect,
            Rounding::same(5.0),
            fill_color,
            Stroke::new(2.0, stroke_color),
        );

        if let Some(key) = key_def {
            let text_pos = rect.center();
            painter.text(
                text_pos,
                egui::Align2::CENTER_CENTER,
                &key.keycode,
                FontId::proportional(11.0),
                text_color,
            );
        }
    }

    fn get_key_color(&self, key_type: &KeyType) -> Color32 {
        match key_type {
            KeyType::Letter => Color32::from_rgb(100, 150, 255),
            KeyType::Number => Color32::from_rgb(100, 255, 150),
            KeyType::Symbol => Color32::from_rgb(200, 100, 255),
            KeyType::Modifier => Color32::from_rgb(255, 150, 100),
            KeyType::Function => Color32::from_rgb(255, 100, 100),
            KeyType::Navigation => Color32::from_rgb(100, 255, 255),
            KeyType::Transparent => Color32::from_rgb(100, 100, 100),
            KeyType::None => Color32::from_rgb(50, 50, 50),
        }
    }
}
