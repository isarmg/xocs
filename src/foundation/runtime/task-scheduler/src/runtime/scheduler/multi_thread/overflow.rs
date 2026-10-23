use crate::runtime::task;

#[cfg(test)]
include!("../../../../../../../../test/foundation/runtime/task-scheduler/unit/src/runtime/scheduler/multi_thread/overflow/27.rs");

pub(crate) trait Overflow<T: 'static> {
    fn push(&self, task: task::Notified<T>);

    fn push_batch<I>(&self, iter: I)
    where
        I: Iterator<Item = task::Notified<T>>;
}

#[cfg(test)]
include!("../../../../../../../../test/foundation/runtime/task-scheduler/unit/src/runtime/scheduler/multi_thread/overflow/248.rs");
