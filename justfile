#set shell := ["cmd.exe", "/c"]

check:
    cargo check --all-targets --features usb_serial,alphanum

example-serial:
    cargo build --example serial --features usb_serial

example-alphanum:
    cargo build --example alphanum --features usb_serial,alphanum

jlink:
    DISPLAY=:0.0 JLinkGDBServer -if SWD -device ATSAMD21G18