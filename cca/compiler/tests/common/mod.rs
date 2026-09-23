//! What the test files share: doing independent work on the cores this
//! session has.
//!
//! Each test walks a folder of programs, and every program is its own
//! job — a build, a run, a proof. Walking them one after another left all
//! but one core idle while the slowest test set the pace (owner: optimised
//! everywhere, the tooling too). The machine is shared, though: three
//! sessions run heavy work on its 10 cores, and each has 3, with one left
//! for the desktop (owner, 2026-09-23). So there is one limit for the whole
//! test process, not one per test — two tests side by side do not make 6.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};

/// How many at once: this session's share, 3, or `CCA_TEST_JOBS`.
pub fn jobs() -> usize {
    std::env::var("CCA_TEST_JOBS").ok().and_then(|v| v.parse().ok()).filter(|n: &usize| *n > 0).unwrap_or(3)
}

static RUNNING: Mutex<usize> = Mutex::new(0);
static FREED: Condvar = Condvar::new();

/// Run `f` once one of the process's `jobs()` slots is free.
fn in_a_slot<R>(f: impl FnOnce() -> R) -> R {
    let mut n = RUNNING.lock().unwrap();
    while *n >= jobs() {
        n = FREED.wait(n).unwrap();
    }
    *n += 1;
    drop(n);
    // Given back even if `f` panics, or every other test would wait forever.
    struct Slot;
    impl Drop for Slot {
        fn drop(&mut self) {
            *RUNNING.lock().unwrap() -= 1;
            FREED.notify_one();
        }
    }
    let _slot = Slot;
    f()
}

/// `f` over every item, `jobs()` at a time across the whole process,
/// answers in the items' order. Threads take the next item as they come
/// free, so one slow program does not hold up a whole share.
pub fn par_map<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let next = AtomicUsize::new(0);
    let n = jobs().min(items.len()).max(1);
    let mut out: Vec<(usize, R)> = std::thread::scope(|s| {
        let workers: Vec<_> = (0..n)
            .map(|_| {
                // Precog recurses through the program; a deep one wants more
                // stack than a spawned thread's 2 MiB.
                std::thread::Builder::new()
                    .stack_size(64 << 20)
                    .spawn_scoped(s, || {
                        let mut mine = Vec::new();
                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            if i >= items.len() {
                                break;
                            }
                            mine.push((i, in_a_slot(|| f(&items[i]))));
                        }
                        mine
                    })
                    .unwrap()
            })
            .collect();
        workers.into_iter().flat_map(|w| w.join().unwrap()).collect()
    });
    out.sort_by_key(|(i, _)| *i);
    out.into_iter().map(|(_, r)| r).collect()
}
