#![no_std]
#![no_main]

use metro_m4 as hal;
use metro_m4_ext as hal_ext;

#[cfg(not(debug_assertions))]
use panic_halt as _;
#[cfg(debug_assertions)]
use panic_semihosting as _;

use hal::entry;
use hal::pac::{interrupt, CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::{clock::GenericClockController, delay::Delay};
use hal_ext::{serial_print, serial_println, usb_serial};

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = hal::Pins::new(peripherals.PORT);

    let mut delay = Delay::new(core.SYST, &mut clocks);

    let mut red_led = pins.d13.into_open_drain_output(&mut pins.port);
    red_led.set_high().unwrap();

    usb_serial::init(
        peripherals.USB,
        &mut clocks,
        &mut peripherals.MCLK,
        pins.usb_dm,
        pins.usb_dp,
        &mut pins.port,
        &mut core.NVIC,
    );

    let mut n = 0u8;

    loop {
        red_led.set_low().unwrap();
        delay.delay_ms(200u8);
        red_led.set_high().unwrap();
        delay.delay_ms(200u8);

        serial_print!(b"I'm working");

        let num = &[n + 48][..];
        serial_println!(num);

        n = (n + 1) % 10;
    }
}

#[interrupt]
fn USB_TRCPT0() {
    usb_serial::default_poll_usb();
}

#[interrupt]
fn USB_TRCPT1() {
    usb_serial::default_poll_usb();
}

#[interrupt]
fn USB_SOF_HSOF() {
    usb_serial::default_poll_usb();
}

#[interrupt]
fn USB_OTHER() {
    usb_serial::default_poll_usb();
}
