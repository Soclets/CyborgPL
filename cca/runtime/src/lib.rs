//! The runtime linked into every CyborgPL binary. `no_std`: it is the
//! floor under the language, so it stands on the OS and nothing else.
//!
//! Today it holds one thing — what happens when a check fires. In the
//! working build an unproven site keeps its check, and a check that fires
//! stops the program and says where (`core-features.md`: "the program runs
//! and fails loudly where a guardrail is missing"; `diagnostics.md`: "the
//! trap at run time names the site").
#![no_std]

pub mod tasks;
pub mod par;
pub mod channels;
pub mod deci;

extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(code: i32) -> !;
    fn malloc(bytes: usize) -> *mut u8;
    fn calloc(count: usize, size: usize) -> *mut u8;
    fn realloc(p: *mut u8, bytes: usize) -> *mut u8;
    fn free(p: *mut u8);
    fn memcpy(dst: *mut u8, src: *const u8, bytes: usize) -> *mut u8;
    fn memmove(dst: *mut u8, src: *const u8, bytes: usize) -> *mut u8;
}

/// A check fired. `msg` is the site's own words, built at compile time, so
/// nothing has to be formatted here.
///
/// # Safety
/// `msg` must point at `len` readable bytes. The compiler only ever passes
/// a constant it emitted itself.
#[no_mangle]
pub unsafe extern "C" fn cyborg_check_failed(msg: *const u8, len: usize) -> ! {
    if !QUIET_STOP {
        write(2, msg, len);
    }
    exit(101)
}

// ------------------------------------------------------------------ tests
//
// `cca test` (owner, 2026-09-29) runs a test binary once per `test fn`,
// the test's number its first argument: a test that passes ends 0; a
// `must` that does not hold says `line N: …` on stderr and ends 1; a
// check that fires ends 101 as it does anywhere.

extern "C" {
    fn fork() -> i32;
    fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
}

/// Set in the process `must.stop` runs its expression in: the check it
/// expects is the test passing, not a message.
static mut QUIET_STOP: bool = false;

/// A `must` that does not hold: `line N: msg` on stderr, and the test ends.
///
/// # Safety
/// `msg` must point at `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn cyborg_must_report(msg: *const u8, len: u64, line: u64) -> ! {
    let mut buf = [0u8; 48];
    let mut n = 0;
    for &b in b"line " {
        buf[n] = b;
        n += 1;
    }
    let mut digits = [0u8; 20];
    let d = digits_of(line as u128, &mut digits);
    for &b in &digits[d..] {
        buf[n] = b;
        n += 1;
    }
    buf[n] = b':';
    buf[n + 1] = b' ';
    n += 2;
    write(2, buf.as_ptr(), n);
    write(2, msg, (len & !STATIC) as usize);
    write(2, b"\n".as_ptr(), 1);
    exit(1)
}

/// `must.stop`: the expression runs in a copy of this process. 1 there;
/// here, once it has ended, 0 when a check fired in it and 2 when none did.
#[no_mangle]
pub unsafe extern "C" fn cyborg_must_stop_fork() -> i32 {
    let pid = fork();
    if pid == 0 {
        QUIET_STOP = true;
        return 1;
    }
    if pid < 0 {
        return 2;
    }
    let mut status: i32 = 0;
    waitpid(pid, &mut status, 0);
    // Ended by `exit` (the low seven bits clear), with a check's code.
    let exited = status & 0x7f == 0;
    let code = (status >> 8) & 0xff;
    if exited && (code == 101 || code == 104) {
        0
    } else {
        2
    }
}

/// `must.stop`'s expression ran to its end: no check fired.
#[no_mangle]
pub unsafe extern "C" fn cyborg_must_stop_done() -> ! {
    exit(3)
}

/// The test to run: the binary's first argument, a number.
///
/// # Safety
/// `argv` must hold `argc` C strings.
#[no_mangle]
pub unsafe extern "C" fn cyborg_test_index(argc: i32, argv: *const *const u8) -> i64 {
    if argc < 2 {
        return -1;
    }
    let mut p = *argv.add(1);
    let mut n: i64 = 0;
    while *p >= b'0' && *p <= b'9' {
        n = n * 10 + (*p - b'0') as i64;
        p = p.add(1);
    }
    n
}

/// A proven site failed in the proof-checking build (`--assert-proofs`):
/// Precog claimed something that was not true. Its own exit code, so a
/// harness can tell a wrong proof from an ordinary check.
///
/// # Safety
/// As `cyborg_check_failed`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_proof_failed(msg: *const u8, len: usize) -> ! {
    if !QUIET_STOP {
        write(2, msg, len);
    }
    exit(104)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { exit(102) }
}

/// Nothing here unwinds — the profiles say `panic = "abort"` — but the
/// prebuilt `core` this links against was not built that way, and the
/// linker wants the symbol its objects name. It is never called.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}


// ----------------------------------------------------------------- memory
//
// `ffi.md`: the language has no raw-allocation primitive — "bytes you can
// name are a `list[uint8]`". So allocating is the runtime's job, and it
// stands on the C allocator the platform already has rather than carrying
// one of its own. A list, a map and a built text all come through here.

/// Out of memory is not a check firing and not a failure the program can
/// handle: there is nothing to hand back a failure *in*. It stops, saying
/// so.
unsafe fn out_of_memory() -> ! {
    let msg = b"out of memory\n";
    write(2, msg.as_ptr(), msg.len());
    exit(103)
}

