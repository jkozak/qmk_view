/*
 * QMKView Integration Example
 *
 * Add this code to your QMK keymap.c to enable Raw HID communication with QMKView.
 *
 * 1. Add `RAW_ENABLE = yes` to your rules.mk
 * 2. Add `#include "raw_hid.h"` at the top of your keymap.c
 * 3. Copy the functions below into your keymap.c
 * 4. If you already have layer_state_set_user, process_record_user, or matrix_scan_user,
 *    merge the code instead of replacing
 */

#include "raw_hid.h"

// QMKView Protocol Constants
#define QMKVIEW_MSG_LAYER_CHANGE    0x01
#define QMKVIEW_MSG_KEY_PRESS       0x02
#define QMKVIEW_MSG_KEY_RELEASE     0x03
#define QMKVIEW_MSG_MODIFIER_STATE  0x04
#define QMKVIEW_MSG_FULL_STATE      0x05

// State tracking variables
static uint8_t qmkview_prev_layer = 0;
static uint8_t qmkview_prev_mods = 0;
static uint32_t qmkview_last_full_state = 0;

// Send layer change to QMKView
static void qmkview_send_layer_change(uint8_t layer) {
    uint8_t data[32] = {0};
    data[0] = QMKVIEW_MSG_LAYER_CHANGE;
    data[1] = layer;
    raw_hid_send(data, 32);
}

// Send key press to QMKView
static void qmkview_send_key_press(uint8_t row, uint8_t col, bool is_left) {
    uint8_t data[32] = {0};
    data[0] = QMKVIEW_MSG_KEY_PRESS;
    data[1] = row;
    data[2] = col;
    data[3] = is_left ? 1 : 0;
    raw_hid_send(data, 32);
}

// Send key release to QMKView
static void qmkview_send_key_release(uint8_t row, uint8_t col, bool is_left) {
    uint8_t data[32] = {0};
    data[0] = QMKVIEW_MSG_KEY_RELEASE;
    data[1] = row;
    data[2] = col;
    data[3] = is_left ? 1 : 0;
    raw_hid_send(data, 32);
}

// Send modifier state to QMKView
static void qmkview_send_modifier_state(uint8_t mods) {
    uint8_t data[32] = {0};
    data[0] = QMKVIEW_MSG_MODIFIER_STATE;

    // Convert QMK mod mask to QMKView format
    uint8_t qmkview_mods = 0;
    if (mods & MOD_MASK_SHIFT) qmkview_mods |= 0x01;
    if (mods & MOD_MASK_CTRL)  qmkview_mods |= 0x02;
    if (mods & MOD_MASK_ALT)   qmkview_mods |= 0x04;
    if (mods & MOD_MASK_GUI)   qmkview_mods |= 0x08;

    data[1] = qmkview_mods;
    raw_hid_send(data, 32);
}

// Send full keyboard state to QMKView
static void qmkview_send_full_state(void) {
    uint8_t data[32] = {0};
    data[0] = QMKVIEW_MSG_FULL_STATE;
    data[1] = get_highest_layer(layer_state);

    uint8_t mods = get_mods() | get_oneshot_mods();
    uint8_t qmkview_mods = 0;
    if (mods & MOD_MASK_SHIFT) qmkview_mods |= 0x01;
    if (mods & MOD_MASK_CTRL)  qmkview_mods |= 0x02;
    if (mods & MOD_MASK_ALT)   qmkview_mods |= 0x04;
    if (mods & MOD_MASK_GUI)   qmkview_mods |= 0x08;

    data[2] = qmkview_mods;
    data[3] = 0; // Number of pressed keys - can be implemented if needed

    raw_hid_send(data, 32);
}

// Hook: Called when layer changes
layer_state_t layer_state_set_user(layer_state_t state) {
    uint8_t layer = get_highest_layer(state);
    if (layer != qmkview_prev_layer) {
        qmkview_send_layer_change(layer);
        qmkview_prev_layer = layer;
    }

    // If you have existing layer_state_set_user code, add it here

    return state;
}

// Hook: Called on every key press/release
bool process_record_user(uint16_t keycode, keyrecord_t *record) {
    uint8_t row = record->event.key.row;
    uint8_t col = record->event.key.col;

    // Determine which hand the key is on
    // For split keyboards: adjust MATRIX_ROWS based on your keyboard
    // For Corne/crkbd: rows 0-2 are left, 3-5 are right
    bool is_left = (row < MATRIX_ROWS / 2);

    // Normalize row for split keyboards
    uint8_t normalized_row = row % (MATRIX_ROWS / 2);

    if (record->event.pressed) {
        qmkview_send_key_press(normalized_row, col, is_left);
    } else {
        qmkview_send_key_release(normalized_row, col, is_left);
    }

    // If you have existing process_record_user code, add it here

    return true;
}

// Hook: Called continuously
void matrix_scan_user(void) {
    // Send periodic full state updates (every 100ms)
    // This prevents the HUD from getting out of sync
    if (timer_elapsed32(qmkview_last_full_state) > 100) {
        qmkview_send_full_state();
        qmkview_last_full_state = timer_read32();
    }

    // Send modifier state changes
    uint8_t current_mods = get_mods() | get_oneshot_mods();
    if (current_mods != qmkview_prev_mods) {
        qmkview_send_modifier_state(current_mods);
        qmkview_prev_mods = current_mods;
    }

    // If you have existing matrix_scan_user code, add it here
}

// Hook: Handle incoming HID data (optional, for future features)
void raw_hid_receive(uint8_t *data, uint8_t length) {
    // Reserved for future host -> keyboard communication
    // For example: requesting specific state, changing LED colors, etc.
}
