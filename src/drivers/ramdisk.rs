use core::{sync::atomic::AtomicUsize, u8};
use alloc::sync::Arc;
use alloc::vec;
use alloc::boxed::Box;
use spin::Mutex;
use crate::drivers::blockdev::BlockDevice;

pub struct RamDiskDevice {
    ramdisk: Arc<Mutex<RamDisk>>,
}

impl RamDiskDevice {
    pub fn new(ramdisk: Arc<Mutex<RamDisk>>) -> Self {
        Self { ramdisk }
    }
}

impl BlockDevice for RamDiskDevice {
    fn read_sector(&self, lba: u64, buf: &mut [u8]) -> Result<(), ()> {
        let ramdisk = self.ramdisk.lock();
        ramdisk.read_block(lba as usize, buf).map_err(|_| ())
    }

    fn write_sector(&self, lba: u64, buf: &[u8]) -> Result<(), ()> {
        let mut ramdisk = self.ramdisk.lock();
        ramdisk.write_block(lba as usize, buf).map_err(|_| ())
    }

    fn sector_size(&self) -> usize {
        let ramdisk = self.ramdisk.lock();
        ramdisk.block_size
    }
}

pub struct RamDisk {
    storage: &'static mut [u8],
    block_size: usize,
    blocks: usize,
    #[allow(dead_code)]
    write_pos: AtomicUsize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RamDiskError {
    OutOfRange,
    Misaligned,
}

impl RamDisk {
    /// Создать RAM-диск из статического массива
    pub fn new(storage: &'static mut [u8], block_size: usize) -> Self {
        let blocks = storage.len() / block_size;
        Self {
            storage,
            block_size,
            blocks,
            write_pos: AtomicUsize::new(0),
        }
    }

    /// Чтение блока
    pub fn read_block(&self, block_id: usize, buf: &mut [u8]) -> Result<(), RamDiskError> {
        let start = block_id * self.block_size;
        let end = start + self.block_size;

        if end > self.blocks * self.block_size {
            return Err(RamDiskError::OutOfRange);
        }
        if buf.len() < self.block_size {
            return Err(RamDiskError::Misaligned);
        }

        buf[..self.block_size].copy_from_slice(&self.storage[start..end]);
        Ok(())
    }

    /// Запись блока
    pub fn write_block(&mut self, block_id: usize, data: &[u8]) -> Result<(), RamDiskError> {
        let start = block_id * self.block_size;
        let end = start + self.block_size;

        if end > self.blocks * self.block_size {
            return Err(RamDiskError::OutOfRange);
        }
        if data.len() < self.block_size {
            return Err(RamDiskError::Misaligned);
        }

        self.storage[start..end].copy_from_slice(&data[..self.block_size]);
        Ok(())
    }

    /// Для теста: добавить данные из include_bytes
    pub fn fill_from_bytes(&mut self, data: &'static [u8]) {
        let len = self.storage.len().min(data.len());
        self.storage[..len].copy_from_slice(&data[..len]);
    }
}

/// Создание Arc<RamDisk> для тестов
pub fn create_ramdisk(size: usize) -> Arc<Mutex<RamDisk>> {
    // Преобразуем Box<[u8]> в &'static mut [u8] через into_raw
    let boxed: Box<[u8]> = vec![0u8; size].into_boxed_slice();
    let ptr = Box::into_raw(boxed);
    let buffer: &'static mut [u8] = unsafe { &mut *ptr };
    let rd = RamDisk::new(buffer, 512);
    Arc::new(Mutex::new(rd))
}