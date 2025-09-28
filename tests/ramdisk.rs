#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use kos::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    kos::init(&mut mapper, &mut frame_allocator);

    test_main();
    loop {}
}

#[cfg(test)]
mod ramdisk_blockdevice_tests {
    extern crate alloc;

    use kos::drivers::ramdisk::{create_ramdisk};
    use kos::drivers::blockdev::BlockDevice;
    use kos::drivers::ramdisk::RamDiskDevice;

    #[test_case]
    fn ramdisk_basic_read_write() {
        let rd = create_ramdisk(1024);
        let dev = RamDiskDevice::new(rd.clone());

        let data = [0xABu8; 512];
        dev.write_sector(0, &data).unwrap();

        let mut buf = [0u8; 512];
        dev.read_sector(0, &mut buf).unwrap();

        assert_eq!(buf, data);
    }

    #[test_case]
    fn ramdisk_fill_from_bytes() {
        let rd = create_ramdisk(1024);
        let dev = RamDiskDevice::new(rd.clone());

        rd.lock().fill_from_bytes(b"HelloWorld");

        let mut buf = [0u8; 512];
        dev.read_sector(0, &mut buf).unwrap();

        assert_eq!(&buf[..10], b"HelloWorld");
    }

    #[test_case]
    fn ramdisk_multiple_blocks() {
        let rd = create_ramdisk(2048); // 4 512b blocks
        let dev = RamDiskDevice::new(rd.clone());

        dev.write_sector(0, &[1u8; 512]).unwrap();
        dev.write_sector(1, &[2u8; 512]).unwrap();

        let mut buf0 = [0u8; 512];
        let mut buf1 = [0u8; 512];
        dev.read_sector(0, &mut buf0).unwrap();
        dev.read_sector(1, &mut buf1).unwrap();

        assert_eq!(buf0[0], 1);
        assert_eq!(buf1[0], 2);
    }

    #[test_case]
    fn ramdisk_out_of_range() {
        let rd = create_ramdisk(1024); // 
        let dev = RamDiskDevice::new(rd.clone());

        let mut buf = [0u8; 512];

        let res = dev.read_sector(2, &mut buf);
        assert!(res.is_err());

        let res = dev.write_sector(2, &buf);
        assert!(res.is_err());
    }

    #[test_case]
    fn ramdisk_misaligned_buffer() {
        let rd = create_ramdisk(1024);
        let dev = RamDiskDevice::new(rd.clone());

        let mut small_buf = [0u8; 10];
        let res = dev.read_sector(0, &mut small_buf);
        assert!(res.is_err());

        let data = [0u8; 10];
        let res = dev.write_sector(0, &data);
        assert!(res.is_err());
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kos::test_panic_handler(info)
}
