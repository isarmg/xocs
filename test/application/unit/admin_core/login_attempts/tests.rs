use super::*;

#[tokio::test(start_paused = true)]
async fn admission_is_bounded_and_rejections_do_not_extend_window() {
    let state = AuthenticationState::new();
    for _ in 0..LOGIN_ATTEMPTS_MAX {
        assert_eq!(state.admit_login_attempt(), Ok(()));
    }
    assert_eq!(
        state.admit_login_attempt(),
        Err(AuthenticationStateError::LoginAttemptsExceeded {
            retry_after_seconds: 60
        })
    );
    tokio::time::advance(std::time::Duration::from_millis(59_500)).await;
    assert_eq!(
        state.admit_login_attempt(),
        Err(AuthenticationStateError::LoginAttemptsExceeded {
            retry_after_seconds: 1
        })
    );
    assert!(!state.can_retire());
    tokio::time::advance(std::time::Duration::from_millis(500)).await;
    assert!(state.can_retire());
    assert_eq!(state.admit_login_attempt(), Ok(()));
    assert!(!state.can_retire());
}

impl AuthenticationState {
    pub(crate) fn elapse_window_for_test(&self) {
        if let Some(window) = self.window.lock().unwrap().as_mut() {
            window.started_at -= LOGIN_ATTEMPTS_WINDOW;
        }
    }
}
