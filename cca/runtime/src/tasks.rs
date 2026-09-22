//! Tasks: `concur` (`design/concurrency.md`).
//!
//! "Tasks run on the thread pool `parallelism.md` already has — as lighter
//! tasks multiplexed over it, so a thousand waiting connections do not cost
//! a thousand threads." So a task is not a thread. It is a stack of its own
//! and a saved stack pointer; a handful of worker threads — one per core —
//! take ready tasks off one queue and run them until they finish or wait.
//! A task that waits is *parked*: its worker switches back to the queue and
//! runs something else, and whoever the task was waiting for puts it back.
//!
//! Structured tasks (`concur { … }`) belong to a *group*: the block that
//! started them waits on the group when it ends. A detached task
//! (`concur.detached`) belongs to none, and the program ending ends it.
//!
//! Only MAIN, and programs that start tasks, pay for any of this: a program
//! with no `concur` never starts a worker.

use core::cell::UnsafeCell;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Storage for a pthread object whose size the platform decides: big and
/// aligned enough for every platform CCA builds for, and initialised by the
/// platform's own `*_init`, never by a constant.
#[repr(C, align(16))]
struct Opaque([u64; 16]);

extern "C" {
    fn pthread_create(t: *mut u64, attr: *const u8, f: extern "C" fn(*mut u8) -> *mut u8, arg: *mut u8) -> i32;
    fn pthread_mutex_init(m: *mut Opaque, attr: *const u8) -> i32;
    fn pthread_mutex_lock(m: *mut Opaque) -> i32;
    fn pthread_mutex_unlock(m: *mut Opaque) -> i32;
    fn pthread_cond_init(c: *mut Opaque, attr: *const u8) -> i32;
    fn pthread_cond_wait(c: *mut Opaque, m: *mut Opaque) -> i32;
    fn pthread_cond_signal(c: *mut Opaque) -> i32;
    fn pthread_key_create(k: *mut u64, destructor: *const u8) -> i32;
    fn pthread_setspecific(k: u64, v: *const u8) -> i32;
    fn pthread_getspecific(k: u64) -> *mut u8;
    fn sysconf(name: i32) -> i64;
    fn mmap(addr: *mut u8, len: usize, prot: i32, flags: i32, fd: i32, off: i64) -> *mut u8;
    fn munmap(addr: *mut u8, len: usize) -> i32;
    fn mprotect(addr: *mut u8, len: usize, prot: i32) -> i32;
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(code: i32) -> !;
}

#[cfg(target_os = "macos")]
mod sys {
    pub const NPROCESSORS_ONLN: i32 = 58;
    pub const MAP_PRIVATE: i32 = 0x0002;
    pub const MAP_ANON: i32 = 0x1000;
}
#[cfg(target_os = "linux")]
mod sys {
    pub const NPROCESSORS_ONLN: i32 = 84;
    pub const MAP_PRIVATE: i32 = 0x0002;
    pub const MAP_ANON: i32 = 0x0020;
}
const PROT_NONE: i32 = 0;
const PROT_READ_WRITE: i32 = 3;

/// How much stack a task gets. Virtual: only the pages it touches cost
/// memory, so this is a ceiling on recursion inside a task, not a price per
/// task. Provisional (`cca/README.md`).
const STACK: usize = 256 * 1024;
/// MAIN's stack when it runs as a task: what a main thread has on the
/// platforms CCA builds for, so a program does not lose recursion depth in
/// MAIN just because it also starts a task.
const MAIN_STACK: usize = 8 * 1024 * 1024;
/// Below the stack, never mapped: running off the end of a task's stack
/// stops the program instead of writing over whatever is next. A multiple
/// of every page size in use (4 KiB, 16 KiB).
const GUARD: usize = 64 * 1024;

// ------------------------------------------------------------ switching

// `cyborg_switch(save, to)`: store the running stack pointer in `*save`,
// continue on the stack `to` points at. Everything the calling convention
// says a callee must preserve is pushed first and popped on the other side,
// so a switch looks like an ordinary call that returns much later.

