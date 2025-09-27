use alloc::{sync::Arc, vec::Vec};
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicI32, AtomicU8, AtomicU64, Ordering},
    task::{Context, Poll, Waker},
};
use spin::Mutex;
use crate::{println};

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy)]
pub enum BlockOp {
    Read,
    Write,
}

pub type ReqResult = i32;

pub struct BlockRequest {
    pub id: u64,
    pub op: BlockOp,
    pub lba: u64,
    pub blocks: u32,
    pub buf: *mut u8,
    pub buf_len: usize,

    state: AtomicU8,      // 0 = pending, 1 = completed
    result: AtomicI32,    // Operation result
    waker: Mutex<Option<Waker>>,
}

impl BlockRequest {
    pub fn new(op: BlockOp, lba: u64, blocks: u32, buf: *mut u8, buf_len: usize) -> Self {
        let id = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        println!("[BlockRequest::new] id={}, op={:?}, lba={}, blocks={}, buf_len={}", id, op, lba, blocks, buf_len);
        Self {
            id,
            op,
            lba,
            blocks,
            buf,
            buf_len,
            state: AtomicU8::new(0),
            result: AtomicI32::new(-1),
            waker: Mutex::new(None),
        }
    }

    pub fn complete(&self, res: ReqResult) {
        println!("[BlockRequest::complete] id={}, lba={}, blocks={}, result={} (op={:?})", self.id, self.lba, self.blocks, res, self.op);
        self.result.store(res, Ordering::Release);
        self.state.store(1, Ordering::Release);
        if let Some(w) = self.waker.lock().take() {
            println!("[BlockRequest::complete] id={} waking future", self.id);
            w.wake();
        }
    }

    pub fn try_result(&self) -> Option<ReqResult> {
        let st = self.state.load(Ordering::Acquire);
        println!("[BlockRequest::try_result] id={}, lba={}, state={}", self.id, self.lba, st);
        if st == 1 {
            Some(self.result.load(Ordering::Acquire))
        } else {
            None
        }
    }
}

pub struct RequestFuture {
    req: Arc<BlockRequest>,
}

impl RequestFuture {
    pub fn new(req: Arc<BlockRequest>) -> Self {
        println!("[RequestFuture::new] created future id={} for lba={}, blocks={} (op={:?})", req.id, req.lba, req.blocks, req.op);
        Self { req }
    }
}

impl Future for RequestFuture {
    type Output = ReqResult;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("[RequestFuture::poll] id={} polling lba={}, blocks={}", self.req.id, self.req.lba, self.req.blocks);
        if let Some(r) = self.req.try_result() {
            println!("[RequestFuture::poll] id={} ready result={}", self.req.id, r);
            return Poll::Ready(r);
        }
        let mut w = self.req.waker.lock();
        *w = Some(cx.waker().clone());
        if let Some(r) = self.req.try_result() {
            println!("[RequestFuture::poll] id={} result appeared after waker set: {}", self.req.id, r);
            w.take();
            return Poll::Ready(r);
        }
        println!("[RequestFuture::poll] id={} pending", self.req.id);
        Poll::Pending
    }
}

pub struct RequestQueue {
    inner: Mutex<Vec<Arc<BlockRequest>>>,
}

impl RequestQueue {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(Vec::new()),
        }
    }

    pub fn submit(&self, req: Arc<BlockRequest>) -> RequestFuture {
        println!("[RequestQueue::submit] pushing request id={}, lba={}, blocks={} (op={:?})", req.id, req.lba, req.blocks, req.op);
        {
            let mut q = self.inner.lock();
            q.push(req.clone());
        }
        RequestFuture::new(req)
    }

    pub fn drain_all(&self) -> Vec<Arc<BlockRequest>> {
        println!("[RequestQueue::drain_all] draining queue");
        let mut q = self.inner.lock();
        let mut out = Vec::new();
        core::mem::swap(&mut out, &mut *q);
        println!("[RequestQueue::drain_all] drained {} requests", out.len());
        out
    }

    pub fn pop_one(&self) -> Option<Arc<BlockRequest>> {
        let mut q = self.inner.lock();
        if q.is_empty() {
            println!("[RequestQueue::pop_one] queue empty");
            None
        } else {
            let r = q.remove(0);
            println!("[RequestQueue::pop_one] popped id={}, lba={}, blocks={} (op={:?})", r.id, r.lba, r.blocks, r.op);
            Some(r)
        }
    }
}