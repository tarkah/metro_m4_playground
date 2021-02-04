#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[macro_use]
extern crate alloc;

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
use hal::pac::{interrupt, CorePeripherals, Peripherals, SCB};
use hal::prelude::*;
use hal::sercom::I2CMaster5;
use hal_ext::alphanum::{Display, MultiDisplay, DISP_I2C_ADDR};
use hal_ext::usb_serial::{self, USB_BUS, USB_SERIAL};

use alloc_cortex_m::CortexMHeap;
#[cfg(debug_assertions)]
use cortex_m_log::log::{trick_init, Logger};
#[cfg(debug_assertions)]
use cortex_m_log::printer::semihosting;

use ds323x::{Datelike, Ds323x, Rtcc, Timelike};
use ht16k33::HT16K33;
use shared_bus::new_cortexm;

const BUFFER_SIZE: usize = 512;
static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
static mut BUFF_LEN: usize = 0;
static mut WRITE_IDX: usize = 0;

const BUFFER_ADDR: u32 = 0x0;
const BUFFER_LEN_ADDR: u32 = BUFFER_ADDR + BUFFER_SIZE as u32;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    let start = cortex_m_rt::heap_start() as usize;
    let size = 1024; // in bytes
    unsafe { ALLOCATOR.init(start, size) }

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

    let mut delay = Delay::new(core.SYST, &mut clocks);

    usb_serial::init(
        peripherals.USB,
        &mut clocks,
        &mut peripherals.MCLK,
        pins.usb_dm,
        pins.usb_dp,
        &mut pins.port,
        &mut core.NVIC,
    );

    let mut flash = hal_ext::flash::QspiFlash::new(
        &mut delay,
        &mut peripherals.MCLK,
        &mut pins.port,
        peripherals.QSPI,
        pins.flash_sck,
        pins.flash_cs,
        pins.flash_mosi,
        pins.flash_miso,
        pins.flash_io2,
        pins.flash_io3,
    );

    // Populate buffer from what's stored in flash
    unsafe {
        flash.read(BUFFER_ADDR, &mut BUFFER[..]);

        let mut buff = [0; 4];
        flash.read(BUFFER_LEN_ADDR, &mut buff);
        BUFF_LEN = usize::from_le_bytes(buff);
    }

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

    let mut clock = Ds323x::new_ds3231(shared_bus.acquire_i2c());

    // Only needed on first run, or if batter isn't inserted
    //let now = NaiveDateTime::parse_from_str("2021-01-26 11:40:00", "%Y-%m-%d %H:%M:%S").unwrap();
    //clock.set_datetime(&now).unwrap();

    let mut text_buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut text_len = 0usize;

    loop {
        let mut last_second = 0;
        let mut delay_total = 0;

        while delay_total < 5000 {
            let time = clock.get_time().unwrap();
            let (is_pm, hour) = time.hour12();
            let minute = time.minute();
            let second = time.second();
            let am_pm = if is_pm { "PM" } else { "AM" };

            if second != last_second {
                last_second = second;

                let time_str = format!("{:0>2?}{:0>2}{:0>2} {}   ", hour, minute, second, am_pm);

                let dot_flags = [
                    false, true, false, true, false, false, false, false, false, false, false,
                    false,
                ];

                if let Err(e) = multidisplay.display(time_str.as_bytes(), Some(&dot_flags)) {
                    #[cfg(debug_assertions)]
                    log::error!("{:?}", e);

                    SCB::sys_reset();
                }
            }

            delay.delay_ms(100u16);
            delay_total += 100;
        }

        let mut last_temp = 0;
        delay_total = 0;

        while delay_total < 5000 {
            let temp = (clock.get_temperature().unwrap() * 1.8 + 32.0) as u32;
            let date = clock.get_date().unwrap();
            let month = date.month();
            let day = date.day();
            let weekday = date.weekday();
            let weekday_str = format!("{}", weekday);

            if temp != last_temp {
                last_temp = temp;

                let temp_str = format!(
                    "{} {:0>2}{:0>2} {: >2}F",
                    &weekday_str[0..3],
                    month,
                    day,
                    temp
                );

                let dot_flags = [
                    false, false, false, false, false, true, false, false, false, false, false,
                    false,
                ];

                if let Err(e) = multidisplay.display(temp_str.as_bytes(), Some(&dot_flags)) {
                    #[cfg(debug_assertions)]
                    log::error!("{:?}", e);

                    SCB::sys_reset();
                }
            }

            delay.delay_ms(100u16);
            delay_total += 100;
        }

        cortex_m::interrupt::free(|_| unsafe {
            if BUFF_LEN > 0 {
                text_buf = BUFFER;
                text_len = BUFF_LEN;

                flash.erase_sector(0x0);

                // Save new word to memory
                flash.write(BUFFER_ADDR, &text_buf[..256]);
                flash.write(BUFFER_ADDR + 256, &text_buf[256..]);
                flash.write(BUFFER_LEN_ADDR, &usize::to_le_bytes(text_len)[..]);

                BUFF_LEN = 0;
            }
        });

        let mut n = 0;
        while n < 2 {
            if text_len > 0 {
                let text = unsafe { core::str::from_utf8_unchecked(&text_buf[0..text_len]) };

                if let Err(e) = multidisplay.marquee(text, &mut delay, 200u8, true) {
                    #[cfg(debug_assertions)]
                    log::error!("{:?}", e);

                    SCB::sys_reset();
                }
            }

            n += 1;
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
