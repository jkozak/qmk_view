// Quick HID device inspector
// Run with: rustc debug_hid.rs && ./debug_hid

use std::process::Command;

fn main() {
    println!("=== HID Devices ===\n");

    // List all HID devices
    let output = Command::new("sh")
        .arg("-c")
        .arg("for device in /sys/class/hidraw/hidraw*/device; do echo \"Device: $(basename $(dirname $device))\"; cat $device/uevent 2>/dev/null | grep -E 'HID_NAME|HID_ID'; echo; done")
        .output()
        .expect("Failed to execute command");

    println!("{}", String::from_utf8_lossy(&output.stdout));

    println!("\n=== Looking for QMK keyboards (VID 0xFEED) ===\n");

    let output2 = Command::new("sh")
        .arg("-c")
        .arg("for device in /sys/class/hidraw/hidraw*/device; do cat $device/uevent 2>/dev/null | grep -q 'HID_ID=.*:0000FEED' && echo \"Found QMK device: $(basename $(dirname $device))\" && cat $device/uevent; done")
        .output()
        .expect("Failed to execute command");

    println!("{}", String::from_utf8_lossy(&output2.stdout));
}
