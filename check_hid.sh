#!/usr/bin/env bash

echo "=== HID Devices ==="
echo ""

for device in /sys/class/hidraw/hidraw*/device; do
  name=$(cat $device/uevent 2>/dev/null | grep HID_NAME | cut -d= -f2)
  id=$(cat $device/uevent 2>/dev/null | grep HID_ID | cut -d= -f2)
  dev=$(basename $(dirname $device))

  if [ ! -z "$name" ]; then
    echo "$dev: $name"
    echo "  ID: $id"

    # Check if it's a QMK keyboard (VID 0xFEED)
    if echo "$id" | grep -q "FEED"; then
      echo "  *** QMK KEYBOARD FOUND ***"
    fi
    echo ""
  fi
done