#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
core::arch::global_asm!(".global _cyborg_switch", "_cyborg_switch:", include_str!("switch_aarch64.s"), ".global _cyborg_task_start", "_cyborg_task_start:", "mov x0, x19", "bl _cyborg_task_main", "brk #0");
#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
core::arch::global_asm!(".global cyborg_switch", "cyborg_switch:", include_str!("switch_aarch64.s"), ".global cyborg_task_start", "cyborg_task_start:", "mov x0, x19", "bl cyborg_task_main", "brk #0");
#[cfg(all(target_arch = "x86_64", target_os = "macos"))]
core::arch::global_asm!(".global _cyborg_switch", "_cyborg_switch:", include_str!("switch_x86_64.s"), ".global _cyborg_task_start", "_cyborg_task_start:", "mov rdi, rbx", "call _cyborg_task_main", "ud2");
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
core::arch::global_asm!(".global cyborg_switch", "cyborg_switch:", include_str!("switch_x86_64.s"), ".global cyborg_task_start", "cyborg_task_start:", "mov rdi, rbx", "call cyborg_task_main", "ud2");

extern "C" {
    fn cyborg_switch(save: *mut *mut u8, to: *mut u8);
    fn cyborg_task_start();
}

// ------------------------------------------------------------ tasks

const RUNNING: u32 = 0;
/// Asked to wait, and not yet off its worker.
const PARKING: u32 = 1;
/// Off its worker, waiting for someone to put it back.
const PARKED: u32 = 2;
/// Put back, or about to be.
const READY: u32 = 3;
const DONE: u32 = 4;

#[repr(C)]
pub struct Task {
    /// The saved stack pointer while the task is not running.
    sp: *mut u8,
    /// Where the worker running it saved *its* stack pointer: the way back.
    back: *mut *mut u8,
    state: AtomicU32,
    map: *mut u8,
    map_len: usize,
    entry: unsafe extern "C" fn(*mut u8),
    arg: *mut u8,
    group: *mut Group,
    next: *mut Task,
}

/// The tasks a block started, so the block can wait for them when it ends.
/// It lives in the starting function's frame; `cyborg_group_init` sets it up.
#[repr(C)]
pub struct Group {
    lock: Spin,
    live: usize,
    waiter: *mut Task,
}

/// How much room a group needs. The compiler reserves this much, aligned
/// to 16, in the frame of every function that starts tasks.
pub const GROUP_BYTES: usize = 64;
const _: () = assert!(core::mem::size_of::<Group>() <= GROUP_BYTES);

/// A lock for a few instructions' worth of work: a group's count, a
/// channel's queue. A worker never sleeps holding one.
#[repr(C)]
pub struct Spin(AtomicBool);

impl Spin {
    pub fn lock(&self) {
        while self.0.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }
    }
    pub fn unlock(&self) {
        self.0.store(false, Ordering::Release);
    }
}

// ------------------------------------------------------------ the scheduler

struct Sched {
    lock: Opaque,
    wake: Opaque,
    head: *mut Task,
    tail: *mut Task,
    /// The thread-local slot saying which task a worker is running.
    key: u64,
    /// MAIN has finished: the real main thread can return.
    main_done: bool,
    main_lock: Opaque,
    main_wake: Opaque,
}

struct Global(UnsafeCell<Sched>);
unsafe impl Sync for Global {}

static SCHED: Global = Global(UnsafeCell::new(Sched {
    lock: Opaque([0; 16]),
    wake: Opaque([0; 16]),
    head: null_mut(),
    tail: null_mut(),
    key: 0,
    main_done: false,
    main_lock: Opaque([0; 16]),
    main_wake: Opaque([0; 16]),
}));

fn sched() -> *mut Sched {
    SCHED.0.get()
}

unsafe fn fail_hard(msg: &[u8]) -> ! {
    write(2, msg.as_ptr(), msg.len());
    exit(102)
}

/// Put a task on the queue and wake a worker for it.
unsafe fn push(t: *mut Task) {
    let s = sched();
    (*t).next = null_mut();
    pthread_mutex_lock(&mut (*s).lock);
    if (*s).tail.is_null() {
        (*s).head = t;
    } else {
        (*(*s).tail).next = t;
    }
    (*s).tail = t;
    pthread_cond_signal(&mut (*s).wake);
    pthread_mutex_unlock(&mut (*s).lock);
}

