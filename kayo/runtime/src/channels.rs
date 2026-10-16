//! Channels (`design/concurrency.md`): how tasks talk.
//!
//! The layout is the owner's (2026-09-23): **one block** — the bookkeeping,
//! then N slots of the element type inline. A channel's capacity is known
//! when it is made (no bound is a hand-off with no slots; `[0, N]` is N),
//! so it never grows, and the value a program holds is a pointer to that
//! block.
//!
//! **When a channel is closed** is the owner's too (2026-09-23, c): a
//! channel counts the tasks that can reach it — the one that made it, and
//! each task it is lent to — and a `take` on an empty channel fails
//! `closed` when every task still holding it is itself waiting to take from
//! it. Nothing can arrive then.
//!
//! A task that waits — a `take` on an empty channel, a `send` with nowhere
//! to put its value, a `whichever` with nothing ready — is parked, not
//! spinning: its worker runs something else, and whoever gives it what it
//! waits for puts it back (`tasks.rs`).
//!
//! A value is moved in and out as bytes: `channel.send` gives it (the
//! sender's name is gone afterwards), so exactly one side owns it at every
//! moment and nothing is copied that would have to be freed twice.

use core::ptr::{copy_nonoverlapping, null_mut};
use core::sync::atomic::{AtomicI64, AtomicU32, AtomicU64, Ordering};

use crate::tasks::{current, park, unpark, Spin, Task, PARKING};
use crate::{cyborg_alloc, cyborg_free};

const WAITING: u32 = 0;
const DONE: u32 = 1;
const CLOSED: u32 = 2;

#[repr(C)]
struct Queue {
    head: *mut Waiter,
    tail: *mut Waiter,
}

#[repr(C)]
pub struct Chan {
    lock: Spin,
    /// Slots: 0 is a hand-off.
    cap: u64,
    /// Bytes per element.
    size: u64,
    head: u64,
    count: u64,
    /// Tasks that can reach this channel.
    holders: u64,
    /// Waiters in `takers` — a plain `take`, or one arm of a `whichever`.
    waiting: u64,
    senders: Queue,
    takers: Queue,
}

/// The bookkeeping, rounded so the slots after it are aligned for anything.
const HEADER: usize = (core::mem::size_of::<Chan>() + 15) & !15;

/// One task waiting on one channel. It lives on the waiting task's own
/// stack, which is safe: that task is parked until this is done with.
#[repr(C)]
pub struct Waiter {
    task: *mut Task,
    /// A sender's value, or where a taker's value goes.
    value: *mut u8,
    /// Set when this is one arm of a `whichever`.
    sel: *mut Select,
    arm: i64,
    state: AtomicU32,
    next: *mut Waiter,
}

/// A `whichever` in progress: which arm won, and how many are closed.
#[repr(C)]
pub struct Select {
    /// -1 while waiting; the arm that fired; -2 when every arm is closed.
    fired: AtomicI64,
    closed: AtomicU64,
    arms: u64,
    task: *mut Task,
}

unsafe fn slot(ch: *mut Chan, i: u64) -> *mut u8 {
    (ch as *mut u8).add(HEADER + (i * (*ch).size) as usize)
}

unsafe fn push(q: *mut Queue, w: *mut Waiter) {
    (*w).next = null_mut();
    if (*q).tail.is_null() {
        (*q).head = w;
    } else {
        (*(*q).tail).next = w;
    }
    (*q).tail = w;
}

unsafe fn pop(q: *mut Queue) -> *mut Waiter {
    let w = (*q).head;
    if !w.is_null() {
        (*q).head = (*w).next;
        if (*q).head.is_null() {
            (*q).tail = null_mut();
        }
    }
    w
}

