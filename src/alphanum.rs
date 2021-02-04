use adafruit_alphanum4::{AlphaNum4, AsciiChar, Index};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::i2c;
use ht16k33::HT16K33;

pub const DISP_I2C_ADDR: u8 = 112;
const LEDS_PER_DRIVER: usize = 4;
const MAX_DRIVERS: usize = 10;

pub struct MultiDisplay<I2C, const N: usize> {
    drivers: [HT16K33<I2C>; N],
}

impl<'a, I2C, E, const N: usize> MultiDisplay<I2C, N>
where
    I2C: i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    E: core::fmt::Debug,
{
    pub fn new(mut drivers: [HT16K33<I2C>; N]) -> MultiDisplay<I2C, N> {
        if drivers.len() > MAX_DRIVERS {
            panic!("Can't use more than 10 drivers with this struct")
        }

        for driver in drivers.iter_mut() {
            driver.initialize().unwrap();
            driver.set_display(ht16k33::Display::ON).unwrap();
        }

        log::info!("{} drivers initialized", drivers.len());

        MultiDisplay { drivers }
    }
}

pub trait Display<E>
where
    E: core::fmt::Debug,
{
    fn display(&mut self, buffer: &[u8], enable_dot: Option<&[bool]>) -> Result<(), E>;

    fn marquee<Delay, UXX>(
        &mut self,
        text: &str,
        delay: &mut Delay,
        delay_ms: UXX,
        clear_end: bool,
    ) -> Result<(), E>
    where
        Delay: DelayMs<UXX>,
        UXX: Copy;
}

impl<I2C, E, const N: usize> Display<E> for MultiDisplay<I2C, N>
where
    I2C: i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    E: core::fmt::Debug,
{
    fn display(&mut self, buffer: &[u8], enable_dot: Option<&[bool]>) -> Result<(), E> {
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

                if let Some(dot_flags) = enable_dot {
                    let flag_idx = n * 4 + idx;

                    let enable = dot_flags[flag_idx];

                    if enable {
                        driver.update_buffer_with_dot(index, true);
                    }
                }
            }

            driver.write_display_buffer()?;
        }

        Ok(())
    }

    fn marquee<Delay, UXX>(
        &mut self,
        text: &str,
        delay: &mut Delay,
        delay_ms: UXX,
        clear_end: bool,
    ) -> Result<(), E>
    where
        Delay: DelayMs<UXX>,
        UXX: Copy,
    {
        let num_drivers = self.drivers.len();
        let num_leds = num_drivers * LEDS_PER_DRIVER;

        let mut buffer = [AsciiChar::Space as u8; MAX_DRIVERS * LEDS_PER_DRIVER];

        let bytes = text.as_bytes();

        for b in bytes {
            shift_left_and_insert_last(*b, &mut buffer[0..num_leds]);

            // Display buffer
            self.display(&mut buffer[0..num_leds], None)?;

            // Wait ms before scrolling
            delay.delay_ms(delay_ms);
        }

        if clear_end {
            for _ in 0..num_leds {
                shift_left_and_insert_last(32, &mut buffer[0..num_leds]);

                // Display buffer
                self.display(&mut buffer[0..num_leds], None)?;

                // Wait ms before scrolling
                delay.delay_ms(delay_ms);
            }
        }

        Ok(())
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
