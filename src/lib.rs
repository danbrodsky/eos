#![no_std] // don't load the standard library for rust
#![feature(panic_info_message, asm)] // enable inline assembly and panic info

/*
+-----------+
|RUST MACROS|
+-----------+
*/

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => {{}};
}
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

#[no_mangle]
extern "C" fn kmain() {}
