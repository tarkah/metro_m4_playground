#set shell := ["cmd.exe", "/c"]

check:
    cargo check --all-targets --features usb_serial,alphanum

example-serial:
    cargo build --example serial --features usb_serial
    gdb target/thumbv7em-none-eabihf/debug/examples/serial

example-alphanum:
    cargo build --example alpha --features usb_serial,alphanum --release
    gdb target/thumbv7em-none-eabihf/release/examples/alpha

example-rainbow:
    cargo build --example neopixel_rainbow --release
    gdb target/thumbv7em-none-eabihf/release/examples/neopixel_rainbow

jlink:
    DISPLAY=:0.0 JLinkGDBServer -if SWD -device atsamd51j19a