/// The next ready task; sleeps until there is one.
unsafe fn pop() -> *mut Task {
    let s = sched();
    pthread_mutex_lock(&mut (*s).lock);
    while (*s).head.is_null() {
        pthread_cond_wait(&mut (*s).wake, &mut (*s).lock);
    }
    let t = (*s).head;
    (*s).head = (*t).next;
    if (*s).head.is_null() {
        (*s).tail = null_mut();
    }
    pthread_mutex_unlock(&mut (*s).lock);
    t
}

extern "C" fn worker(_: *mut u8) -> *mut u8 {
    unsafe {
        let mut own_sp: *mut u8 = null_mut();
        loop {
            let t = pop();
            pthread_setspecific((*sched()).key, t as *const u8);
            (*t).back = &mut own_sp;
            (*t).state.store(RUNNING, Ordering::Release);
            cyborg_switch(&mut own_sp, (*t).sp);
            // The task stopped: finished, or waiting.
            match (*t).state.load(Ordering::Acquire) {
                DONE => {
                    munmap((*t).map, (*t).map_len);
                }
                PARKING => {
                    // If someone put it back while it was still getting off,
                    // it goes straight back on the queue.
                    if (*t).state.compare_exchange(PARKING, PARKED, Ordering::AcqRel, Ordering::Acquire).is_err() {
                        push(t);
                    }
                }
                _ => push(t),
            }
        }
    }
}

/// The task the calling worker is running.
unsafe fn current() -> *mut Task {
    pthread_getspecific((*sched()).key) as *mut Task
}

/// Stop running this task until `unpark` puts it back. The caller has
/// already set `PARKING` and made itself findable, under whatever lock the
/// waker takes.
unsafe fn park(t: *mut Task) {
    cyborg_switch(&mut (*t).sp, *(*t).back);
}

/// Put a parked (or parking) task back.
unsafe fn unpark(t: *mut Task) {
    if (*t).state.swap(READY, Ordering::AcqRel) == PARKED {
        push(t);
    }
    // Still PARKING: its worker sees READY when it gets back, and queues it.
}

/// Where a new task begins: the switch "returns" into `cyborg_task_start`,
/// which calls here with the task.
#[no_mangle]
pub unsafe extern "C" fn cyborg_task_main(t: *mut Task) -> ! {
    ((*t).entry)((*t).arg);
    if !(*t).group.is_null() {
        group_done((*t).group);
    }
    (*t).state.store(DONE, Ordering::Release);
    cyborg_switch(&mut (*t).sp, *(*t).back);
    fail_hard(b"a finished task was resumed\n")
}

/// A new task: its own stack, laid out so the first switch to it lands in
/// `cyborg_task_start` with the task in hand.
unsafe fn new_task(entry: unsafe extern "C" fn(*mut u8), arg: *mut u8, group: *mut Group, stack: usize) -> *mut Task {
    let len = GUARD + stack;
    let map = mmap(null_mut(), len, PROT_READ_WRITE, sys::MAP_PRIVATE | sys::MAP_ANON, -1, 0);
    if map as isize == -1 {
        fail_hard(b"out of memory: no room for another task's stack\n");
    }
    mprotect(map, GUARD, PROT_NONE);
    let top = map.add(len);
    // The task's own record sits at the top of its stack.
    let t = top.sub(core::mem::size_of::<Task>()).cast::<Task>();
    let t = ((t as usize) & !15) as *mut Task;
    t.write(Task { sp: null_mut(), back: null_mut(), state: AtomicU32::new(READY), map, map_len: len, entry, arg, group, next: null_mut() });
    (*t).sp = first_frame(t as *mut u8, t);
    t
}

/// The frame `cyborg_switch` pops on the first switch in.
#[cfg(target_arch = "aarch64")]
unsafe fn first_frame(below: *mut u8, t: *mut Task) -> *mut u8 {
    // x19..x28, x29, x30, d8..d15: 20 slots, 160 bytes.
    let top = ((below as usize) & !15) as *mut u64;
    let frame = top.sub(20);
    for i in 0..20 {
        *frame.add(i) = 0;
    }
    *frame.add(0) = t as u64; // x19: the task
    *frame.add(11) = cyborg_task_start as *const () as u64; // x30: where `ret` goes
    frame as *mut u8
}

