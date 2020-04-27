#![no_std] // don't load the standard library for rust
#![feature(panic_info_message, asm)] // enable inline assembly and panic info

const BACKSPACE: u8 = b'\x08';
const NEWLINE: u8 = b'\x0a';
const CARR_RET: u8 = b'\x0d';
const ESCAPE: u8 = b'\x1b';

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


pub fn id_map_range(root: &mut page::Table, start: usize, end: usize, bits: i64) {
    let mut memaddr = start & !(page::PAGE_SIZE - 1);
    let num_kb_pages = (page::align_val(end, 12) - memaddr) / page::PAGE_SIZE;

    for _ in 0..num_kb_pages {
        page::map(root, memaddr, memaddr, bits, 0);
        memaddr += 1 << 12;
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

    page::init();

    for _ in 0..64 {
        page::alloc(1);
    }
    page::alloc(1);
    page::alloc(64);

    page::print_page_allocations();

    // TODO: stopped at end of ch3.2 because no kmem implementation


    // println!("This is my operating system!");
    // println!("I'm so awesome. If you start typing something, I'll show you what you typed!");

    // loop {
    //     if let Some(c) = my_uart.get() {
    //         match c as u8 {
    //             BACKSPACE => {
    //                 // for backspace need to move back 1 char, then overwrite
    //                 // char at point with space, then move back again
    //                 print!("{}{}{}", 8 as char, ' ', 8 as char);
    //             }
    //             NEWLINE | CARR_RET => {
    //                 // newline or carriage return
    //                 println!();
    //             }
    //             // escape char for escape sequence
    //             ESCAPE => {
    //                 if let Some(next_byte) = my_uart.get() {
    //                     // [ for start of sequence
    //                     if next_byte == 91 {
    //                         if let Some(b) = my_uart.get() {
    //                             match b as char {
    //                                 'A' => {
    //                                     println!("Up");
    //                                 }
    //                                 'B' => {
    //                                     println!("Down");
    //                                 }
    //                                 'C' => {
    //                                     println!("Right");
    //                                 }
    //                                 'D' => {
    //                                     println!("Left");
    //                                 }
    //                                 _ => {
    //                                     println!("Invalid");
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //             _ => {
    //                 print!("{}", c as char);
    //             }
    //         }
    //     }
    // }
}

// // we use unsafe here so we can use raw pointers
// unsafe fn mmio_write(address: usize, offset: usize, value: u8) {
//     // When we write to UART we set THR (Transmitter holding register)
//     // to point at location of UART MMIO address
//     let reg = address as *mut u8;

//     // then we write to (reg+offset) our value with
//     // write_volatile so compiler does not ignore
//     reg.add(offset).write_volatile(value);
// }

// unsafe fn mmio_read(address: usize, offset: usize, value: u8) -> u8 {
//     // When we read from UART we read RBR (Receive Buffer Register)
//     // to get values from UART MMIO
//     let reg = address as *mut u8;

//     // then we read from (reg+offset) our value with
//     // read_volatile so compiler does not ignore
//     reg.add(offset).read_volatile()
// }

/*
+------------+
|RUST MODULES|
+------------+
*/

pub mod page;
pub mod uart;
