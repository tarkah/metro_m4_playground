#![no_std]
#![no_main]

use metro_m4 as hal;
use metro_m4_ext as hal_ext;
use panic_halt as _;

use hal::entry;
use hal::pac::{interrupt, CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::{clock::GenericClockController, delay::Delay};
use hal_ext::alphanum::{Display, MultiDisplay, DISP_I2C_ADDR};
use hal_ext::usb_serial::{self, USB_BUS, USB_SERIAL};

use core::cell::RefCell;
use ht16k33::HT16K33;

const BUFFER_SIZE: usize = 512;
static mut BUFFER: [u8; 512] = [0; 512];
static mut WRITE_IDX: usize = 0;
static mut BUFF_LEN: usize = 0;

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

    let mut red_led = pins.d13.into_open_drain_output(&mut pins.port);
    red_led.set_low().unwrap();

    usb_serial::init(
        peripherals.USB,
        &mut clocks,
        &mut peripherals.MCLK,
        pins.usb_dm,
        pins.usb_dp,
        &mut pins.port,
        &mut core.NVIC,
    );

    let i2c = RefCell::new(hal::i2c_master(
        &mut clocks,
        20.khz(),
        peripherals.SERCOM5,
        &mut peripherals.MCLK,
        pins.sda,
        pins.scl,
        &mut pins.port,
    ));

    let mut delay = Delay::new(core.SYST, &mut clocks);

    let mut displays = [
        HT16K33::new(&i2c, DISP_I2C_ADDR),
        HT16K33::new(&i2c, DISP_I2C_ADDR + 1),
        HT16K33::new(&i2c, DISP_I2C_ADDR + 2),
    ];
    let mut multidisplay = MultiDisplay::new(&mut displays);

    let mut s = "";

    loop {
        cortex_m::interrupt::free(|_| unsafe {
            if BUFF_LEN > 0 {
                s = core::str::from_utf8_unchecked(&BUFFER[0..BUFF_LEN]);

                BUFF_LEN = 0;
            }
        });

        if !s.is_empty() {
            multidisplay.marquee(s, &mut delay, Some(200), true);
        }
    }
}

fn poll_usb() {
    unsafe {
        USB_BUS.as_mut().map(|usb_dev| {
            USB_SERIAL.as_mut().map(|serial| {
                if usb_dev.poll(&mut [serial]) {
                    // Make the other side happy
                    let mut buf = [0u8; BUFFER_SIZE];
                    if let Ok(n) = serial.read(&mut buf) {
                        let end = buf[n - 1] == 0;
                        let through = if end { n - 1 } else { n };

                        // Dont panic if we write more than buffer can hold
                        // just return and display up till last value
                        if WRITE_IDX + n >= BUFFER_SIZE {
                            WRITE_IDX = 0;
                            BUFF_LEN = BUFFER_SIZE;

                            return;
                        }

                        if through > 0 {
                            BUFFER[WRITE_IDX..WRITE_IDX + through]
                                .swap_with_slice(&mut buf[0..through]);
                        }

                        if end {
                            BUFF_LEN = WRITE_IDX + through;
                            WRITE_IDX = 0;
                        } else {
                            WRITE_IDX += through;
                        }
                    }
                }
            });
        });
    }
}

#[interrupt]
fn USB_TRCPT0() {
    poll_usb();
}

#[interrupt]
fn USB_TRCPT1() {
    poll_usb();
}

#[interrupt]
fn USB_SOF_HSOF() {
    poll_usb();
}

#[interrupt]
fn USB_OTHER() {
    poll_usb();
}
