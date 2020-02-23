#![no_std] // don't load the standard library for rust
#![feature(panic_info_message, asm)] // enable inline assembly and panic info

/*
+-----------+
|RUST MACROS|
+-----------+
*/

// tt (token tree) directive indicates that the arg can be any value type
#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!(crate::uart::Uart::new(0x1000_0000), $($args)+);
    });
}

// Macro below uses guard pattern to change functionality based on args
// expr indicates format expression style arg ({}, "value")
#[macro_export]
macro_rules! println
{
    () => ({
        print!("\r\n")
    });
    ($fmt:expr) => ({
        print!(concat!($fmt, "\r\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        print!(concat!($fmt, "\r\n"), $($args)+)
    });
}

/*
+-------------------------------+
|LANGUAGE STRUCTURES / FUNCTIONS|
+-------------------------------+
*/

#[no_mangle] // don't mangle symbols so linker can use them
extern "C" fn eh_personality() {}

#[panic_handler] // mark this function as entry on panic
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Aborting: ");
    if let Some(p) = info.location() {
        println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        println!("no information available.");
    }
    abort();
}

#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            asm!("wfi"::::"volatile");
        }
    }
}

/*
+-----------+
|ENTRY POINT|
+-----------+
*/

#[no_mangle]
extern "C" fn kmain() {
    let mut my_uart = uart::Uart::new(0x1000_0000);
    my_uart.init();

    println!("This is my operating system!");
    println!("I'm so awesome. If you start typing something, I'll show you what you typed!");
}

// we use unsafe here so we can use raw pointers
unsafe fn mmio_write(address: usize, offset: usize, value: u8) {
    // When we write to UART we set THR (Transmitter holding register)
    // to point at location of UART MMIO address
    let reg = address as *mut u8;

    // then we write to (reg+offset) our value with
    // write_volatile so compiler does not ignore
    reg.add(offset).write_volatile(value);
}

unsafe fn mmio_read(address: usize, offset: usize, value: u8) -> u8 {
    // When we read from UART we read RBR (Receive Buffer Register)
    // to get values from UART MMIO
    let reg = address as *mut u8;

    // then we read from (reg+offset) our value with
    // read_volatile so compiler does not ignore
    reg.add(offset).read_volatile()
}

/*
+------------+
|RUST MODULES|
+------------+
*/

pub mod uart;
