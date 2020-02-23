use core::convert::TryInto;
use core::fmt::{Error, Write};

pub struct Uart {
    base_addr: usize,
}

// implement the Write trait for Uart struct, adding in the required
// fn `write_str` for the trait's functionality
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        // Result can be one of None or Error
        for c in s.bytes() {
            self.put(c);
        }

        Ok(())
    }
}

impl Uart {
    pub fn new(base_addr: usize) -> Self {
        Uart { base_addr }
    }

    // Initialize UART (Universal Async Receiver-Transmitter)
    // Set word length, FIFO mode, and interrupt handling
    pub fn init(&mut self) {
        let ptr = self.base_addr as *mut u8;
        unsafe {
            // Set LCR (Line Control Register) at base + 3 to 0b11
            // to enable word length selection
            let lcr = (1 << 1) | (1 << 0);
            ptr.add(3).write_volatile(lcr);

            // Set FCR (FIFO Control Register) at base + 2 to 0b1 to enable
            // using a stack instead of a queue for UART read/write buffer
            let fifo = 1 << 0;
            ptr.add(2).write_volatile(fifo);

            // Enable receive buffer interrupts (IER at base + 1)
            // so we can trigger interrupts when data written to RBR
            let ier = 1 << 0;
            ptr.add(1).write_volatile(ier);

            // signalling divisor determines how often the CPU checks for signals
            // and is calculated by ceil(clock_rate / signaling_rate (in BAUD) * 16)
            // Using a value of 2400 for BAUD:
            // 22_729_000Hz / 2400 x 1600 ~= 592 as divisor

            // can only write 1 byte at a time so split divisor and write each
            let divisor: u16 = 592;
            let divisor_lo: u8 = (divisor & 0xff).try_into().unwrap();
            let divisor_hi: u8 = (divisor >> 8).try_into().unwrap();

            // need to flip Divisor Latch acccess Bit (DLAB) so that base + 0 and
            // base + 1 point to divisor latch least (DLL) and divisor latch most (DLM) bytes
            // instead of THR/RBR and IER
            let dlab = 1 << 7;
            ptr.add(3).write_volatile(lcr | dlab);

            ptr.add(0).write_volatile(divisor_lo);
            ptr.add(1).write_volatile(divisor_hi);

            // clear DLAB bit now so that we can access our RBR, THR, and IER again
            ptr.add(3).write_volatile(lcr);
        }
    }

    fn get(&mut self) -> Option<u8> {
        // *mut is a raw mutable pointer, meant to be shared and modified so long
        // as it's not changed to None
        let ptr = self.base_addr as *mut u8;
        unsafe {
            // Bit 0 of Line Status Register is the Data Ready (DR) register, which
            // indicates if there is data to be read from RBR
            if ptr.add(5).read_volatile() & 1 == 0 {
                // No data to be read, return nothing
                None
            } else {
                // bit must be 1, data can be received
                // Use Some to indicate a return that can be
                // evaluated for different return types
                Some(ptr.add(0).read_volatile())
            }
        }
    }

    fn put(&mut self, c: u8) {
        let ptr = self.base_addr as *mut u8;
        unsafe {
            ptr.add(0).write_volatile(c);
        }
    }
}
