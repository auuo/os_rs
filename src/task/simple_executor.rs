use super::Task;
use alloc::collections::VecDeque;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> Self {
        Self {
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {}, // 任务结束
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }
}

/// RawWaker 要求我们定义一个虚表指定当 RawWaker 被克隆，唤醒或者 drop 时应该调用什么函数
/// 每个函数接收一个 `*const` 类型的参数，这个参数的值是使用 RawWaker::new 时传递的值
/// 使用 Box::into_raw 可以创建一个 `*const T` 类型的指针，但是调用者需要负责内存的释放（可以使用 Box::from_raw）
fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        // 使用当前方法创建一个新的
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}