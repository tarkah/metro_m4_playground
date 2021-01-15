#![no_std]
#![no_main]

use metro_m4 as hal;
use panic_halt as _;
use ws2812_timer_delay as ws2812;

use hal::entry;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::timer::SpinTimer;
use hal::{clock::GenericClockController, delay::Delay};

use embedded_hal::digital::v1_compat::OldOutputPin;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
};

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

    let timer = SpinTimer::new(4);
    let neopixel_pin: OldOutputPin<_> = pins.neopixel.into_push_pull_output(&mut pins.port).into();
    let mut neopixel = ws2812::Ws2812::new(timer, neopixel_pin);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    loop {
        for j in 0..255u8 {
            let colors = [hsv2rgb(Hsv {
                hue: j,
                sat: 255,
                val: 32,
            })];
            neopixel.write(colors.iter().cloned()).unwrap();
            delay.delay_ms(50u8);
        }
    }
}