/// `bytes` of memory, or the program stops. Zero bytes is a null pointer:
/// an empty list owns nothing, so there is nothing to free.
#[no_mangle]
pub unsafe extern "C" fn cyborg_alloc(bytes: u64) -> *mut u8 {
    if bytes == 0 {
        return core::ptr::null_mut();
    }
    let p = malloc(bytes as usize);
    if p.is_null() {
        out_of_memory();
    }
    p
}

/// Grow (or shrink) an allocation, keeping what was in it.
#[no_mangle]
pub unsafe extern "C" fn cyborg_realloc(p: *mut u8, bytes: u64) -> *mut u8 {
    if bytes == 0 {
        cyborg_free(p);
        return core::ptr::null_mut();
    }
    let q = if p.is_null() { malloc(bytes as usize) } else { realloc(p, bytes as usize) };
    if q.is_null() {
        out_of_memory();
    }
    q
}

/// Zeroed memory for `list.fill` of a zero (owner, 2026-09-30): the
/// system's own zero pages, as `calloc` hands them, never written byte by
/// byte. 0 bytes is the empty list's null pointer.
#[no_mangle]
pub unsafe extern "C" fn cyborg_alloc_zeroed(bytes: u64) -> *mut u8 {
    if bytes == 0 {
        return core::ptr::null_mut();
    }
    let p = calloc(bytes as usize, 1);
    if p.is_null() {
        out_of_memory();
    }
    p
}

/// Give memory back. A null pointer is an empty value and is not an error.
#[no_mangle]
pub unsafe extern "C" fn cyborg_free(p: *mut u8) {
    if !p.is_null() {
        free(p);
    }
}

/// Copy bytes. The regions never overlap: every caller is building a fresh
/// allocation.
#[no_mangle]
pub unsafe extern "C" fn cyborg_copy(dst: *mut u8, src: *const u8, bytes: u64) {
    if bytes != 0 {
        memcpy(dst, src, bytes as usize);
    }
}

/// Move bytes that may overlap: `list.remove` slides the tail of a list
/// down over the element that left.
#[no_mangle]
pub unsafe extern "C" fn cyborg_move(dst: *mut u8, src: *const u8, bytes: u64) {
    if bytes != 0 {
        memmove(dst, src, bytes as usize);
    }
}

// ------------------------------------------------------------------ text
//
// A text handle is a pointer and a length (`types.md`, owner
// 2026-09-22). The bytes may be the binary's own — a literal needs no
// heap — or they may have been built at run time, and freeing the first
// kind would crash. Which memory a text is in is Precog's to know; where
// it cannot, the handle says so in **the high bit of the length**, which
// is the fallback the owner decided on. A length is a count of bytes, so
// the top bit is free forever.

/// The bytes are the binary's own: never free them.
pub const STATIC: u64 = 1 << 63;

#[repr(C)]
pub struct Text {
    pub ptr: *mut u8,
    pub len: u64,
}

/// `++` — a new text holding one after the other. The answer is always on
/// the heap, so its length carries no mark.
///
/// # Safety
/// Each pointer must have as many readable bytes as its length says.
#[no_mangle]
pub unsafe extern "C" fn cyborg_str_concat(out: *mut Text, ap: *const u8, alen: u64, bp: *const u8, blen: u64) {
    let (alen, blen) = (alen & !STATIC, blen & !STATIC);
    let total = alen + blen;
    let p = cyborg_alloc(total);
    if alen != 0 {
        memcpy(p, ap, alen as usize);
    }
    if blen != 0 {
        memcpy(p.add(alen as usize), bp, blen as usize);
    }
    (*out).ptr = p;
    (*out).len = total;
}

/// Bytes read as UTF-8 (owner, 2026-09-30): a new text holding them, and
/// 0; or 1, `not_utf8`, with nothing made.
///
/// # Safety
/// `src` must point at `len` readable bytes, and `out` at a text to fill.
#[no_mangle]
pub unsafe extern "C" fn cyborg_str_from_bytes(out: *mut Text, src: *const u8, len: u64) -> u8 {
    let b = if len == 0 { &[][..] } else { core::slice::from_raw_parts(src, len as usize) };
    if core::str::from_utf8(b).is_err() {
        return 1;
    }
    let p = cyborg_alloc(len);
    if len != 0 {
        memcpy(p, src, len as usize);
    }
    (*out).ptr = p;
    (*out).len = len;
    0
}

/// A second, independent text. A constant one is already independent —
/// nothing can write to it — so it is handed back as it is.
///
/// # Safety
/// `src` must point at as many readable bytes as `len` says.
#[no_mangle]
pub unsafe extern "C" fn cyborg_str_copy(out: *mut Text, src: *const u8, len: u64) {
    if len & STATIC != 0 {
        (*out).ptr = src as *mut u8;
        (*out).len = len;
        return;
    }
    let p = cyborg_alloc(len);
    if len != 0 {
        memcpy(p, src, len as usize);
    }
    (*out).ptr = p;
    (*out).len = len;
}

/// Give a text's bytes back, unless they are the binary's own.
#[no_mangle]
pub unsafe extern "C" fn cyborg_str_free(ptr: *mut u8, len: u64) {
    if len & STATIC == 0 {
        cyborg_free(ptr);
    }
}


// ------------------------------------------------------------------- maps
//
// `maps.md` (owner, 2026-09-22): "one flat table, open addressing, never
// a linked bucket per entry". An unordered map's handle is three fields —
// the table, a count, a capacity — and an ordered map's is four, the
// fourth pointing at the insertion-order list. Entries come back in the
// order they were `set`, and a replaced key keeps its place.
//
// The table is one allocation holding four runs, in this order:
//
//     hashes   u64 per slot     (first, so it is aligned)
//     keys     ks  per slot
//     values   vs  per slot
//     states   u8  per slot
//
// The compiler passes the sizes, and the functions that hash, compare,
// copy and drop one key or value — it knows the types; the runtime does
// not.

