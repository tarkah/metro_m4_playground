use crate::hal::delay::Delay;
use adafruit_alphanum4::{AlphaNum4, AsciiChar, Index};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::i2c;
use ht16k33::HT16K33;

pub const DISP_I2C_ADDR: u8 = 112;
const MAX_DRIVERS: usize = 10;

pub struct MultiDisplay<'a, I2C> {
    drivers: &'a mut [HT16K33<I2C>],
}

impl<'a, I2C, E> MultiDisplay<'a, I2C>
where
    I2C: i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    E: core::fmt::Debug,
{
    pub fn new(drivers: &'a mut [HT16K33<I2C>]) -> MultiDisplay<'a, I2C> {
        for driver in drivers.iter_mut() {
            driver.initialize().unwrap();
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
                AsciiChar::Null
            };

            let driver = &mut drivers[n / 4];

            driver.update_buffer_with_char(index, ascii);
        }

        for driver in drivers.iter_mut() {
            driver.write_display_buffer().unwrap();
        }
    }

    fn marquee(&mut self, text: &str, delay: &mut Delay, delay_ms: u16) {
        let num_drivers = self.drivers().len();

        let mut _buf = [0; MAX_DRIVERS];
        let buffer = &mut _buf[0..num_drivers * 4];

        let bytes = text.as_bytes();

        for b in bytes {
            // Shift all bytes in buf to the left
            for n in 1..buffer.len() {
                buffer.swap(n - 1, n);
            }

            // Update last byte
            buffer[num_drivers * 4 - 1] = *b;

            // Display buffer
            self.display(buffer);

            // Wait ms before scrolling
            delay.delay_ms(delay_ms);
        }
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
