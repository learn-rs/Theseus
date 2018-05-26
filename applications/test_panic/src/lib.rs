#![no_std]
#![feature(alloc)]
// #![feature(plugin)]
// #![plugin(application_main_fn)]


extern crate alloc;
#[macro_use] extern crate log;
#[macro_use] extern crate console;
extern crate panic; 
extern crate task;


use alloc::{Vec, String};
use alloc::boxed::Box;
use panic::PanicInfo;


#[no_mangle]
pub fn main(_args: Vec<String>) -> isize {
    info!("test_panic::main(): at top");
    if let Some(t) = task::get_my_current_task() {
        t.write().register_panic_handler(Box::new(panic_handler));
    }

    info!("test_panic::main(): registered panic handler. Calling panic...");

    panic!("yo i'm testing a panic!!");

    info!("test_panic::main(): after panic");

    0
}


fn panic_handler(info: &PanicInfo) {
    println!("In test_panic::panic_handler:  {:?}", info);
}