const EMPTY: u8 = 0;
const FULL: u8 = 1;
/// A slot something was removed from: it still has to be probed past.
const DEAD: u8 = 2;

type Hasher = unsafe extern "C" fn(*const u8) -> u64;
type Same = unsafe extern "C" fn(*const u8, *const u8) -> bool;
type OneWay = unsafe extern "C" fn(*mut u8);

/// What the compiler knows about one map's types and the runtime does not.
#[repr(C)]
pub struct MapSpec {
    pub key_size: u64,
    pub val_size: u64,
    pub hash: Hasher,
    pub same: Same,
    /// Giving back what a key or a value owns; null when it owns nothing.
    pub drop_key: Option<OneWay>,
    pub drop_val: Option<OneWay>,
    /// Nonzero: the key type has its own `hash` and `op ==`, and "equal
    /// keys hash the same" is checked (`maps.md`). Keys that hash apart are
    /// never compared on the way to a slot, so a lookup that misses asks
    /// every key there is — `same` stops the program if one is equal.
    pub scan: u64,
}

#[repr(C)]
pub struct Map {
    pub table: *mut u8,
    pub count: u64,
    pub cap: u64,
}

unsafe fn slot_hashes(m: *const Map) -> *mut u64 {
    (*m).table as *mut u64
}
unsafe fn slot_keys(m: *const Map) -> *mut u8 {
    (*m).table.add(8 * (*m).cap as usize)
}
unsafe fn slot_vals(m: *const Map, s: *const MapSpec) -> *mut u8 {
    slot_keys(m).add(((*s).key_size * (*m).cap) as usize)
}
unsafe fn slot_states(m: *const Map, s: *const MapSpec) -> *mut u8 {
    slot_vals(m, s).add(((*s).val_size * (*m).cap) as usize)
}
fn block_bytes(cap: u64, s: *const MapSpec) -> u64 {
    unsafe { cap * (8 + (*s).key_size + (*s).val_size + 1) }
}

/// Where this key is, or where it would go. `found` says which.
unsafe fn probe(m: *const Map, s: *const MapSpec, key: *const u8, h: u64, found: *mut bool) -> u64 {
    let at = probe_slots(m, s, key, h, found);
    if !*found && (*s).scan != 0 {
        let states = slot_states(m, s);
        let keys = slot_keys(m);
        for i in 0..(*m).cap {
            if *states.add(i as usize) != EMPTY && *states.add(i as usize) != DEAD {
                ((*s).same)(keys.add((i * (*s).key_size) as usize), key);
            }
        }
    }
    at
}

unsafe fn probe_slots(m: *const Map, s: *const MapSpec, key: *const u8, h: u64, found: *mut bool) -> u64 {
    let cap = (*m).cap;
    let mask = cap - 1;
    let mut i = h & mask;
    let mut free = cap; // the first slot this key could be put in
    let states = slot_states(m, s);
    let hashes = slot_hashes(m);
    let keys = slot_keys(m);
    loop {
        match *states.add(i as usize) {
            EMPTY => {
                *found = false;
                return if free == cap { i } else { free };
            }
            DEAD => {
                if free == cap {
                    free = i;
                }
            }
            _ => {
                if *hashes.add(i as usize) == h && ((*s).same)(keys.add((i * (*s).key_size) as usize), key) {
                    *found = true;
                    return i;
                }
            }
        }
        i = (i + 1) & mask;
    }
}

/// A bigger table, with everything moved into it. An ordered map's list
/// is rewritten as it goes, because every slot has changed.
unsafe fn regrow(m: *mut Map, order: *mut *mut u64, s: *const MapSpec, want: u64) {
    let mut cap = 8u64;
    while cap < want * 2 {
        cap *= 2;
    }
    let old = Map { table: (*m).table, count: (*m).count, cap: (*m).cap };
    let fresh = cyborg_alloc(block_bytes(cap, s));
    let mut i = 0;
    while i < block_bytes(cap, s) {
        *fresh.add(i as usize) = 0;
        i += 1;
    }
    (*m).table = fresh;
    (*m).cap = cap;
    (*m).count = 0;
    let old_order = if order.is_null() { core::ptr::null_mut() } else { *order };
    let new_order = if order.is_null() { core::ptr::null_mut() } else { cyborg_alloc(cap * 8) as *mut u64 };
    // In insertion order when there is one, so the order survives.
    let moves = old.count;
    let mut n = 0;
    while n < moves {
        let from = if old_order.is_null() { next_full(&old, s, n) } else { *old_order.add(n as usize) };
        let k = slot_keys(&old).add((from * (*s).key_size) as usize);
        let v = slot_vals(&old, s).add((from * (*s).val_size) as usize);
        let h = *slot_hashes(&old).add(from as usize);
        let mut found = false;
        // Keys already known to be distinct: no need to ask them all.
        let to = probe_slots(m, s, k, h, &mut found);
        *slot_hashes(m).add(to as usize) = h;
        *slot_states(m, s).add(to as usize) = FULL;
        memcpy(slot_keys(m).add((to * (*s).key_size) as usize), k, (*s).key_size as usize);
        memcpy(slot_vals(m, s).add((to * (*s).val_size) as usize), v, (*s).val_size as usize);
        (*m).count += 1;
        if !new_order.is_null() {
            *new_order.add(n as usize) = to;
        }
        n += 1;
    }
    cyborg_free(old.table);
    if !order.is_null() {
        cyborg_free(old_order as *mut u8);
        *order = new_order;
    }
}

