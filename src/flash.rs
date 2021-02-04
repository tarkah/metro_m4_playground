use embedded_hal::blocking::delay::DelayMs;
use metro_m4::hal::delay::Delay;
use metro_m4::hal::gpio::{Floating, Input, Pa10, Pa11, Pa8, Pa9, Pb10, Pb11, Port};
use metro_m4::hal::qspi::{Command, OneShot, Qspi};
use metro_m4::pac::{MCLK, QSPI};

pub struct QspiFlash {
    flash: Qspi<OneShot>,
}

impl QspiFlash {
    pub fn new(
        delay: &mut Delay,
        mclk: &mut MCLK,
        port: &mut Port,
        qspi: QSPI,
        sck: Pb10<Input<Floating>>,
        cs: Pb11<Input<Floating>>,
        io0: Pa8<Input<Floating>>,
        io1: Pa9<Input<Floating>>,
        io2: Pa10<Input<Floating>>,
        io3: Pa11<Input<Floating>>,
    ) -> Self {
        let mut flash = Qspi::new(mclk, port, qspi, sck, cs, io0, io1, io2, io3);

        // Startup delay. Can't find documented but Adafruit use 5ms
        delay.delay_ms(5u8);
        // Reset. It is recommended to check the BUSY(WIP?) bit and the SUS before reset
        wait_ready(&mut flash);
        flash.run_command(Command::EnableReset).unwrap();
        flash.run_command(Command::Reset).unwrap();
        // tRST(30μs) to reset. During this period, no command will be accepted
        delay.delay_ms(1u8);

        // 120MHz / 2 = 60mhz
        // faster than 104mhz at 3.3v would require High Performance Mode
        flash.set_clk_divider(2);

        // Enable Quad SPI mode. Requires write enable. Check WIP.
        flash.run_command(Command::WriteEnable).unwrap();
        flash.write_command(Command::WriteStatus2, &[0x02]).unwrap();
        wait_ready(&mut flash);

        QspiFlash { flash }
    }

    pub fn write(&mut self, addr: u32, buffer: &[u8]) {
        // Page Program. Requires write enable. Check WIP.
        // If more than 256 bytes are sent to the device, previously latched data
        // are discarded and the last 256 data bytes are guaranteed to be
        // programmed correctly within the same page. If less than 256 data
        // bytes are sent to device, they are correctly programmed at the
        // requested addresses without having any effects on the other bytes of
        // the same page

        self.flash.run_command(Command::WriteEnable).unwrap();
        self.flash.write_memory(addr, buffer);
        wait_ready(&mut self.flash);
    }

    pub fn read(&mut self, addr: u32, buffer: &mut [u8]) {
        // Read back data
        // datasheet claims 6BH needs a single dummy byte, but doesnt work then
        // adafruit uses 8, and the underlying implementation uses 8 atm as well
        self.flash.read_memory(addr, buffer);
    }

    pub fn erase_chip(&mut self) {
        // Chip Erase. Requires write enable. Check WIP.
        self.flash.run_command(Command::WriteEnable).unwrap();
        self.flash.erase_command(Command::EraseChip, 0x0).unwrap();
        wait_ready(&mut self.flash);
    }

    pub fn erase_sector(&mut self, addr: u32) {
        // Chip Erase. Requires write enable. Check WIP.
        self.flash.run_command(Command::WriteEnable).unwrap();
        self.flash
            .erase_command(Command::EraseSector, addr)
            .unwrap();
        wait_ready(&mut self.flash);
    }
}

/// Wait for the write-in-progress and suspended write/erase.
fn wait_ready(flash: &mut Qspi<OneShot>) {
    while flash_status(flash, Command::ReadStatus) & 0x01 != 0 {}
    while flash_status(flash, Command::ReadStatus2) & 0x80 != 0 {}
}

/// Returns the contents of the status register indicated by cmd.
fn flash_status(flash: &mut Qspi<OneShot>, cmd: Command) -> u8 {
    let mut out = [0u8; 1];
    flash.read_command(cmd, &mut out).ok().unwrap();
    out[0]
}
