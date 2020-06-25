#![no_std]
#![no_main]

use metro_m4 as hal;
use metro_m4_ext as hal_ext;
use panic_halt as _;

use hal::entry;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::{clock::GenericClockController, delay::Delay};
use hal_ext::{
    alphanum::{Display, MultiDisplay, DISP_I2C_ADDR},
    serial_print, usb_serial,
};

use ht16k33::HT16K33;

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

    // Initialize usb serial
    usb_serial::init(
        peripherals.USB,
        &mut clocks,
        &mut peripherals.MCLK,
        pins.usb_dm,
        pins.usb_dp,
        &mut pins.port,
    );

    // Setup AlphaNum Backpack Display (only using 1)
    let i2c = hal::i2c_master(
        &mut clocks,
        20.khz(),
        peripherals.SERCOM5,
        &mut peripherals.MCLK,
        pins.sda,
        pins.scl,
        &mut pins.port,
    );
    let mut display_drivers = [HT16K33::new(i2c, DISP_I2C_ADDR)];
    let mut display = MultiDisplay::new(&mut display_drivers);

    loop {
        serial_print!("Displaying text\n");

        display.marquee("Test", &mut delay, 250);

        delay.delay_ms(500u16);
    }
}
