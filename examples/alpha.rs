#![no_std]
#![no_main]

use metro_m4 as hal;
use metro_m4_ext as hal_ext;
use panic_halt as _;

use hal::entry;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::{clock::GenericClockController, delay::Delay};
use hal_ext::alphanum::{Display, MultiDisplay};

use ht16k33::HT16K33;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = hal::Pins::new(peripherals.PORT);

    let mut red_led = pins.d13.into_open_drain_output(&mut pins.port);
    red_led.set_low().unwrap();

    let i2c = hal::i2c_master(
        &mut clocks,
        20.khz(),
        peripherals.SERCOM5,
        &mut peripherals.MCLK,
        pins.sda,
        pins.scl,
        &mut pins.port,
    );

    let mut delay = Delay::new(core.SYST, &mut clocks);

    let mut displays = [HT16K33::new(i2c, 112)];
    let mut multidisplay = MultiDisplay::new(&mut displays);
    multidisplay.marquee("hello", &mut delay, 50);

    loop {
        red_led.set_high().unwrap();
        delay.delay_ms(200u8);
        red_led.set_low().unwrap();
        delay.delay_ms(200u8);
    }
}
