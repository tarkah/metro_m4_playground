#![no_std]
#![no_main]

use metro_m4 as hal;
use metro_m4_ext as hal_ext;

#[cfg(not(debug_assertions))]
use panic_halt as _;
#[cfg(debug_assertions)]
use panic_semihosting as _;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::entry;
use hal::gpio;
use hal::pac::{interrupt, CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::sercom::I2CMaster5;
use hal_ext::alphanum::{Display, MultiDisplay, DISP_I2C_ADDR};
use hal_ext::usb_serial::{self, USB_BUS, USB_SERIAL};

#[cfg(debug_assertions)]
use cortex_m_log::log::{trick_init, Logger};
#[cfg(debug_assertions)]
use cortex_m_log::printer::semihosting;

use ht16k33::HT16K33;
use shared_bus::new_cortexm;

const BUFFER_SIZE: usize = 512;
static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
static mut BUFF_LEN: usize = 0;
static mut WRITE_IDX: usize = 0;

#[entry]
fn main() -> ! {
    #[cfg(debug_assertions)]
    {
        let logger = Logger {
            inner: semihosting::InterruptOk::<_>::stdout().unwrap(),
            level: log::LevelFilter::Info,
        };

        unsafe {
            let _ = trick_init(&logger);
        }
    }

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

    usb_serial::init(
        peripherals.USB,
        &mut clocks,
        &mut peripherals.MCLK,
        pins.usb_dm,
        pins.usb_dp,
        &mut pins.port,
        &mut core.NVIC,
    );

    let i2c = hal::i2c_master(
        &mut clocks,
        400.khz(),
        peripherals.SERCOM5,
        &mut peripherals.MCLK,
        pins.sda,
        pins.scl,
        &mut pins.port,
    );

    let shared_bus = new_cortexm!(I2CMaster5<
        hal::sercom::Sercom5Pad0<gpio::Pb2<gpio::PfD>>,
        hal::sercom::Sercom5Pad1<gpio::Pb3<gpio::PfD>>,
    > = i2c)
    .unwrap();

    let displays = [
        HT16K33::new(shared_bus.acquire_i2c(), DISP_I2C_ADDR),
        HT16K33::new(shared_bus.acquire_i2c(), DISP_I2C_ADDR + 1),
        HT16K33::new(shared_bus.acquire_i2c(), DISP_I2C_ADDR + 2),
    ];

    let mut multidisplay = MultiDisplay::new(displays);

    let mut delay = Delay::new(core.SYST, &mut clocks);

    let mut text_buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut text_len = 0usize;

    loop {
        cortex_m::interrupt::free(|_| unsafe {
            if BUFF_LEN > 0 {
                text_buf = BUFFER;
                text_len = BUFF_LEN;

                BUFF_LEN = 0;
            }
        });

        //if text_len > 0 {
        if 1 == 1 {
            let text = "TESTING, TESTING, 1, 2, 3"; //core::str::from_utf8_unchecked(&text_buf[0..text_len]);

            if let Err(e) = multidisplay.marquee(text, &mut delay, 200u8, true) {
                #[cfg(debug_assertions)]
                log::error!("{:?}", e);
            }
        }
    }
}

fn poll_usb() {
    unsafe {
        USB_BUS.as_mut().map(|usb_dev| {
            USB_SERIAL.as_mut().map(|serial| {
                if usb_dev.poll(&mut [serial]) {
                    // Make the other side happy
                    let mut buf = [0u8; 128];
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
