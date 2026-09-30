//! Parallel `for`'s slices (`parallelism.md`): "a thread pool — one thread
//! per core, created at startup, idle when unused — and a scheduler that
//! hands them slices of a parallelised loop". Created at the first loop
//! that asks, so a program without one never starts a thread.
//!
//! A loop is `trips` iterations; it is cut into as many slices as there are
//! threads (the caller is one of them), at most `MAX_SLICES`, and each slice
//! runs `body(env, first, end, slice)` for the iterations `first..end`.
//! A loop started while another runs — one inside another's slice, or from
//! a second task — runs inline, in one slice, on the thread that asked.
//!
//! `CYBORG_THREADS` in the environment caps the threads (the machine's
//! cores otherwise): how many cores a program may take is the person
//! running it's to say, never the program's.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Must agree with `par::MAX_SLICES` in the compiler: the partials a fold
/// keeps, one per slice.
const MAX_SLICES: u64 = 64;

type Body = extern "C" fn(env: *mut u8, first: i64, end: i64, slice: i64);

#[repr(C, align(16))]
struct Opaque([u64; 16]);

extern "C" {
    fn pthread_create(t: *mut u64, attr: *const u8, f: extern "C" fn(*mut u8) -> *mut u8, arg: *mut u8) -> i32;
    fn pthread_attr_init(a: *mut Opaque) -> i32;
    fn pthread_attr_setstacksize(a: *mut Opaque, size: usize) -> i32;
    fn pthread_mutex_init(m: *mut Opaque, attr: *const u8) -> i32;
    fn pthread_mutex_lock(m: *mut Opaque) -> i32;
    fn pthread_mutex_unlock(m: *mut Opaque) -> i32;
    fn pthread_cond_init(c: *mut Opaque, attr: *const u8) -> i32;
    fn pthread_cond_wait(c: *mut Opaque, m: *mut Opaque) -> i32;
    fn pthread_cond_broadcast(c: *mut Opaque) -> i32;
    fn sysconf(name: i32) -> i64;
    fn getenv(name: *const u8) -> *const u8;
    fn sched_yield() -> i32;
}

#[cfg(target_os = "macos")]
const NPROCESSORS_ONLN: i32 = 58;
#[cfg(target_os = "linux")]
const NPROCESSORS_ONLN: i32 = 84;

struct Pool {
    mutex: UnsafeCell<Opaque>,
    start: UnsafeCell<Opaque>,
    /// Set once: how many threads, the caller included.
    threads: UnsafeCell<u64>,
    /// The job, read by a worker under the mutex when `generation` moves.
    body: UnsafeCell<Option<Body>>,
    env: UnsafeCell<*mut u8>,
    trips: UnsafeCell<u64>,
    slices: UnsafeCell<u64>,
    generation: UnsafeCell<u64>,
}

unsafe impl Sync for Pool {}

static POOL: Pool = Pool {
    mutex: UnsafeCell::new(Opaque([0; 16])),
    start: UnsafeCell::new(Opaque([0; 16])),
    threads: UnsafeCell::new(0),
    body: UnsafeCell::new(None),
    env: UnsafeCell::new(core::ptr::null_mut()),
    trips: UnsafeCell::new(0),
    slices: UnsafeCell::new(0),
    generation: UnsafeCell::new(0),
};

/// The next slice to hand out: the job's generation in the high 32 bits,
/// the slice in the low. A worker late from one loop can never take a
/// slice of the next with the last one's body.
static NEXT: AtomicU64 = AtomicU64::new(0);
/// Slices not yet finished.
static REMAINING: AtomicU64 = AtomicU64::new(0);
/// A loop is being sliced now.
static BUSY: AtomicBool = AtomicBool::new(false);
static STARTED: AtomicBool = AtomicBool::new(false);

unsafe fn thread_count() -> u64 {
    let v = getenv(b"CYBORG_THREADS\0".as_ptr());
    if !v.is_null() {
        let mut n: u64 = 0;
        let mut p = v;
        while (*p).is_ascii_digit() {
            n = n.saturating_mul(10).saturating_add((*p - b'0') as u64);
            p = p.add(1);
        }
        if n >= 1 {
            return n.min(MAX_SLICES);
        }
    }
    let n = sysconf(NPROCESSORS_ONLN);
    if n < 1 { 1 } else { (n as u64).min(MAX_SLICES) }
}

