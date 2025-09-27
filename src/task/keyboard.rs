use futures_util::StreamExt;
use crate::{print};
use crate::drivers::keyboard::{KeyboardEvent, KeyboardStream};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream},
    task::AtomicWaker,
};
use pc_keyboard::{DecodedKey};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // fast path
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn print_keypresses() {
    let mut events = KeyboardStream::new();

    while let Some(event) = events.next().await {
        match event {
            KeyboardEvent::KeyPress(decoded) => match decoded {
                DecodedKey::Unicode(c) => print!("{}", c),
                DecodedKey::RawKey(k) => print!("{:?}", k),
            },
            KeyboardEvent::KeyRelease(decoded) => match decoded {
                DecodedKey::Unicode(c) => {
                    // обычно отпуск клавиши можно игнорировать или выводить в дебаг
                    print!("<Release {}>", c);
                }
                DecodedKey::RawKey(k) => print!("<Release {:?}>", k),
            },
        }
    }
}