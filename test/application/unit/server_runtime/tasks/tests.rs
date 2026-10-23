use super::*;
use futures::poll;
use std::task::Poll;

#[tokio::test]
async fn admitted_request_can_register_commit_before_drain_closes_registration() {
    let scope = WorkScope::new();
    let request = scope.enter_request().await.unwrap();
    let mut drain = Box::pin(scope.drain_requests());
    assert!(matches!(poll!(&mut drain), Poll::Pending));
    assert!(scope.enter_request().await.is_none());
    let (finish, pending) = tokio::sync::oneshot::channel::<()>();
    scope.commit_tasks.spawn(async {
        pending.await.unwrap();
    });
    drop(request);
    drain.await;
    let mut commits = Box::pin(scope.drain_commits());
    assert!(matches!(poll!(&mut commits), Poll::Pending));
    assert!(scope.commit_tasks.try_spawn(async {}).is_err());
    assert!(scope.work_tasks.try_spawn(async {}).is_err());
    finish.send(()).unwrap();
    commits.await;
}

#[tokio::test]
async fn cancelling_waiter_does_not_release_running_blocking_work_resources() {
    let work = TrackedTasks::new();
    let slots = Arc::new(tokio::sync::Semaphore::new(1));
    let permit = slots.clone().acquire_owned().await.unwrap();
    let (started, entered) = tokio::sync::oneshot::channel();
    let (release, blocked) = std::sync::mpsc::channel();
    let waiter = work.spawn(async move {
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            started.send(()).unwrap();
            blocked.recv().unwrap();
        })
        .await
        .unwrap();
    });
    entered.await.unwrap();
    drop(waiter);
    work.close();
    assert_eq!(slots.available_permits(), 0);
    assert_eq!(work.len(), 1);
    release.send(()).unwrap();
    work.wait().await;
    assert_eq!(slots.available_permits(), 1);
}