unsafe fn remove(q: *mut Queue, w: *mut Waiter) -> bool {
    let mut prev: *mut Waiter = null_mut();
    let mut cur = (*q).head;
    while !cur.is_null() {
        if cur == w {
            let next = (*cur).next;
            if prev.is_null() {
                (*q).head = next;
            } else {
                (*prev).next = next;
            }
            if (*q).tail == cur {
                (*q).tail = prev;
            }
            return true;
        }
        prev = cur;
        cur = (*cur).next;
    }
    false
}

/// A waiting taker, if there still is one. The arm of a `whichever` that
/// has already fired somewhere else is dropped on the way.
unsafe fn claim_taker(ch: *mut Chan) -> *mut Waiter {
    loop {
        let w = pop(&mut (*ch).takers);
        if w.is_null() {
            return w;
        }
        (*ch).waiting -= 1;
        if (*w).sel.is_null() {
            return w;
        }
        if (*(*w).sel).fired.compare_exchange(-1, (*w).arm, Ordering::AcqRel, Ordering::Acquire).is_ok() {
            return w;
        }
    }
}

/// Hand over: the waiter has its value, and goes back on the queue.
unsafe fn wake(w: *mut Waiter, how: u32) {
    if (*w).sel.is_null() {
        let t = (*w).task;
        (*w).state.store(how, Ordering::Release);
        unpark(t);
    } else {
        unpark((*(*w).sel).task);
    }
}

/// Nothing can ever arrive: empty, nobody waiting to send, and every task
/// that can reach it waiting to take from it (the owner's rule). An arm of
/// a `whichever` that already fired elsewhere is not waiting any more, so
/// it is cleared out before counting.
unsafe fn closes(ch: *mut Chan) -> bool {
    if (*ch).count != 0 || !(*ch).senders.head.is_null() {
        return false;
    }
    let mut cur = (*ch).takers.head;
    while !cur.is_null() {
        let next = (*cur).next;
        if !(*cur).sel.is_null() && (*(*cur).sel).fired.load(Ordering::Acquire) != -1 {
            remove(&mut (*ch).takers, cur);
            (*ch).waiting -= 1;
        }
        cur = next;
    }
    (*ch).waiting >= (*ch).holders
}

/// Tell every waiting taker the channel is closed.
unsafe fn close_all(ch: *mut Chan) {
    loop {
        let w = pop(&mut (*ch).takers);
        if w.is_null() {
            break;
        }
        (*ch).waiting -= 1;
        if (*w).sel.is_null() {
            wake(w, CLOSED);
        } else {
            let sel = (*w).sel;
            let now = (*sel).closed.fetch_add(1, Ordering::AcqRel) + 1;
            if now == (*sel).arms && (*sel).fired.compare_exchange(-1, -2, Ordering::AcqRel, Ordering::Acquire).is_ok() {
                unpark((*sel).task);
            }
        }
    }
}

/// `channel[T]` / `channel[T][0, N]`: the block, empty, held by its maker.
///
/// # Safety
/// Called by generated code with the element's size.
#[no_mangle]
pub unsafe extern "C" fn cyborg_chan_new(cap: u64, size: u64) -> *mut Chan {
    let ch = cyborg_alloc(HEADER as u64 + cap * size) as *mut Chan;
    ch.write(Chan {
        lock: Spin(core::sync::atomic::AtomicBool::new(false)),
        cap,
        size,
        head: 0,
        count: 0,
        holders: 1,
        waiting: 0,
        senders: Queue { head: null_mut(), tail: null_mut() },
        takers: Queue { head: null_mut(), tail: null_mut() },
    });
    ch
}

/// Another task can reach this channel: one lent into a `concur`.
///
/// # Safety
/// `ch` is a live channel.
#[no_mangle]
pub unsafe extern "C" fn cyborg_chan_hold(ch: *mut Chan) {
    (*ch).lock.lock();
    (*ch).holders += 1;
    (*ch).lock.unlock();
}

