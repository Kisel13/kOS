#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{BootInfo, entry_point};

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
mod tests {
    extern crate alloc;
    use core::ptr;
    use alloc::sync::Arc;
    use alloc::{vec, vec::Vec};
    use kos::task::{Task, executor::Executor};
    use kos::drivers::blockdev::{BlockOp, RequestQueue, BlockRequest, ReqResult};
    use core::panic::PanicInfo;

    fn spawn_block_task<F>(executor: &mut Executor, fut: F)
    where
        F: core::future::Future<Output = ()> + 'static,
    {
        executor.spawn(Task::new(fut));
    }

    #[test_case]
    fn basic_submit_and_complete() {
        let mut executor = Executor::new();
        let queue = Arc::new(RequestQueue::new());
        let mut buf = [0u8; 512];
        let req = Arc::new(BlockRequest::new(BlockOp::Read, 0, 1, buf.as_mut_ptr(), buf.len()));

        let result_holder = Arc::new(spin::Mutex::new(None));
        let result_clone = result_holder.clone();
        let queue_clone = queue.clone();

        let future = queue_clone.submit(req.clone());
        spawn_block_task(&mut executor, async move {
            let res = future.await;
            *result_clone.lock() = Some(res);
        });

        // Worker ends request
        if let Some(r) = queue.pop_one() {
            unsafe { ptr::write_bytes(r.buf, 0xAB, r.buf_len); }
            r.complete(r.blocks as i32);
        }

        while result_holder.lock().as_ref().is_none() {
            executor.run_ready_once();
        }

        assert_eq!(buf[0], 0xAB);
        assert_eq!(*result_holder.lock(), Some(1));
    }

    #[test_case]
    fn multiple_requests() {
        let mut executor = Executor::new();
        let queue = Arc::new(RequestQueue::new());
        let mut bufs: Vec<[u8; 512]> = vec![[0; 512]; 3];
        let mut results: Vec<Arc<spin::Mutex<Option<ReqResult>>>> = vec![];
        let mut requests: Vec<Arc<BlockRequest>> = vec![];

        for (i, buf) in bufs.iter_mut().enumerate() {
            let req = Arc::new(BlockRequest::new(BlockOp::Read, i as u64, 1, buf.as_mut_ptr(), buf.len()));
            requests.push(req.clone());
            let result = Arc::new(spin::Mutex::new(None));
            results.push(result.clone());

            let queue_clone = queue.clone();
            spawn_block_task(&mut executor, async move {
                let r = queue_clone.submit(req.clone()).await;
                *result.lock() = Some(r);
            });
        }

        // Worker ends all requests
        for r in requests.iter() {
            unsafe { ptr::write_bytes(r.buf, r.id as u8, r.buf_len); }
            r.complete(r.blocks as i32);
        }

        while results.iter().any(|r| r.lock().as_ref().is_none()) {
            executor.run_ready_once();
        }

        // Check
        for (i, buf) in bufs.iter().enumerate() {
            assert_eq!(buf[0], (i as u8 + 2) as u8);
            assert_eq!(results[i].lock().as_ref().unwrap(), &1);
        }
    }

    #[test_case]
    fn queue_empty_behavior() {
        let queue = RequestQueue::new();
        assert!(queue.pop_one().is_none());
        let drained = queue.drain_all();
        assert!(drained.is_empty());
    }

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        kos::test_panic_handler(info)
    }
}
