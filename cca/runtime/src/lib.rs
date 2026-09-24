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
pub mod channels;

extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(code: i32) -> !;
    fn malloc(bytes: usize) -> *mut u8;
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
    write(2, msg, len);
    exit(101)
}

/// A proven site failed in the proof-checking build (`--assert-proofs`):
/// Precog claimed something that was not true. Its own exit code, so a
/// harness can tell a wrong proof from an ordinary check.
///
/// # Safety
/// As `cyborg_check_failed`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_proof_failed(msg: *const u8, len: usize) -> ! {
    write(2, msg, len);
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
        let to = probe(m, s, k, h, &mut found);
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