/// The `n`th full slot, for a map that keeps no order of its own.
unsafe fn next_full(m: *const Map, s: *const MapSpec, n: u64) -> u64 {
    let states = slot_states(m, s);
    let mut seen = 0;
    let mut i = 0;
    while i < (*m).cap {
        if *states.add(i as usize) == FULL {
            if seen == n {
                return i;
            }
            seen += 1;
        }
        i += 1;
    }
    0
}

/// `map.set` — add, or replace an existing key's value. A replaced key
/// keeps its place in the order (`maps.md`).
///
/// # Safety
/// `key` and `val` must point at one key and one value of this map's types.
#[no_mangle]
pub unsafe extern "C" fn cyborg_map_set(m: *mut Map, order: *mut *mut u64, s: *const MapSpec, key: *const u8, val: *const u8) {
    if (*m).cap == 0 || ((*m).count + 1) * 4 >= (*m).cap * 3 {
        regrow(m, order, s, (*m).count + 1);
    }
    let h = ((*s).hash)(key);
    let mut found = false;
    let at = probe(m, s, key, h, &mut found);
    let kp = slot_keys(m).add((at * (*s).key_size) as usize);
    let vp = slot_vals(m, s).add((at * (*s).val_size) as usize);
    if found {
        // The key that is already there stays; the value it had does not.
        if let Some(d) = (*s).drop_val {
            d(vp);
        }
        if let Some(d) = (*s).drop_key {
            d(key as *mut u8);
        }
        memcpy(vp, val, (*s).val_size as usize);
        return;
    }
    *slot_hashes(m).add(at as usize) = h;
    *slot_states(m, s).add(at as usize) = FULL;
    memcpy(kp, key, (*s).key_size as usize);
    memcpy(vp, val, (*s).val_size as usize);
    if !order.is_null() {
        *(*order).add((*m).count as usize) = at;
    }
    (*m).count += 1;
}

/// `map.get` / `map.has`. True when the key is there; `out` is written
/// only then, and only when it is not null.
///
/// # Safety
/// `key` must point at one key of this map's type, `out` at room for one
/// value or be null.
#[no_mangle]
pub unsafe extern "C" fn cyborg_map_get(m: *const Map, s: *const MapSpec, key: *const u8, out: *mut u8) -> bool {
    if (*m).cap == 0 {
        return false;
    }
    let h = ((*s).hash)(key);
    let mut found = false;
    let at = probe(m, s, key, h, &mut found);
    if found && !out.is_null() {
        memcpy(out, slot_vals(m, s).add((at * (*s).val_size) as usize), (*s).val_size as usize);
    }
    found
}

/// `map.remove` — hands the value back, like `list.remove`.
///
/// # Safety
/// As `cyborg_map_get`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_map_remove(m: *mut Map, order: *mut *mut u64, s: *const MapSpec, key: *const u8, out: *mut u8) -> bool {
    if (*m).cap == 0 {
        return false;
    }
    let h = ((*s).hash)(key);
    let mut found = false;
    let at = probe(m, s, key, h, &mut found);
    if !found {
        return false;
    }
    let vp = slot_vals(m, s).add((at * (*s).val_size) as usize);
    if out.is_null() {
        if let Some(d) = (*s).drop_val {
            d(vp);
        }
    } else {
        memcpy(out, vp, (*s).val_size as usize);
    }
    if let Some(d) = (*s).drop_key {
        d(slot_keys(m).add((at * (*s).key_size) as usize));
    }
    *slot_states(m, s).add(at as usize) = DEAD;
    if !order.is_null() {
        let list = *order;
        let mut i = 0;
        while i < (*m).count {
            if *list.add(i as usize) == at {
                while i + 1 < (*m).count {
                    *list.add(i as usize) = *list.add((i + 1) as usize);
                    i += 1;
                }
                break;
            }
            i += 1;
        }
    }
    (*m).count -= 1;
    true
}

/// The slot the `n`th entry is in, in insertion order when the map keeps
/// one. `map.keys`, `map.values` and `map.pairs` walk with this.
///
/// # Safety
/// `n` must be below the map's count.
#[no_mangle]
pub unsafe extern "C" fn cyborg_map_slot(m: *const Map, s: *const MapSpec, order: *const u64, n: u64) -> u64 {
    if order.is_null() {
        next_full(m, s, n)
    } else {
        *order.add(n as usize)
    }
}

/// Where one slot's key and value live.
///
/// # Safety
/// `at` must be a slot this map has.
#[no_mangle]
pub unsafe extern "C" fn cyborg_map_key_at(m: *const Map, s: *const MapSpec, at: u64) -> *mut u8 {
    slot_keys(m).add((at * (*s).key_size) as usize)
}

/// # Safety
/// As `cyborg_map_key_at`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_map_val_at(m: *const Map, s: *const MapSpec, at: u64) -> *mut u8 {
    slot_vals(m, s).add((at * (*s).val_size) as usize)
}

