# QMKView - Real-time QMK Keyboard HUD Overlay

A transparent overlay HUD that displays the current state of your QMK keyboard in real-time, showing available keys per layer with modifiers highlighted.

## Features

- Transparent, always-on-top overlay
- Real-time visualization of keyboard state via Raw HID
- 2D keyboard layout showing pressed keys
- Layer-aware key display
- Modifier state visualization
- Color-coded keys by function type
- Click-through window (doesn't interfere with your workflow)

## Prerequisites

- NixOS (for development environment)
- QMK-compatible keyboard (6x3 split keyboard)
- QMK firmware source code

## Setup

### 1. Development Environment

The project uses lorri + direnv for dependency management:

```bash
cd qmkview
direnv allow
```

This will automatically load the development shell with all dependencies.

### 2. HID Device Access

Create a udev rule to allow non-root access to HID devices:

```bash
sudo nano /etc/udev/rules.d/50-qmk.rules
```

Add this line:
```
KERNEL=="hidraw*", SUBSYSTEM=="hidraw", MODE="0666", TAG+="uaccess"
```

Reload udev rules:
```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### 3. Build the Application

```bash
cargo build --release
```

### 4. Configure QMK Firmware

You need to modify your QMK firmware to send keyboard state via Raw HID.

#### Step 1: Enable Raw HID

In your keymap's `rules.mk`, add:

```make
RAW_ENABLE = yes
```

#### Step 2: Modify `keymap.c`

Add the following to your `keymap.c`:

```c
#include "raw_hid.h"

// Protocol constants
#define MSG_LAYER_CHANGE    0x01
#define MSG_KEY_PRESS       0x02
#define MSG_KEY_RELEASE     0x03
#define MSG_MODIFIER_STATE  0x04
#define MSG_FULL_STATE      0x05

// State tracking
static uint8_t prev_layer = 0;
static uint8_t prev_mods = 0;
static uint32_t last_full_state_time = 0;

// Send layer change message
void send_layer_change(uint8_t layer) {
    uint8_t data[32] = {0};
    data[0] = MSG_LAYER_CHANGE;
    data[1] = layer;
    raw_hid_send(data, 32);
}

// Send key press message
void send_key_press(uint8_t row, uint8_t col, bool is_left) {
    uint8_t data[32] = {0};
    data[0] = MSG_KEY_PRESS;
    data[1] = row;
    data[2] = col;
    data[3] = is_left ? 1 : 0;
    raw_hid_send(data, 32);
}

// Send key release message
void send_key_release(uint8_t row, uint8_t col, bool is_left) {
    uint8_t data[32] = {0};
    data[0] = MSG_KEY_RELEASE;
    data[1] = row;
    data[2] = col;
    data[3] = is_left ? 1 : 0;
    raw_hid_send(data, 32);
}

// Send modifier state message
void send_modifier_state(uint8_t mods) {
    uint8_t data[32] = {0};
    data[0] = MSG_MODIFIER_STATE;
    data[1] = mods;
    raw_hid_send(data, 32);
}

// Send full state message
void send_full_state(void) {
    uint8_t data[32] = {0};
    data[0] = MSG_FULL_STATE;
    data[1] = get_highest_layer(layer_state);
    data[2] = get_mods();
    data[3] = 0; // Number of pressed keys (implement if needed)
    raw_hid_send(data, 32);
}

// Hook for layer changes
layer_state_t layer_state_set_user(layer_state_t state) {
    uint8_t layer = get_highest_layer(state);
    if (layer != prev_layer) {
        send_layer_change(layer);
        prev_layer = layer;
    }
    return state;
}

// Hook for key presses
bool process_record_user(uint16_t keycode, keyrecord_t *record) {
    uint8_t row = record->event.key.row;
    uint8_t col = record->event.key.col;

    // For split keyboards: determine which hand
    // Adjust this based on your keyboard's matrix
    bool is_left = (row < MATRIX_ROWS / 2);

    if (record->event.pressed) {
        send_key_press(row % (MATRIX_ROWS / 2), col, is_left);
    } else {
        send_key_release(row % (MATRIX_ROWS / 2), col, is_left);
    }

    return true;
}

// Periodic state updates
void matrix_scan_user(void) {
    // Send full state every 100ms to prevent desync
    if (timer_elapsed32(last_full_state_time) > 100) {
        send_full_state();
        last_full_state_time = timer_read32();
    }

    // Check modifier changes
    uint8_t current_mods = get_mods();
    if (current_mods != prev_mods) {
        send_modifier_state(current_mods);
        prev_mods = current_mods;
    }
}

// Handle incoming HID data (optional)
void raw_hid_receive(uint8_t *data, uint8_t length) {
    // Can be used for host -> keyboard communication in the future
}
```

#### Step 3: Flash Your Keyboard

```bash
cd qmk_firmware
make your_keyboard:your_keymap:flash
```

## Running

```bash
cargo run --release
```

The overlay will appear on your screen. It will automatically reconnect if the keyboard is disconnected.

## Configuration

QMKView automatically creates a configuration file at `~/.config/qmkview/config.json` on first run.

### Keymap Parsing

QMKView can automatically parse your QMK keymap.c file to display the correct keys for each layer.

**Configuration** (`~/.config/qmkview/config.json`):

```json
{
  "keymap": {
    "keymap_path": "/home/jk/qmk_firmware/keyboards/crkbd/keymaps/jk/keymap.c",
    "layer_names": ["Base", "Lower", "Raise", "Adjust"]
  }
}
```

Set `keymap_path` to the absolute path of your QMK keymap.c file. The parser will:
- Extract all layer definitions
- Parse QMK keycodes (KC_A, KC_BSPC, etc.)
- Handle mod-tap keys (MT, home row mods)
- Handle layer switching keys (MO, LT, etc.)
- Simplify keycodes for display (KC_BSPC → ⌫, KC_ENT → ↵)

If the parser fails or no path is specified, QMKView falls back to default layers.

See `config.example.json` for a complete configuration example.

### Window Position

Edit `qmkview-gui/src/overlay.rs` to change the default window position and size:

```rust
.with_inner_size([800.0, 300.0])
.with_position([100.0, 100.0])
```

### Colors

Colors are defined in `qmkview-gui/src/keyboard_view.rs` in the `get_key_color` method:

- Blue: Letters
- Green: Numbers
- Purple: Symbols
- Orange: Modifiers
- Red: Function keys
- Cyan: Navigation

## Protocol Reference

### Message Format

All messages are 32 bytes (standard HID packet size).

#### Layer Change (0x01)
```
[0] = 0x01
[1] = layer_number
[2-31] = padding
```

#### Key Press (0x02)
```
[0] = 0x02
[1] = row
[2] = col
[3] = is_left (0 or 1)
[4-31] = padding
```

#### Key Release (0x03)
```
[0] = 0x03
[1] = row
[2] = col
[3] = is_left (0 or 1)
[4-31] = padding
```

#### Modifier State (0x04)
```
[0] = 0x04
[1] = modifier_mask
    Shift = 0x01
    Ctrl  = 0x02
    Alt   = 0x04
    Gui   = 0x08
[2-31] = padding
```

#### Full State (0x05)
```
[0] = 0x05
[1] = layer_number
[2] = modifier_mask
[3] = pressed_keys_count
[4-31] = pressed_keys array (triplets: row, col, is_left)
```

## Troubleshooting

### Keyboard not detected

1. Check that Raw HID is enabled in your QMK firmware (`RAW_ENABLE = yes`)
2. Verify udev rules are installed and active
3. Check that the keyboard's vendor ID is `0xFEED` (standard QMK VID)
4. Run with logging: `RUST_LOG=debug cargo run`

### Overlay not transparent

This depends on your window manager. Ensure your compositor supports transparency.

### High CPU usage

The overlay repaints continuously for smooth animations. You can reduce the repaint rate in `app.rs` if needed.

## Development

### Running Tests

```bash
cargo test
```

### Code Structure

- `qmkview-core`: Core logic (HID protocol, keyboard state)
  - `hid/`: HID communication and protocol
  - `keyboard/`: Keyboard layout and state management
- `qmkview-gui`: GUI application (egui overlay)
  - `app.rs`: Main application
  - `overlay.rs`: Window configuration
  - `keyboard_view.rs`: 2D keyboard visualization

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
