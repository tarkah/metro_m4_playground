#set shell := ["cmd.exe", "/c"]

check:
    cargo check --all-targets --features usb_serial,alphanum

debug-serial:
    cargo build --example serial --features usb_serial
    gdb target/thumbv7em-none-eabihf/debug/examples/serial

debug-alphanum:
    cargo build --example alpha --features usb_serial,alphanum
    gdb target/thumbv7em-none-eabihf/debug/examples/alpha

debug-rainbow:
    cargo build --example neopixel_rainbow --release
    gdb target/thumbv7em-none-eabihf/release/examples/neopixel_rainbow

flash-serial:
    cargo hf2 --example serial --features usb_serial --release

flash-alphanum:
    cargo hf2 --example alpha --features usb_serial,alphanum --release

flash-rainbow:
    cargo hf2 --example neopixel_rainbow --release

jlink:
    JLinkGDBServer -if SWD -device atsamd51j19a