# Media service path clone allocation

`ServeDir` is cloned when the router dispatches requests. Its immutable base path
previously used `PathBuf`, so each clone allocated and copied the path. The base
now uses `Arc<Path>`; cloning shares that path while each request still owns its
independently constructed filesystem lookup path.

The isolated measurement below compares precisely this changed operation. It
excludes construction of both values and performs 100,000 clone/drop iterations
using the same nonempty path. An allocation counter delegates to Rust's system
allocator and counts allocation and reallocation calls during each loop.

Recorded on Linux x86_64 with Rust 1.99, optimized compilation:

```text
iterations=100000 before_allocations=100000 after_allocations=0 before_ns=941133 after_ns=335818
```

This demonstrates elimination of one allocation for each service base-path
clone. The elapsed times describe one isolated run; they do not establish an
HTTP throughput or latency improvement. Request parsing, pathname construction,
filesystem access and body streaming are outside this measurement. `Arc` still
uses an allocation at construction and atomic reference counting when cloned.

Save the following complete source as `/tmp/xocs-path-clone-bench.rs`:

```rust
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

struct Allocator;
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

// The counter does not allocate. All pointer/layout handling is delegated
// unchanged to the system allocator that owns the allocation.
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, size) }
    }
}

fn main() {
    let old = PathBuf::from("/var/lib/blog/service/content/media");
    let new: Arc<Path> = Arc::from(old.as_path());
    let n = 100_000;

    ALLOCS.store(0, Ordering::Relaxed);
    let start = Instant::now();
    for _ in 0..n {
        black_box(black_box(&old).clone());
    }
    let old_ns = start.elapsed().as_nanos();
    let old_allocs = ALLOCS.load(Ordering::Relaxed);

    ALLOCS.store(0, Ordering::Relaxed);
    let start = Instant::now();
    for _ in 0..n {
        black_box(black_box(&new).clone());
    }
    let new_ns = start.elapsed().as_nanos();
    let new_allocs = ALLOCS.load(Ordering::Relaxed);

    println!(
        "iterations={n} before_allocations={old_allocs} after_allocations={new_allocs} before_ns={old_ns} after_ns={new_ns}"
    );
}
```

Run with the repository's Rust 1.99 toolchain:

```sh
rustc --version
rustc --edition=2024 -O /tmp/xocs-path-clone-bench.rs -o /tmp/xocs-path-clone-bench
/tmp/xocs-path-clone-bench
```

The consumer regression in `src/server/tests/media_paths.rs` independently
checks a cloned service's Unicode response and rejection of encoded traversal,
backslashes and NUL. The existing media range tests cover `HEAD`, partial
responses and empty files. These behavior checks are separate from the
allocation measurement.
