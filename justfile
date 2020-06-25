set shell := ["cmd.exe", "/c"]

example-serial:
    cargo build --example serial --features usb_serial

example-alphanum:
    cargo build --example alphanum --features usb_serial,alphanum