#[cfg(target_arch = "x86_64")]
unsafe fn first_frame(below: *mut u8, t: *mut Task) -> *mut u8 {
    // r15 r14 r13 r12 rbx rbp, then the return address. After the pops and
    // the `ret`, the stack pointer must be 16-aligned for the `call` in
    // `cyborg_task_start`.
    let top = (below as usize) & !15;
    let frame = (top - 72) as *mut u64;
    for i in 0..7 {
        *frame.add(i) = 0;
    }
    *frame.add(4) = t as u64; // rbx: the task
    *frame.add(6) = cyborg_task_start as *const () as u64;
    frame as *mut u8
}

// ------------------------------------------------------------ what the compiler calls

/// Start a task. With a group, the block that owns the group waits for it;
/// without one it is detached, and lives as long as the program does.
///
/// # Safety
/// `entry` must be safe to call with `arg` on another thread, once.
#[no_mangle]
pub unsafe extern "C" fn cyborg_spawn(group: *mut Group, entry: unsafe extern "C" fn(*mut u8), arg: *mut u8) {
    if !group.is_null() {
        (*group).lock.lock();
        (*group).live += 1;
        (*group).lock.unlock();
    }
    push(new_task(entry, arg, group, STACK));
}

/// A group with nothing in it yet.
///
/// # Safety
/// `g` must point at `GROUP_BYTES` writable bytes, aligned to 16.
#[no_mangle]
pub unsafe extern "C" fn cyborg_group_init(g: *mut Group) {
    g.write(Group { lock: Spin(AtomicBool::new(false)), live: 0, waiter: null_mut() });
}

/// The end of a block that started tasks: wait for every one of them.
///
/// # Safety
/// Called from a task (MAIN is one, whenever a program starts tasks).
#[no_mangle]
pub unsafe extern "C" fn cyborg_group_wait(g: *mut Group) {
    (*g).lock.lock();
    if (*g).live == 0 {
        (*g).lock.unlock();
        return;
    }
    let t = current();
    (*t).state.store(PARKING, Ordering::Release);
    (*g).waiter = t;
    (*g).lock.unlock();
    park(t);
}

unsafe fn group_done(g: *mut Group) {
    (*g).lock.lock();
    (*g).live -= 1;
    let w = if (*g).live == 0 { core::mem::replace(&mut (*g).waiter, null_mut()) } else { null_mut() };
    (*g).lock.unlock();
    // Nothing of the group is touched after this: the block it belongs to
    // may end the moment its waiter is back.
    if !w.is_null() {
        unpark(w);
    }
}

unsafe extern "C" fn run_main(arg: *mut u8) {
    let main: extern "C" fn() = core::mem::transmute(arg);
    main();
    let s = sched();
    pthread_mutex_lock(&mut (*s).main_lock);
    (*s).main_done = true;
    pthread_cond_signal(&mut (*s).main_wake);
    pthread_mutex_unlock(&mut (*s).main_lock);
}

/// A program that starts tasks runs MAIN as a task too, on one worker per
/// core, so that MAIN can wait like any other task. The real main thread
/// sleeps until MAIN is done; returning then ends the program, and any
/// detached task still running with it.
///
/// # Safety
/// Called once, from the program's `main`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_run(main: extern "C" fn()) {
    let s = sched();
    pthread_mutex_init(&mut (*s).lock, core::ptr::null());
    pthread_cond_init(&mut (*s).wake, core::ptr::null());
    pthread_mutex_init(&mut (*s).main_lock, core::ptr::null());
    pthread_cond_init(&mut (*s).main_wake, core::ptr::null());
    pthread_key_create(&mut (*s).key, core::ptr::null());
    let cores = sysconf(sys::NPROCESSORS_ONLN).max(1);
    for _ in 0..cores {
        let mut id = 0u64;
        if pthread_create(&mut id, core::ptr::null(), worker, null_mut()) != 0 {
            fail_hard(b"could not start a worker thread\n");
        }
    }
    push(new_task(run_main, main as *mut u8, null_mut(), MAIN_STACK));
    pthread_mutex_lock(&mut (*s).main_lock);
    while !(*s).main_done {
        pthread_cond_wait(&mut (*s).main_wake, &mut (*s).main_lock);
    }
    pthread_mutex_unlock(&mut (*s).main_lock);
}
