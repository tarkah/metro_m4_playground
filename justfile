#set shell := ["cmd.exe", "/c"]

check:
    cargo check --features usb_serial,alphanum --examples --lib

debug-serial:
    cargo build --example serial --features usb_serial
    gdb target/thumbv7em-none-eabihf/debug/examples/serial

debug-alphanum:
    cargo build --example alpha --features usb_serial,alphanum
    gdb target/thumbv7em-none-eabihf/debug/examples/alpha

debug-rainbow:
    cargo build --example neopixel_rainbow --release
    gdb target/thumbv7em-none-eabihf/release/examples/neopixel_rainbow

debug-clock:
    cargo build --example clock --features usb_serial,alphanum
    gdb target/thumbv7em-none-eabihf/debug/examples/clock

flash-serial:
    cargo hf2 --example serial --features usb_serial --release

flash-alphanum:
    cargo hf2 --example alpha --features usb_serial,alphanum --release

flash-rainbow:
    cargo hf2 --example neopixel_rainbow --release

flash-clock:
    cargo hf2 --example clock --features usb_serial,alphanum --release

jlink:
    JLinkGDBServer -if SWD -device atsamd51j19a