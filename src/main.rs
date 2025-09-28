#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use kos::task::{keyboard, Task};
use kos::{println};
use kos::task::{executor::Executor};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use kos::drivers::tty::Color;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use kos::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello World!");
    
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
     
    kos::init(&mut mapper, &mut frame_allocator);

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!((Color::Red, Color::Black), "{}", info);
    kos::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kos::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
