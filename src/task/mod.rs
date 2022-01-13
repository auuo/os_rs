use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll};

pub mod simple_executor;
pub mod executor;
pub mod keyboard;

/// 为每个 spawn 的 task 分配一个 taskId，用于唤醒时定位
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

// 包装一个  Future，无返回值
pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output=()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output=()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    // 包装内部的 poll
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}