/// A task that could reach this channel has ended. If everyone left is
/// waiting to take, nothing will ever arrive, and they are told so.
///
/// # Safety
/// `ch` is a live channel this task held.
#[no_mangle]
pub unsafe extern "C" fn cyborg_chan_release(ch: *mut Chan) {
    (*ch).lock.lock();
    (*ch).holders -= 1;
    if (*ch).waiting > 0 && closes(ch) {
        close_all(ch);
    }
    (*ch).lock.unlock();
}

/// The channel's owner lets go of it: whatever is still in it is given
/// back, then the block.
///
/// # Safety
/// No task may use `ch` afterwards; structured tasks have all ended by the
/// time their block drops what it owns.
#[no_mangle]
pub unsafe extern "C" fn cyborg_chan_drop(ch: *mut Chan, drop_elem: Option<unsafe extern "C" fn(*mut u8)>) {
    if ch.is_null() {
        return;
    }
    (*ch).lock.lock();
    (*ch).holders = (*ch).holders.saturating_sub(1);
    if (*ch).waiting > 0 && closes(ch) {
        close_all(ch);
    }
    if let Some(d) = drop_elem {
        for k in 0..(*ch).count {
            d(slot(ch, ((*ch).head + k) % (*ch).cap.max(1)));
        }
    }
    (*ch).lock.unlock();
    cyborg_free(ch as *mut u8);
}

/// `channel.send` — the value is given. A waiting taker gets it directly;
/// otherwise it goes in a slot; with no slot free (or none at all: a
/// hand-off) the sender waits until a taker comes for it.
///
/// # Safety
/// `value` points at one element, which the channel now owns.
#[no_mangle]
pub unsafe extern "C" fn cyborg_chan_send(ch: *mut Chan, value: *mut u8) {
    (*ch).lock.lock();
    let w = claim_taker(ch);
    if !w.is_null() {
        copy_nonoverlapping(value, (*w).value, (*ch).size as usize);
        wake(w, DONE);
        (*ch).lock.unlock();
        return;
    }
    if (*ch).count < (*ch).cap {
        let at = ((*ch).head + (*ch).count) % (*ch).cap;
        copy_nonoverlapping(value, slot(ch, at), (*ch).size as usize);
        (*ch).count += 1;
        (*ch).lock.unlock();
        return;
    }
    let me = current();
    let mut w = Waiter { task: me, value, sel: null_mut(), arm: 0, state: AtomicU32::new(WAITING), next: null_mut() };
    push(&mut (*ch).senders, &mut w);
    (*me).state.store(PARKING, Ordering::Release);
    (*ch).lock.unlock();
    park(me);
}

/// Take a value that is there now, if one is: from a slot (and let a
/// waiting sender's value into the slot it frees) or straight from a
/// waiting sender. Called with the lock held.
unsafe fn take_now(ch: *mut Chan, dest: *mut u8) -> bool {
    if (*ch).count > 0 {
        copy_nonoverlapping(slot(ch, (*ch).head), dest, (*ch).size as usize);
        (*ch).head = ((*ch).head + 1) % (*ch).cap;
        (*ch).count -= 1;
        let s = pop(&mut (*ch).senders);
        if !s.is_null() {
            let at = ((*ch).head + (*ch).count) % (*ch).cap;
            copy_nonoverlapping((*s).value, slot(ch, at), (*ch).size as usize);
            (*ch).count += 1;
            wake(s, DONE);
        }
        return true;
    }
    let s = pop(&mut (*ch).senders);
    if !s.is_null() {
        copy_nonoverlapping((*s).value, dest, (*ch).size as usize);
        wake(s, DONE);
        return true;
    }
    false
}

