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
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
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

    loop {
        //red_led.set_low().unwrap();
        //delay.delay_ms(200u8);
        //red_led.set_high().unwrap();
        delay.delay_ms(5000u16);

        cortex_m::interrupt::free(|_| unsafe {
            let avail = usb_serial::recv_buf_len();

            if avail > 0 {
                let mut buf = [0; 256];

                usb_serial::read_recv(&mut buf, avail);

                let b = &buf[0..avail];

                serial_print!(b);
            }
        });
    }
}
