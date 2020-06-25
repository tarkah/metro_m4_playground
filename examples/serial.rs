#![no_std]
#![no_main]

use metro_m4 as hal;
use metro_m4_ext as hal_ext;
use panic_halt as _;

use hal::entry;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::{clock::GenericClockController, delay::Delay};
use hal_ext::{serial_print, usb_serial};

#[entry]
fn main() -> ! {
    // General setup
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = hal::Pins::new(peripherals.PORT);

    // Setup delay
    let mut delay = Delay::new(core.SYST, &mut clocks);

    usb_serial::init(
        peripherals.USB,
        &mut clocks,
        &mut peripherals.MCLK,
        pins.usb_dm,
        pins.usb_dp,
        &mut pins.port,
    );

    loop {
        serial_print!("TEST\n");

        delay.delay_ms(500u16);
    }
}