/// The iterations of slice `k` of `slices`: an even share, the remainder
/// spread over the first ones.
fn bounds(k: u64, slices: u64, trips: u64) -> (i64, i64) {
    let first = (k as u128 * trips as u128 / slices as u128) as i64;
    let end = ((k + 1) as u128 * trips as u128 / slices as u128) as i64;
    (first, end)
}

/// Take slices of the job `gen` until there are none left.
unsafe fn run_slices(gen: u64, body: Body, env: *mut u8, trips: u64, slices: u64) {
    loop {
        let cur = NEXT.load(Ordering::Acquire);
        if cur >> 32 != gen & 0xffff_ffff || (cur & 0xffff_ffff) >= slices {
            return;
        }
        if NEXT.compare_exchange(cur, cur + 1, Ordering::AcqRel, Ordering::Acquire).is_err() {
            continue;
        }
        let k = cur & 0xffff_ffff;
        let (first, end) = bounds(k, slices, trips);
        body(env, first, end, k as i64);
        REMAINING.fetch_sub(1, Ordering::AcqRel);
    }
}

extern "C" fn worker(_: *mut u8) -> *mut u8 {
    unsafe {
        let mut seen: u64 = 0;
        loop {
            pthread_mutex_lock(POOL.mutex.get());
            while *POOL.generation.get() == seen {
                pthread_cond_wait(POOL.start.get(), POOL.mutex.get());
            }
            seen = *POOL.generation.get();
            let body = *POOL.body.get();
            let env = *POOL.env.get();
            let trips = *POOL.trips.get();
            let slices = *POOL.slices.get();
            pthread_mutex_unlock(POOL.mutex.get());
            if let Some(b) = body {
                run_slices(seen, b, env, trips, slices);
            }
        }
    }
}

unsafe fn start() {
    let threads = thread_count();
    *POOL.threads.get() = threads;
    pthread_mutex_init(POOL.mutex.get(), core::ptr::null());
    pthread_cond_init(POOL.start.get(), core::ptr::null());
    let mut attr = Opaque([0; 16]);
    pthread_attr_init(&mut attr);
    // As much stack as the program's main thread: a slice runs the same
    // code, recursion and all.
    pthread_attr_setstacksize(&mut attr, 8 << 20);
    for _ in 1..threads {
        let mut t: u64 = 0;
        pthread_create(&mut t, &attr as *const Opaque as *const u8, worker, core::ptr::null_mut());
    }
}

/// Run `trips` iterations of a loop in slices; how many slices it used,
/// so the caller combines that many partials of each fold.
///
/// # Safety
/// `body` must be safe to run on many threads at once with `env`, each
/// slice on its own iterations: the compiler's proof (`par.rs`).
#[no_mangle]
pub unsafe extern "C" fn cyborg_par_for(body: Body, env: *mut u8, trips: i64) -> i64 {
    let trips = trips.max(0) as u64;
    if trips == 0 {
        return 0;
    }
    if BUSY.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
        body(env, 0, trips as i64, 0);
        return 1;
    }
    if !STARTED.swap(true, Ordering::AcqRel) {
        start();
    }
    let threads = *POOL.threads.get();
    let slices = threads.min(trips).min(MAX_SLICES).max(1);
    if slices == 1 {
        body(env, 0, trips as i64, 0);
        BUSY.store(false, Ordering::Release);
        return 1;
    }
    pthread_mutex_lock(POOL.mutex.get());
    let gen = *POOL.generation.get() + 1;
    *POOL.generation.get() = gen;
    *POOL.body.get() = Some(body);
    *POOL.env.get() = env;
    *POOL.trips.get() = trips;
    *POOL.slices.get() = slices;
    REMAINING.store(slices, Ordering::Release);
    NEXT.store((gen & 0xffff_ffff) << 32, Ordering::Release);
    pthread_cond_broadcast(POOL.start.get());
    pthread_mutex_unlock(POOL.mutex.get());
    run_slices(gen, body, env, trips, slices);
    while REMAINING.load(Ordering::Acquire) != 0 {
        sched_yield();
    }
    BUSY.store(false, Ordering::Release);
    slices as i64
}
