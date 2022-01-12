use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub mod simple_executor;
pub mod keyboard;

// newtype 包装一个  Future，无返回值
pub struct Task {
    future: Pin<Box<dyn Future<Output=()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output=()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }

    // 包装内部的 poll
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}