/// Give the whole map back, and everything its keys and values own.
///
/// # Safety
/// The map must not be used afterwards.
#[no_mangle]
pub unsafe extern "C" fn cyborg_map_free(m: *mut Map, order: *mut *mut u64, s: *const MapSpec) {
    if (*m).cap != 0 && !(*m).table.is_null() {
        let states = slot_states(m, s);
        let mut i = 0;
        while i < (*m).cap {
            if *states.add(i as usize) == FULL {
                if let Some(d) = (*s).drop_key {
                    d(slot_keys(m).add((i * (*s).key_size) as usize));
                }
                if let Some(d) = (*s).drop_val {
                    d(slot_vals(m, s).add((i * (*s).val_size) as usize));
                }
            }
            i += 1;
        }
        cyborg_free((*m).table);
    }
    (*m).table = core::ptr::null_mut();
    (*m).count = 0;
    (*m).cap = 0;
    if !order.is_null() {
        cyborg_free(*order as *mut u8);
        *order = core::ptr::null_mut();
    }
}

// ---------------------------------------------------------------- hashing
//
// The compiler generates one hash function per key type — it knows the
// shape; these are the pieces it builds them out of.

/// One round of a bit mixer: the same value always gives the same answer,
/// on every machine, which `reproducible-builds.md` asks of everything.
#[no_mangle]
pub extern "C" fn cyborg_hash_int(v: u64) -> u64 {
    let mut x = v ^ 0x9e37_79b9_7f4a_7c15;
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// FNV-1a over the bytes: short, and the same everywhere.
///
/// # Safety
/// `p` must point at `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn cyborg_hash_bytes(p: *const u8, len: u64) -> u64 {
    let len = len & !STATIC;
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut i = 0;
    while i < len {
        h ^= *p.add(i as usize) as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
        i += 1;
    }
    h
}

/// Two hashes into one, so a struct's is its fields' and a list's is its
/// elements'.
#[no_mangle]
pub extern "C" fn cyborg_hash_mix(a: u64, b: u64) -> u64 {
    cyborg_hash_int(a ^ b.rotate_left(17))
}

/// Do these bytes say the same thing? For keys that are plain values.
///
/// # Safety
/// Both must point at `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn cyborg_same_bytes(a: *const u8, b: *const u8, len: u64) -> bool {
    let mut i = 0;
    while i < len {
        if *a.add(i as usize) != *b.add(i as usize) {
            return false;
        }
        i += 1;
    }
    true
}

/// Two texts saying the same thing. The mark on a length is not part of
/// what a text *is*, so it is taken off first.
///
/// # Safety
/// Each pointer must have as many readable bytes as its length says.
#[no_mangle]
pub unsafe extern "C" fn cyborg_same_text(ap: *const u8, alen: u64, bp: *const u8, blen: u64) -> bool {
    let (alen, blen) = (alen & !STATIC, blen & !STATIC);
    alen == blen && cyborg_same_bytes(ap, bp, alen)
}


/// Text to a whole number (`conversion.md`), as the oracle reads it: an
/// optional `+` or `-`, then one or more ASCII digits and nothing else — no
/// spaces, no `_`, no decimals, no other scripts' digits. 0 and the value in
/// `out`; 1 when the text is not a number; 2 when it is one too big for 128
/// bits, with `out` saturated in its direction.
#[no_mangle]
pub unsafe extern "C" fn cyborg_parse_int(p: *const u8, len: u64, out: *mut i128) -> u8 {
    let bytes = if len == 0 { &[][..] } else { core::slice::from_raw_parts(p, len as usize) };
    let (neg, digits) = match bytes.first() {
        Some(b'-') => (true, &bytes[1..]),
        Some(b'+') => (false, &bytes[1..]),
        _ => (false, bytes),
    };
    if digits.is_empty() || !digits.iter().all(|c| c.is_ascii_digit()) {
        return 1;
    }
    let mut v: i128 = 0;
    for c in digits {
        let d = (c - b'0') as i128;
        match v.checked_mul(10).and_then(|x| if neg { x.checked_sub(d) } else { x.checked_add(d) }) {
            Some(x) => v = x,
            None => {
                *out = if neg { i128::MIN } else { i128::MAX };
                return 2;
            }
        }
    }
    *out = v;
    0
}

/// A text of `bytes`, on the heap.
unsafe fn text_of(out: *mut Text, bytes: &[u8]) {
    let p = cyborg_alloc(bytes.len() as u64);
    if !bytes.is_empty() {
        memcpy(p, bytes.as_ptr(), bytes.len());
    }
    (*out).ptr = p;
    (*out).len = bytes.len() as u64;
}

/// Decimal digits of `v` into the end of `buf`; where they start.
fn digits_of(mut v: u128, buf: &mut [u8]) -> usize {
    let mut at = buf.len();
    loop {
        at -= 1;
        buf[at] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 {
            return at;
        }
    }
}

/// A whole number as text, in its stored form (`conversion.md`): its
/// decimal digits, a `-` before a negative one. 128 bits, as two halves,
/// so every integer type and the exact one fit.
///
/// # Safety
/// `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_int_to_text(out: *mut Text, lo: u64, hi: u64) {
    let v = ((hi as u128) << 64 | lo as u128) as i128;
    let mut buf = [0u8; 41];
    let mut at = digits_of(v.unsigned_abs(), &mut buf);
    if v < 0 {
        at -= 1;
        buf[at] = b'-';
    }
    text_of(out, &buf[at..]);
}

/// A number held as limbs of nine decimal digits, least significant first:
/// enough for the most digits a `bin64`'s expansion needs, m x 5^1074
/// (767).
struct Big {
    limbs: [u32; 96],
    used: usize,
}

impl Big {
    fn of(mut v: u64) -> Big {
        let mut b = Big { limbs: [0; 96], used: 0 };
        while v != 0 {
            b.limbs[b.used] = (v % 1_000_000_000) as u32;
            v /= 1_000_000_000;
            b.used += 1;
        }
        b
    }

