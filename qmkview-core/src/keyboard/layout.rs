use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    Letter,
    Number,
    Symbol,
    Modifier,
    Function,
    Navigation,
    Transparent,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDef {
    pub row: u8,
    pub col: u8,
    pub is_left: bool,
    pub keycode: String,
    pub key_type: KeyType,
}

#[derive(Debug, Clone)]
pub struct Layout {
    rows: usize,
    cols: usize,
}

impl Layout {
    pub const ROWS: usize = 4;  // 3 main rows + 1 thumb row
    pub const COLS: usize = 6;  // 6 columns in main rows, 3 in thumb row

    pub fn new() -> Self {
        Self {
            rows: Self::ROWS,
            cols: Self::COLS,
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn total_keys(&self) -> usize {
        // 3 rows of 6 keys + 1 row of 3 keys, per hand
        // (3 * 6 + 3) * 2 = 42
        21 * 2
    }

    pub fn is_valid_position(&self, row: u8, col: u8) -> bool {
        if (row as usize) >= self.rows {
            return false;
        }
        // Row 3 (thumb row) only has 3 columns
        if row == 3 {
            (col as usize) < 3
        } else {
            (col as usize) < self.cols
        }
    }

    pub fn key_index(&self, row: u8, col: u8, is_left: bool) -> Option<usize> {
        if !self.is_valid_position(row, col) {
            return None;
        }

        let base = if is_left { 0 } else { 21 };  // 21 keys per hand

        // Calculate local index within this hand
        let local_index = if row < 3 {
            row as usize * self.cols + col as usize
        } else {
            // Row 3 (thumb row): after 3 rows of 6 keys = 18 keys
            18 + col as usize
        };

        Some(base + local_index)
    }

    pub fn position_from_index(&self, index: usize) -> Option<(u8, u8, bool)> {
        if index >= self.total_keys() {
            return None;
        }

        let is_left = index < 21;
        let local_index = if is_left { index } else { index - 21 };

        let (row, col) = if local_index < 18 {
            // Rows 0-2: 6 keys each
            ((local_index / self.cols) as u8, (local_index % self.cols) as u8)
        } else {
            // Row 3: thumb keys
            (3, (local_index - 18) as u8)
        };

        Some((row, col, is_left))
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_dimensions() {
        let layout = Layout::new();
        assert_eq!(layout.rows(), 4);  // 3 main rows + 1 thumb row
        assert_eq!(layout.cols(), 6);  // Max columns (main rows)
        assert_eq!(layout.total_keys(), 42);  // (3*6 + 3) * 2
    }

    #[test]
    fn test_key_index() {
        let layout = Layout::new();

        // Left hand
        assert_eq!(layout.key_index(0, 0, true), Some(0));
        assert_eq!(layout.key_index(0, 5, true), Some(5));
        assert_eq!(layout.key_index(2, 5, true), Some(17));

        // Right hand starts at index 24 (3*6 + 3*1 + 3*0 = 21 keys on left + first 3 keys = 24)
        // But wait, left hand has rows 0-2 (6 cols each) + row 3 (3 cols) = 18 + 6 = 24 keys
        // Actually: row 0: 0-5, row 1: 6-11, row 2: 12-17, row 3: 18-20
        // Right hand starts at 21
        assert_eq!(layout.key_index(0, 0, false), Some(21));
        assert_eq!(layout.key_index(2, 5, false), Some(38));
    }

    #[test]
    fn test_position_roundtrip() {
        let layout = Layout::new();

        for i in 0..layout.total_keys() {
            let (row, col, is_left) = layout.position_from_index(i).unwrap();
            let index = layout.key_index(row, col, is_left).unwrap();
            assert_eq!(index, i);
        }
    }
}
