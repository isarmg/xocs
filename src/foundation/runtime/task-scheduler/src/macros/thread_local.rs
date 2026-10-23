#[cfg(test)]
include!("../../../../../../test/foundation/runtime/task-scheduler/unit/src/macros/thread_local/0.rs");

#[cfg(not(all(loom, test)))]
macro_rules! tokio_thread_local {
    ($($tts:tt)+) => {
        ::std::thread_local!{ $($tts)+ }
    }
}