    fn times(&mut self, k: u32) {
        let mut carry: u64 = 0;
        for i in 0..self.used {
            let x = self.limbs[i] as u64 * k as u64 + carry;
            self.limbs[i] = (x % 1_000_000_000) as u32;
            carry = x / 1_000_000_000;
        }
        while carry != 0 {
            self.limbs[self.used] = (carry % 1_000_000_000) as u32;
            carry /= 1_000_000_000;
            self.used += 1;
        }
    }

    /// Its decimal digits into `buf`, most significant first; how many.
    fn write(&self, buf: &mut [u8]) -> usize {
        if self.used == 0 {
            buf[0] = b'0';
            return 1;
        }
        let mut n = 0;
        for i in (0..self.used).rev() {
            let mut limb = [0u8; 9];
            let mut v = self.limbs[i];
            for d in limb.iter_mut().rev() {
                *d = b'0' + (v % 10) as u8;
                v /= 10;
            }
            // The top limb without its leading zeros.
            let from = if i == self.used - 1 { limb.iter().position(|&c| c != b'0').unwrap_or(8) } else { 0 };
            for &c in &limb[from..] {
                buf[n] = c;
                n += 1;
            }
        }
        n
    }
}

/// A bin as text, in its stored form: "the full decimal expansion of the
/// stored binary" (DECISIONS, 2026-09-14) — every digit, no exponent, no
/// trailing zeros, `-0` kept; `infinity`, `-infinity` and `NaN` as the
/// oracle spells them. A `bin16` or `bin32` comes widened to 64 bits,
/// which is exact, so its expansion is its own.
///
/// # Safety
/// `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_bin_to_text(out: *mut Text, x: f64) {
    // The longest is the smallest subnormal: "-0." and 1074 digits.
    let mut buf = [0u8; 1080];
    let n = expand(x, &mut buf);
    text_of(out, &buf[..n]);
}

/// The exact decimal expansion of `x` into `buf`; how many bytes.
fn expand(x: f64, buf: &mut [u8; 1080]) -> usize {
    let bits = x.to_bits();
    let neg = bits >> 63 != 0;
    let exp = ((bits >> 52) & 0x7ff) as i32;
    let frac = bits & ((1 << 52) - 1);
    if exp == 0x7ff {
        let s: &[u8] = if frac != 0 { b"NaN" } else if neg { b"-infinity" } else { b"infinity" };
        buf[..s.len()].copy_from_slice(s);
        return s.len();
    }
    // x = m x 2^e, with m odd (or 0).
    let (mut m, mut e) = if exp == 0 { (frac, -1074) } else { (frac | 1 << 52, exp - 1075) };
    while m != 0 && m & 1 == 0 {
        m >>= 1;
        e += 1;
    }
    let mut n = 0;
    if neg {
        buf[0] = b'-';
        n = 1;
    }
    let mut big = Big::of(m);
    if m == 0 || e >= 0 {
        // Whole: m x 2^e, doubled in steps a limb's product can hold.
        let mut k = if m == 0 { 0 } else { e };
        while k > 0 {
            let s = k.min(30);
            big.times(1 << s);
            k -= s;
        }
        n += big.write(&mut buf[n..]);
        return n;
    }
    // m / 2^k is m x 5^k / 10^k: the digits of m x 5^k, with the point k
    // from the right. m is odd, so the last digit is 5 — nothing to trim.
    let k = (-e) as usize;
    let mut left = k;
    while left > 0 {
        let s = left.min(13);
        big.times(5u32.pow(s as u32));
        left -= s;
    }
    let mut digits = [0u8; 800];
    let d = big.write(&mut digits);
    if d > k {
        buf[n..n + d - k].copy_from_slice(&digits[..d - k]);
        n += d - k;
    } else {
        buf[n] = b'0';
        n += 1;
    }
    buf[n] = b'.';
    n += 1;
    for _ in d..k {
        buf[n] = b'0';
        n += 1;
    }
    let from = d.saturating_sub(k);
    buf[n..n + (d - from)].copy_from_slice(&digits[from..d]);
    n + (d - from)
}

/// A character as text: the UTF-8 of its code. A `char.byte` is read as
/// the code point of that number, so byte 200 is two bytes of text — the
/// oracle's reading (`cca/README.md`, provisional).
///
/// # Safety
/// `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_char_to_text(out: *mut Text, code: u32) {
    let mut buf = [0u8; 4];
    let n = match code {
        0..=0x7f => {
            buf[0] = code as u8;
            1
        }
        0x80..=0x7ff => {
            buf[0] = 0xc0 | (code >> 6) as u8;
            buf[1] = 0x80 | (code & 0x3f) as u8;
            2
        }
        0x800..=0xffff => {
            buf[0] = 0xe0 | (code >> 12) as u8;
            buf[1] = 0x80 | ((code >> 6) & 0x3f) as u8;
            buf[2] = 0x80 | (code & 0x3f) as u8;
            3
        }
        _ => {
            buf[0] = 0xf0 | (code >> 18) as u8;
            buf[1] = 0x80 | ((code >> 12) & 0x3f) as u8;
            buf[2] = 0x80 | ((code >> 6) & 0x3f) as u8;
            buf[3] = 0x80 | (code & 0x3f) as u8;
            4
        }
    };
    text_of(out, &buf[..n]);
}

