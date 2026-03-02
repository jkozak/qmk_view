with import <nixpkgs> {};

let
  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    # Rust toolchain
    rustc
    cargo
    rust-analyzer
    rustfmt
    clippy

    # HID libraries
    hidapi
    libusb1
    systemd

    # GUI dependencies (X11/Wayland)
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    libxkbcommon
    wayland

    # OpenGL for egui
    libGL

    # Build tools
    cmake
    gcc
  ];

in mkShell {
  name = "qmkview-dev";
  inherit buildInputs nativeBuildInputs;

  shellHook = ''
    export RUST_LOG=qmkview=info
    export RUST_BACKTRACE=1
    export LD_LIBRARY_PATH="${lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH"

    echo "QMKView development environment loaded"
    echo ""
    echo "HID device access:"
    echo "  Add udev rule to /etc/udev/rules.d/50-qmk.rules:"
    echo "  KERNEL==\"hidraw*\", SUBSYSTEM==\"hidraw\", MODE=\"0666\", TAG+=\"uaccess\""
    echo ""
    echo "Build: cargo build"
    echo "Run:   cargo run"
    echo ""
    echo "Logging: RUST_LOG=info (change to 'debug' for verbose output)"
  '';
}
