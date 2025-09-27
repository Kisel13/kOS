use core::sync::atomic::{AtomicBool, Ordering};
use crossbeam_queue::ArrayQueue;
use futures_util::task::AtomicWaker;
use pc_keyboard::{Keyboard, ScancodeSet1, layouts, HandleControl, DecodedKey};

use alloc::sync::Arc;
use spin::Mutex;

/// Driver event
#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    KeyPress(DecodedKey),
    KeyRelease(DecodedKey),
}

/// Global driver state
pub struct KeyboardDriver {
    keyboard: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>>,
    queue: Arc<ArrayQueue<KeyboardEvent>>,
    waker: AtomicWaker,
    initialized: AtomicBool,
}

impl KeyboardDriver {
    pub fn new() -> Self {
        Self {
            keyboard: Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore,
            )),
            queue: Arc::new(ArrayQueue::new(256)),
            waker: AtomicWaker::new(),
            initialized: AtomicBool::new(false),
        }
    }

    /// PS/2 init
    pub fn init(&self) {
        if self.initialized.swap(true, Ordering::SeqCst) {
            return;
        }

        unsafe {
            use x86_64::instructions::port::Port;
            let mut cmd = Port::<u8>::new(0x64);
            let mut data = Port::<u8>::new(0x60);

            // Cleaning buffer
            while (cmd.read() & 1) != 0 {
                let _: u8 = data.read();
            }

            // Enabling keyboard interruption
            cmd.write(0xAEu8); // enable first PS/2 port
            data.write(0xF4u8); // enable scanning
        }
    }

    /// Scancode IRQ-handler
    pub fn handle_scancode(&self, scancode: u8) {
        let mut kb = self.keyboard.lock();
        if let Ok(Some(event)) = kb.add_byte(scancode) {
            if let Some(decoded) = kb.process_keyevent(event) {
                let ev = KeyboardEvent::KeyPress(decoded); // TODO: KeyRelease
                if let Err(_) = self.queue.push(ev) {
                    // Queue overflow
                    println!((Color::Yellow, Color::Black), "WARNING: Keyboard queue overflow");
                } else {
                    self.waker.wake();
                }
            }
        }
    }
    
    /// Get event queue (for async Stream)
    pub fn queue(&self) -> Arc<ArrayQueue<KeyboardEvent>> {
        self.queue.clone()
    }
    
    pub fn register_waker(&self, waker: &core::task::Waker) {
        self.waker.register(waker);
    }
}

/// Global driver
use conquer_once::spin::OnceCell;
static DRIVER: OnceCell<KeyboardDriver> = OnceCell::uninit();

pub fn init_keyboard() {
    let driver = KeyboardDriver::new();
    driver.init();
    DRIVER.init_once(move || driver);
}

pub fn add_scancode_from_irq(scancode: u8) {
    if let Ok(driver) = DRIVER.try_get() {
        driver.handle_scancode(scancode);
    }
}

/// Async keyboard events stream
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_util::stream::Stream;

use crate::drivers::tty::Color;
use crate::{println};

pub struct KeyboardStream {
    queue: Arc<ArrayQueue<KeyboardEvent>>,
}

impl KeyboardStream {
    pub fn new() -> Self {
        let driver = DRIVER.try_get().expect("keyboard not initialized");
        Self { queue: driver.queue() }
    }
}

impl Stream for KeyboardStream {
    type Item = KeyboardEvent;
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(event) = self.queue.pop() {
            return Poll::Ready(Some(event));
        }
        
        if let Ok(driver) = DRIVER.try_get() {
            driver.register_waker(cx.waker());
        }
        
        match self.queue.pop() {
            Some(ev) => Poll::Ready(Some(ev)),
            None => Poll::Pending,
        }
    }
}

use x86_64::structures::idt::InterruptStackFrame;
use crate::interrupts::PICS;
use crate::interrupts::InterruptIndex;

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::keyboard::add_scancode_from_irq(scancode);

    unsafe {
        PICS.lock()
        .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}