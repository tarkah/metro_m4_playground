use adafruit_alphanum4::{AlphaNum4, AsciiChar, Index};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::i2c;
use ht16k33::HT16K33;
use metro_m4::delay::Delay;

pub const DISP_I2C_ADDR: u8 = 112;
const MAX_DRIVERS: usize = 10;
static mut BUFFER: [u8; MAX_DRIVERS * 4] = [0; MAX_DRIVERS * 4];

pub struct MultiDisplay<'a, I2C> {
    drivers: &'a mut [HT16K33<I2C>],
}

impl<'a, I2C, E> MultiDisplay<'a, I2C>
where
    I2C: i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    E: core::fmt::Debug,
{
    pub fn new(drivers: &'a mut [HT16K33<I2C>]) -> MultiDisplay<'a, I2C> {
        if drivers.len() > MAX_DRIVERS {
            panic!("Can't use more than 10 drivers with this struct")
        }

        // Set entire buffer as empty (spaces)
        unsafe {
            BUFFER = [AsciiChar::Space as u8; MAX_DRIVERS * 4];
        }

        for driver in drivers.iter_mut() {
            driver.initialize().unwrap();
            driver.set_display(ht16k33::Display::ON).unwrap();
        }

        MultiDisplay { drivers }
    }
}

pub trait Display<I2C, E>
where
    I2C: i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    E: core::fmt::Debug,
{
    fn drivers(&mut self) -> &mut [HT16K33<I2C>];

    fn display(&mut self, buffer: &[u8]) {
        let drivers = self.drivers();

        for (n, b) in buffer.iter().enumerate() {
            let index: Index = (n as u8 % 4).into();

            let ascii = if b.is_ascii() {
                unsafe { AsciiChar::from_ascii_unchecked(*b) }
            } else {
                AsciiChar::Space
            };

            let driver = &mut drivers[n / 4];

            driver.update_buffer_with_char(index, ascii);
        }

        for driver in drivers.iter_mut() {
            driver.write_display_buffer().unwrap();
        }
    }

    fn marquee(&mut self, text: &str, delay: &mut Delay, delay_ms: Option<u16>, clear_end: bool) {
        let num_drivers = self.drivers().len();
        let num_leds = num_drivers * 4;

        let buffer = unsafe { &mut BUFFER[0..num_leds] };

        let bytes = text.as_bytes();

        for b in bytes {
            self.write_scroll(*b, buffer);

            // Wait ms before scrolling
            if let Some(ms) = delay_ms {
                delay.delay_ms(ms);
            }
        }

        if clear_end {
            for _ in 0..num_leds {
                self.write_scroll(32, buffer);

                // Wait ms before scrolling
                if let Some(ms) = delay_ms {
                    delay.delay_ms(ms);
                }
            }
        }
    }

    fn write_scroll(&mut self, b: u8, buffer: &mut [u8]) {
        // Shift all bytes in buf to the left
        for n in 1..buffer.len() {
            buffer.swap(n - 1, n);
        }

        // Update last byte
        buffer[buffer.len() - 1] = b;

        // Display buffer
        self.display(buffer);
    }
}

impl<'a, I2C, E> Display<I2C, E> for MultiDisplay<'a, I2C>
where
    I2C: i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    E: core::fmt::Debug,
{
    fn drivers(&mut self) -> &mut [HT16K33<I2C>] {
        self.drivers
    }
}
