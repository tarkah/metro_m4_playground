use metro_m4 as hal;
use metro_m4::clock::GenericClockController;
use metro_m4::gpio::{Floating, Input, Pa24, Pa25, Port};
use metro_m4::pac::{interrupt, MCLK, USB};
use metro_m4::usb::UsbBus;

use cortex_m::peripheral::NVIC;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
pub static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
pub static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;

static mut RECV_BUF: [u8; 256] = [0; 256];
static mut RECV_IDX: usize = 0;

#[macro_export]
macro_rules! serial_println {
    ($str: tt) => {{
        cortex_m::interrupt::free(|_| unsafe {
            $crate::usb_serial::USB_BUS.as_mut().map(|_| {
                $crate::usb_serial::USB_SERIAL.as_mut().map(|serial| {
                    let _ = serial.write($str);
                    let _ = serial.write(b"\n\r");
                });
            })
        });
    }};
}

#[macro_export]
macro_rules! serial_print {
    ($str: tt) => {{
        cortex_m::interrupt::free(|_| unsafe {
            $crate::usb_serial::USB_BUS.as_mut().map(|_| {
                $crate::usb_serial::USB_SERIAL.as_mut().map(|serial| {
                    let _ = serial.write($str);
                });
            })
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
    nvic: &mut NVIC,
) {
    // Setup USB Serial Device
    let bus_allocator = unsafe {
        USB_ALLOCATOR = Some(hal::usb_allocator(usb_dm, usb_dp, usb, clocks, mclk, port));
        USB_ALLOCATOR.as_ref().unwrap()
    };

    unsafe {
        USB_SERIAL = Some(SerialPort::new(&bus_allocator));
        USB_BUS = Some(
            UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x2222, 0x3333))
                .manufacturer("Fake company")
                .product("Serial port")
                .serial_number("TEST")
                .device_class(USB_CLASS_CDC)
                .build(),
        );
    }

    unsafe {
        nvic.set_priority(interrupt::USB_TRCPT0, 1);
        NVIC::unmask(interrupt::USB_TRCPT0);
        nvic.set_priority(interrupt::USB_TRCPT1, 1);
        NVIC::unmask(interrupt::USB_TRCPT1);
        nvic.set_priority(interrupt::USB_SOF_HSOF, 1);
        NVIC::unmask(interrupt::USB_SOF_HSOF);
        nvic.set_priority(interrupt::USB_OTHER, 1);
        NVIC::unmask(interrupt::USB_OTHER);
    }
}

pub unsafe fn recv_buf_len() -> usize {
    RECV_IDX
}

pub unsafe fn read_recv(buf: &mut [u8], n: usize) {
    for x in 0..n.min(RECV_IDX) {
        buf[x] = RECV_BUF[x];
    }

    RECV_BUF.rotate_left(n);

    RECV_IDX = RECV_IDX - n.min(RECV_IDX);
}

fn poll_usb() {
    unsafe {
        USB_BUS.as_mut().map(|usb_dev| {
            USB_SERIAL.as_mut().map(|serial| {
                if usb_dev.poll(&mut [serial]) {
                    // Make the other side happy
                    let mut buf = [0u8; 256];

                    if let Ok(n) = serial.read(&mut buf) {
                        for x in 0..n {
                            RECV_BUF[RECV_IDX] = buf[x];

                            RECV_IDX = (RECV_IDX + 1) % 256;
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
