#![no_std]
#![no_main]

use core::arch::asm;
use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!(concat!(env!("OUT_DIRX"), "/global.S")));

#[panic_handler]
fn panic_handler(_panic_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() {
    unsafe {
        asm! {
            "nop",
            options(nostack, nomem),
        }
    }
}