/// `channel.take` — true with the value in `dest`, or false: `closed`.
///
/// # Safety
/// `dest` has room for one element.
#[no_mangle]
pub unsafe extern "C" fn cyborg_chan_take(ch: *mut Chan, dest: *mut u8) -> bool {
    (*ch).lock.lock();
    if take_now(ch, dest) {
        (*ch).lock.unlock();
        return true;
    }
    (*ch).waiting += 1;
    if closes(ch) {
        (*ch).waiting -= 1;
        close_all(ch);
        (*ch).lock.unlock();
        return false;
    }
    let me = current();
    let mut w = Waiter { task: me, value: dest, sel: null_mut(), arm: 0, state: AtomicU32::new(WAITING), next: null_mut() };
    push(&mut (*ch).takers, &mut w);
    (*me).state.store(PARKING, Ordering::Release);
    (*ch).lock.unlock();
    park(me);
    w.state.load(Ordering::Acquire) == DONE
}

/// `whichever` — the index of the arm that got a value (in its `dest`), or
/// -1 when every arm is closed. Arms are tried in the order written, so of
/// several ready at once the first wins, which keeps a run reproducible.
///
/// # Safety
/// `chans` and `dests` hold `n` live channels and destinations.
#[no_mangle]
pub unsafe extern "C" fn cyborg_whichever(n: u64, chans: *const *mut Chan, dests: *const *mut u8) -> i64 {
    // Anything ready now?
    for i in 0..n as usize {
        let ch = *chans.add(i);
        (*ch).lock.lock();
        let got = take_now(ch, *dests.add(i));
        (*ch).lock.unlock();
        if got {
            return i as i64;
        }
    }
    // Wait on all of them at once.
    let me = current();
    let mut sel = Select { fired: AtomicI64::new(-1), closed: AtomicU64::new(0), arms: n, task: me };
    let waiters = cyborg_alloc(n * core::mem::size_of::<Waiter>() as u64) as *mut Waiter;
    let mut registered = 0usize;
    (*me).state.store(PARKING, Ordering::Release);
    for i in 0..n as usize {
        let ch = *chans.add(i);
        let w = waiters.add(i);
        w.write(Waiter { task: me, value: *dests.add(i), sel: &mut sel, arm: i as i64, state: AtomicU32::new(WAITING), next: null_mut() });
        (*ch).lock.lock();
        // Something may have come while the others were being set up.
        if (*ch).count > 0 || !(*ch).senders.head.is_null() {
            if sel.fired.compare_exchange(-1, i as i64, Ordering::AcqRel, Ordering::Acquire).is_ok() {
                take_now(ch, *dests.add(i));
                (*ch).lock.unlock();
                (*me).state.store(crate::tasks::RUNNING, Ordering::Release);
                unregister(chans, waiters, registered);
                cyborg_free(waiters as *mut u8);
                return i as i64;
            }
            (*ch).lock.unlock();
            break;
        }
        (*ch).waiting += 1;
        if closes(ch) {
            (*ch).waiting -= 1;
            close_all(ch);
            sel.closed.fetch_add(1, Ordering::AcqRel);
            (*ch).lock.unlock();
            continue;
        }
        push(&mut (*ch).takers, w);
        registered = i + 1;
        (*ch).lock.unlock();
    }
    if sel.closed.load(Ordering::Acquire) == n && sel.fired.compare_exchange(-1, -2, Ordering::AcqRel, Ordering::Acquire).is_ok() {
        (*me).state.store(crate::tasks::RUNNING, Ordering::Release);
        unregister(chans, waiters, registered);
        cyborg_free(waiters as *mut u8);
        return -1;
    }
    park(me);
    unregister(chans, waiters, n as usize);
    cyborg_free(waiters as *mut u8);
    let got = sel.fired.load(Ordering::Acquire);
    if got >= 0 {
        got
    } else {
        -1
    }
}

/// Take this `whichever`'s arms back off every channel it waited on.
unsafe fn unregister(chans: *const *mut Chan, waiters: *mut Waiter, n: usize) {
    for i in 0..n {
        let ch = *chans.add(i);
        (*ch).lock.lock();
        if remove(&mut (*ch).takers, waiters.add(i)) {
            (*ch).waiting -= 1;
        }
        (*ch).lock.unlock();
    }
}
