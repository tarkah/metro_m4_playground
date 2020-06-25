use crate::hal::clock::GenericClockController;
use crate::hal::gpio::{Floating, Input, Pa24, Pa25, Port};
use crate::hal::pac::{interrupt, MCLK, USB};
use crate::hal::usb::UsbBus;

use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{DefaultBufferStore, SerialPort, USB_CLASS_CDC};

pub const SERIAL_WRITE_BUF_LEN: usize = 256;

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;
pub static mut SERIAL_WRITE_BUF: [u8; SERIAL_WRITE_BUF_LEN] = [0; SERIAL_WRITE_BUF_LEN];
pub static mut SERIAL_WRITE_LEN: usize = 0;

#[macro_export]
macro_rules! serial_print {
    ($str: tt) => {{
        cortex_m::interrupt::free(|_| unsafe {
            let bytes = $str.as_bytes();
            let count = bytes.len();

            if count > $crate::usb_serial::SERIAL_WRITE_BUF_LEN {
                panic!(
                    "String more than {} bytes",
                    $crate::usb_serial::SERIAL_WRITE_BUF_LEN
                );
            }

            for (n, x) in $crate::usb_serial::SERIAL_WRITE_BUF[0..count]
                .iter_mut()
                .enumerate()
            {
                *x = bytes[n];
            }

            $crate::usb_serial::SERIAL_WRITE_LEN = count;
        });
    }};
}

pub fn init(
    usb: USB,
    clocks: &mut GenericClockController,
    mclk: &mut MCLK,
    usb_dm: Pa24<Input<Floating>>,
    usb_dp: Pa25<Input<Floating>>,
    port: &mut Port,
) {
    // Setup USB Serial Device
    let bus_allocator = unsafe {
        USB_ALLOCATOR = Some(crate::hal::usb_bus(usb, clocks, mclk, usb_dm, usb_dp, port));
        USB_ALLOCATOR.as_ref().unwrap()
    };

    unsafe {
        USB_SERIAL = Some(SerialPort::new(&bus_allocator));
        USB_BUS = Some(
            UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x16c0, 0x27dd))
                .manufacturer("Fake company")
                .product("Serial port")
                .serial_number("TEST")
                .device_class(USB_CLASS_CDC)
                .build(),
        );
    }

    serial_print!("USB Serial Device Initialized!\n");
}

fn poll_usb() {
    unsafe {
        USB_BUS.as_mut().map(|usb_dev| {
            USB_SERIAL.as_mut().map(|serial| {
                if usb_dev.poll(&mut [serial]) {
                    serial_write(serial);
                }
            });
        });
    }
}

unsafe fn serial_write(serial: &mut SerialPort<UsbBus, DefaultBufferStore, DefaultBufferStore>) {
    if SERIAL_WRITE_LEN > 0 {
        if let Ok(count) = serial.write(&SERIAL_WRITE_BUF[0..SERIAL_WRITE_LEN]) {
            if count < SERIAL_WRITE_LEN {
                let remaining = SERIAL_WRITE_LEN - count;

                for (n, x) in SERIAL_WRITE_BUF.iter_mut().enumerate() {
                    if n < remaining {
                        *x = SERIAL_WRITE_BUF[n + count];
                    }
                }

                SERIAL_WRITE_LEN = remaining;
            } else {
                SERIAL_WRITE_LEN = 0;
                for x in SERIAL_WRITE_BUF.iter_mut() {
                    *x = 0
                }
            }
        }
    }
}

#[interrupt]
fn USB_OTHER() {
    poll_usb();
}

#[interrupt]
fn USB_TRCPT0() {
    poll_usb();
}

#[interrupt]
fn USB_TRCPT1() {
    poll_usb();
}