/// The one code point a text holds, or -1 when it holds none or more than
/// one (`to.char.point[…]` fails `not_one_char`). Text is UTF-8, so the
/// first byte says how long its point is, and that has to be all of it.
///
/// # Safety
/// `p` must have `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn cyborg_text_one_point(p: *const u8, len: u64) -> i64 {
    let len = len & !STATIC;
    if len == 0 || len > 4 {
        return -1;
    }
    let b = core::slice::from_raw_parts(p, len as usize);
    let (want, first) = match b[0] {
        0x00..=0x7f => (1, b[0] as u32),
        0xc0..=0xdf => (2, (b[0] & 0x1f) as u32),
        0xe0..=0xef => (3, (b[0] & 0x0f) as u32),
        0xf0..=0xf7 => (4, (b[0] & 0x07) as u32),
        _ => return -1,
    };
    if want != b.len() {
        return -1;
    }
    let mut code = first;
    for &c in &b[1..] {
        code = code << 6 | (c & 0x3f) as u32;
    }
    code as i64
}

/// Text to a bin (`conversion.md`, owner 2026-09-24: b): an optional sign,
/// digits, an optional `.` and digits, an optional `e`/`E` exponent with
/// its own sign — and nothing else, so `inf`, `nan`, `_`, `.5`, `5.` and
/// spaces are not numbers. The value is rounded once, from the digits, to
/// the target's width (`bits`): 0 and the value in `out`; 1 when the text
/// is not a number; 2 when it rounds past the largest (`out_of_range`).
///
/// # Safety
/// `p` must have `len` readable bytes, and `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_parse_bin(p: *const u8, len: u64, bits: u32, out: *mut f64) -> u8 {
    let len = len & !STATIC;
    let b = if len == 0 { &[][..] } else { core::slice::from_raw_parts(p, len as usize) };
    let Some(parts) = bin_grammar(b) else { return 1 };
    let Ok(s) = core::str::from_utf8(b) else { return 1 };
    let v = match bits {
        64 => s.parse::<f64>().ok(),
        32 => s.parse::<f32>().ok().map(|x| x as f64),
        16 => s.parse::<f64>().ok().map(|x| round_to_half(x, &parts)),
        BRAIN => s.parse::<f64>().ok().map(|x| round_to_brain(x, &parts)),
        _ => None,
    };
    match v {
        Some(x) if x.is_finite() => {
            *out = x;
            0
        }
        Some(_) => 2,
        None => 1,
    }
}

/// A number's written digits, as the grammar found them.
struct Written<'a> {
    neg: bool,
    int: &'a [u8],
    frac: &'a [u8],
    exp: i64,
}

fn bin_grammar(b: &[u8]) -> Option<Written<'_>> {
    let mut i = 0;
    let neg = match b.first() {
        Some(b'-') => {
            i = 1;
            true
        }
        Some(b'+') => {
            i = 1;
            false
        }
        _ => false,
    };
    let digits = |from: usize| from + b[from..].iter().take_while(|c| c.is_ascii_digit()).count();
    let int_end = digits(i);
    if int_end == i {
        return None;
    }
    let int = &b[i..int_end];
    i = int_end;
    let mut frac: &[u8] = &[];
    if b.get(i) == Some(&b'.') {
        let end = digits(i + 1);
        if end == i + 1 {
            return None;
        }
        frac = &b[i + 1..end];
        i = end;
    }
    let mut exp: i64 = 0;
    if matches!(b.get(i), Some(b'e') | Some(b'E')) {
        i += 1;
        let eneg = match b.get(i) {
            Some(b'-') => {
                i += 1;
                true
            }
            Some(b'+') => {
                i += 1;
                false
            }
            _ => false,
        };
        let end = digits(i);
        if end == i {
            return None;
        }
        for &c in &b[i..end] {
            // Past a million the value is 0 or past every width anyway.
            exp = (exp * 10 + (c - b'0') as i64).min(1_000_000);
        }
        if eneg {
            exp = -exp;
        }
        i = end;
    }
    if i != b.len() {
        return None;
    }
    Some(Written { neg, int, frac, exp })
}

/// `x` (the digits read at 64 bits) rounded to a bin16, returned as the
/// 64-bit value it is. 64 bits hold more than twice a bin16's precision,
/// so rounding twice goes wrong only when `x` lands exactly halfway
/// between two bin16 values; there the written digits decide.
/// The width code of a `brain16` (owner, 2026-10-02), where a call takes a
/// bin's width: the brain float, 8 exponent bits and 7 of precision.
pub const BRAIN: u32 = 0xB16;

fn round_to_half(x: f64, w: &Written) -> f64 {
    round_to(x, w, 10, -14, 65504.0)
}

/// To the nearest brain16: bin32's exponents, 7 bits of precision; its
/// largest is (2 - 2^-7) x 2^127.
fn round_to_brain(x: f64, w: &Written) -> f64 {
    round_to(x, w, 7, -126, 3.3895313892515355e38)
}

/// The nearest value with `frac` fraction bits, normal from 2^`min_exp`,
/// ties broken by the written digits (then to even); past `max`, infinity.
fn round_to(x: f64, w: &Written, frac: i32, min_exp: i32, max: f64) -> f64 {
    let a = x.abs();
    let exp = ((a.to_bits() >> 52) & 0x7ff) as i32 - 1023;
    // The step between values here: fixed below the smallest normal.
    let two = |e: i32| f64::from_bits(((e + 1023) as u64) << 52);
    let q = if a < two(min_exp) { two(min_exp - frac) } else { two(exp - frac) };
    let n = a / q;
    let lower = floor(n);
    let frac = n - lower;
    let up = if frac > 0.5 {
        true
    } else if frac < 0.5 {
        false
    } else {
        // Halfway at 64 bits: is the written number above it, below it,
        // or on it (then to even)?
        let mut buf = [0u8; 1080];
        let len = expand(a, &mut buf);
        match compare_written(w, &buf[..len]) {
            core::cmp::Ordering::Greater => true,
            core::cmp::Ordering::Less => false,
            core::cmp::Ordering::Equal => (lower as u64) % 2 == 1,
        }
    };
    let r = (if up { lower + 1.0 } else { lower }) * q;
    let r = if r > max { f64::INFINITY } else { r };
    if w.neg { -r } else { r }
}

