use adafruit_alphanum4::{AlphaNum4, AsciiChar, Index};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::i2c;
use ht16k33::HT16K33;
use metro_m4::delay::Delay;

pub const DISP_I2C_ADDR: u8 = 112;
const LEDS_PER_DRIVER: usize = 4;
const MAX_DRIVERS: usize = 10;

pub struct MultiDisplay<'a, I2C> {
    drivers: &'a mut [HT16K33<'a, I2C>],
}

impl<'a, I2C, E> MultiDisplay<'a, I2C>
where
    I2C: i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    E: core::fmt::Debug,
{
    pub fn new(drivers: &'a mut [HT16K33<'a, I2C>]) -> MultiDisplay<'a, I2C> {
        if drivers.len() > MAX_DRIVERS {
            panic!("Can't use more than 10 drivers with this struct")
        }

        for driver in drivers.iter_mut() {
            driver.initialize().unwrap();
            driver.set_display(ht16k33::Display::ON).unwrap();
        }

        MultiDisplay { drivers }
    }
}

pub trait Display<'a, I2C, E>
where
    I2C: i2c::Write<Error = E> + i2c::WriteRead<Error = E> + 'a,
    E: core::fmt::Debug,
{
    fn display(&mut self, buffer: &[u8]);

    fn marquee(&mut self, text: &str, delay: &mut Delay, delay_ms: Option<u16>, clear_end: bool);
}

impl<'a, I2C, E> Display<'a, I2C, E> for MultiDisplay<'a, I2C>
where
    I2C: i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    E: core::fmt::Debug,
{
    fn display(&mut self, buffer: &[u8]) {
        let drivers = self.drivers.as_mut();

        for (n, buff) in buffer.chunks(LEDS_PER_DRIVER).enumerate() {
            let driver = drivers.get_mut(n).unwrap();

            for (idx, b) in buff.iter().enumerate() {
                let index: Index = (idx as u8).into();

                let ascii = if b.is_ascii() {
                    unsafe { AsciiChar::from_ascii_unchecked(*b) }
                } else {
                    AsciiChar::Space
                };

                driver.update_buffer_with_char(index, ascii);
            }

            let _ = driver.write_display_buffer();
        }
    }

    fn marquee(&mut self, text: &str, delay: &mut Delay, delay_ms: Option<u16>, clear_end: bool) {
        let num_drivers = self.drivers.len();
        let num_leds = num_drivers * LEDS_PER_DRIVER;

        let mut buffer = [AsciiChar::Space as u8; MAX_DRIVERS * LEDS_PER_DRIVER];

        let bytes = text.as_bytes();

        for b in bytes {
            shift_left_and_insert_last(*b, &mut buffer[0..num_leds]);

            // Display buffer
            self.display(&mut buffer[0..num_leds]);

            // Wait ms before scrolling
            if let Some(ms) = delay_ms {
                delay.delay_ms(ms);
            }
        }

        if clear_end {
            for _ in 0..num_leds {
                shift_left_and_insert_last(32, &mut buffer[0..num_leds]);

                // Display buffer
                self.display(&mut buffer[0..num_leds]);

                // Wait ms before scrolling
                if let Some(ms) = delay_ms {
                    delay.delay_ms(ms);
                }
            }
        }
    }
}

fn shift_left_and_insert_last(b: u8, buffer: &mut [u8]) {
    // Shift all bytes in buf to the left
    for n in 1..buffer.len() {
        buffer.swap(n - 1, n);
    }

    // Update last byte
    buffer[buffer.len() - 1] = b;
}
