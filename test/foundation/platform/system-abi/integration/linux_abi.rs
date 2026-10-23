#![cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]

use blog_platform_system_abi as libc;
use std::{
    fs,
    mem::{align_of, offset_of, size_of},
    process::Command,
};

#[test]
fn linux_amd64_bindings_match_system_c_headers() {
    let dir = std::env::temp_dir().join(format!("xocs-linux-abi-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(dir.clone());
    let source = dir.join("abi.c");
    let executable = dir.join("abi");
    fs::write(
        &source,
        r#"
#define _GNU_SOURCE
#include <stddef.h>
#include <stdio.h>
#include <pthread.h>
#include <sys/epoll.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include <time.h>
#define LAYOUT(T) printf("%zu %zu ", sizeof(T), _Alignof(T))
int main(void) {
    LAYOUT(pthread_mutex_t); LAYOUT(pthread_cond_t); LAYOUT(pthread_rwlock_t);
    LAYOUT(struct stat); LAYOUT(struct statvfs); LAYOUT(struct sockaddr_storage);
    LAYOUT(struct msghdr); LAYOUT(struct epoll_event); LAYOUT(struct timespec);
    printf("%zu %zu %zu %zu", offsetof(struct stat, st_size),
        offsetof(struct stat, st_mtim), offsetof(struct epoll_event, data),
        offsetof(struct msghdr, msg_iov));
}
"#,
    )
    .unwrap();
    let compile = Command::new("cc")
        .arg("-std=c11")
        .arg(&source)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("the native build requires a C compiler");
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let output = Command::new(&executable).output().unwrap();
    assert!(output.status.success());
    let actual: Vec<usize> = String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect();
    let mut expected = Vec::new();
    macro_rules! layout {
        ($ty:ty) => {
            expected.extend([size_of::<$ty>(), align_of::<$ty>()]);
        };
    }
    layout!(libc::pthread_mutex_t);
    layout!(libc::pthread_cond_t);
    layout!(libc::pthread_rwlock_t);
    layout!(libc::stat);
    layout!(libc::statvfs);
    layout!(libc::sockaddr_storage);
    layout!(libc::msghdr);
    layout!(libc::epoll_event);
    layout!(libc::timespec);
    expected.extend([
        offset_of!(libc::stat, st_size),
        offset_of!(libc::stat, st_mtime),
        offset_of!(libc::epoll_event, u64),
        offset_of!(libc::msghdr, msg_iov),
    ]);
    assert_eq!(actual, expected);
}