/// `floor` without std.
fn floor(x: f64) -> f64 {
    let t = x as i64 as f64;
    if t > x { t - 1.0 } else { t }
}

/// The written number's magnitude against an exact expansion ("123.45").
fn compare_written(w: &Written, expansion: &[u8]) -> core::cmp::Ordering {
    // Each as significant digits and where the point goes after them.
    let (eint, efrac) = match expansion.iter().position(|&c| c == b'.') {
        Some(p) => (&expansion[..p], &expansion[p + 1..]),
        None => (expansion, &[][..]),
    };
    let a = Norm::of(w.int, w.frac, w.exp);
    let b = Norm::of(eint, efrac, 0);
    a.cmp(&b)
}

/// A positive decimal as 0.DIGITS x 10^point, no leading or trailing zeros.
struct Norm {
    digits: [u8; 1200],
    len: usize,
    point: i64,
}

impl Norm {
    fn of(int: &[u8], frac: &[u8], exp: i64) -> Norm {
        let mut n = Norm { digits: [0; 1200], len: 0, point: int.len() as i64 + exp };
        let mut leading = true;
        for &c in int.iter().chain(frac.iter()) {
            if leading && c == b'0' {
                n.point -= 1;
                continue;
            }
            leading = false;
            if n.len < n.digits.len() {
                n.digits[n.len] = c;
                n.len += 1;
            }
        }
        while n.len > 0 && n.digits[n.len - 1] == b'0' {
            n.len -= 1;
        }
        n
    }

    fn cmp(&self, o: &Norm) -> core::cmp::Ordering {
        use core::cmp::Ordering::*;
        match (self.len == 0, o.len == 0) {
            (true, true) => return Equal,
            (true, false) => return Less,
            (false, true) => return Greater,
            _ => {}
        }
        if self.point != o.point {
            return self.point.cmp(&o.point);
        }
        self.digits[..self.len].cmp(&o.digits[..o.len])
    }
}

/// How many grapheme clusters a text holds (UAX #29, extended): what a
/// person sees as one character (`types.md`, `char.grapheme`).
///
/// # Safety
/// `p` must have `len` readable bytes of UTF-8.
#[no_mangle]
pub unsafe extern "C" fn cyborg_str_graphemes(p: *const u8, len: u64) -> u64 {
    use unicode_segmentation::UnicodeSegmentation;
    let len = len & !STATIC;
    if len == 0 {
        return 0;
    }
    let s = core::str::from_utf8_unchecked(core::slice::from_raw_parts(p, len as usize));
    s.graphemes(true).count() as u64
}

/// How many code points a text holds: every byte that does not continue one.
///
/// # Safety
/// `p` must have `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn cyborg_str_points(p: *const u8, len: u64) -> u64 {
    let len = len & !STATIC;
    let b = if len == 0 { &[][..] } else { core::slice::from_raw_parts(p, len as usize) };
    b.iter().filter(|&&c| c & 0xc0 != 0x80).count() as u64
}

/// Whether every byte of a text is ASCII: `to.str.ascii` fails
/// `not_ascii` otherwise (owner, 2026-09-30). Any text is UTF-8 inside,
/// so a byte of 0x80 or more is a character past ASCII.
///
/// # Safety
/// `p` must have `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn cyborg_str_is_ascii(p: *const u8, len: u64) -> u8 {
    let len = len & !STATIC;
    let b = if len == 0 { &[][..] } else { core::slice::from_raw_parts(p, len as usize) };
    b.iter().all(|&c| c < 0x80) as u8
}

/// A text's code points, decoded into `dst` (room for as many as
/// `cyborg_str_points` counted). `str.each.point`.
///
/// # Safety
/// `p` must have `len` readable bytes of UTF-8 and `dst` room for them.
#[no_mangle]
pub unsafe extern "C" fn cyborg_str_decode_points(p: *const u8, len: u64, dst: *mut u32) -> u64 {
    let len = len & !STATIC;
    if len == 0 {
        return 0;
    }
    let s = core::str::from_utf8_unchecked(core::slice::from_raw_parts(p, len as usize));
    let mut n = 0;
    for c in s.chars() {
        *dst.add(n) = c as u32;
        n += 1;
    }
    n as u64
}

/// A text's grapheme clusters, each a new text of its own in `dst` (room
/// for as many as `cyborg_str_graphemes` counted). `str.each.grapheme`.
///
/// # Safety
/// `p` must have `len` readable bytes of UTF-8 and `dst` room for them.
#[no_mangle]
pub unsafe extern "C" fn cyborg_str_split_graphemes(p: *const u8, len: u64, dst: *mut Text) -> u64 {
    use unicode_segmentation::UnicodeSegmentation;
    let len = len & !STATIC;
    if len == 0 {
        return 0;
    }
    let s = core::str::from_utf8_unchecked(core::slice::from_raw_parts(p, len as usize));
    let mut n = 0;
    for g in s.graphemes(true) {
        text_of(dst.add(n), g.as_bytes());
        n += 1;
    }
    n as u64
}
