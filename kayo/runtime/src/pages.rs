//! Page-aligned memory for lists (owner, 2026-10-25; `design/gpu.md`):
//! Precog's choice for a list that reaches a GPU block, and
//! `std::list.page_aligned`'s at every size. Metal shares memory with no
//! copy only when it starts on a page and is whole pages long.

extern "C" {
    fn getpagesize() -> i32;
    fn malloc_size(p: *const u8) -> usize;
    fn posix_memalign(p: *mut *mut u8, align: usize, size: usize) -> i32;
    fn malloc(bytes: usize) -> *mut u8;
    fn realloc(p: *mut u8, bytes: usize) -> *mut u8;
    fn free(p: *mut u8);
}

fn page() -> usize {
    // SAFETY: getpagesize has no preconditions.
    unsafe { getpagesize() as usize }
}

/// Page-aligned memory, whole pages long.
unsafe fn paged(bytes: usize) -> *mut u8 {
    let page = page();
    let mut p = core::ptr::null_mut();
    if posix_memalign(&mut p, page, bytes.div_ceil(page).max(1) * page) != 0 {
        crate::out_of_memory();
    }
    p
}

/// Memory for a list Precog gives page-aligned memory (owner, 2026-10-25),
/// or one written `std::list.page_aligned` (`always`): from a page's size
/// on, it starts on a page and is whole pages long; smaller, Precog's is
/// malloc's, as copying a list that small to the GPU is cheaper.
///
/// # Safety
/// As `cyborg_alloc`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_alloc_paged(bytes: u64, always: bool) -> *mut u8 {
    if bytes == 0 {
        return core::ptr::null_mut();
    }
    if always || bytes as usize >= page() {
        return paged(bytes as usize);
    }
    crate::cyborg_alloc(bytes)
}

/// The same, zeroed, for `list.fill` of a zero: calloc's own pages when they
/// start on a page, as they do from a page's size on here.
///
/// # Safety
/// As `cyborg_alloc_zeroed`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_alloc_zeroed_paged(bytes: u64, always: bool) -> *mut u8 {
    if bytes == 0 {
        return core::ptr::null_mut();
    }
    let page = page();
    if !(always || bytes as usize >= page) {
        return crate::cyborg_alloc_zeroed(bytes);
    }
    let whole = (bytes as usize).div_ceil(page) * page;
    let p = crate::cyborg_alloc_zeroed(whole as u64);
    if p as usize % page == 0 {
        return p;
    }
    free(p);
    let q = paged(whole);
    core::ptr::write_bytes(q, 0, whole);
    q
}

/// A list's three words, as codegen lays them out.
#[repr(C)]
pub struct ListWords {
    pub data: *mut u8,
    pub len: u64,
    pub cap: u64,
}

/// A list Precog gives page-aligned memory, moved there if it holds a page
/// or more (or `always`) and is not there yet: a value that came some way
/// Precog did not make, a function's result, a copy.
///
/// # Safety
/// `list` is a list the program owns, of elements `esize` bytes each.
#[no_mangle]
pub unsafe extern "C" fn cyborg_list_rehome(list: *mut ListWords, esize: u64, always: bool) {
    let l = &mut *list;
    let page = page();
    let held = (l.cap * esize) as usize;
    if l.data.is_null() || !(always || held >= page) {
        return;
    }
    if l.data as usize % page == 0 && malloc_size(l.data) >= held.div_ceil(page) * page {
        return;
    }
    let to = paged(held);
    core::ptr::copy_nonoverlapping(l.data, to, (l.len * esize) as usize);
    free(l.data);
    l.data = to;
}

/// The same, grown: realloc first, which keeps a block of a page or more
/// on a page here, and moved to a page only when it did not.
///
/// # Safety
/// As `cyborg_realloc`; `p` came from these or from `cyborg_alloc`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_realloc_paged(p: *mut u8, bytes: u64, always: bool) -> *mut u8 {
    let page = page();
    let bytes = bytes as usize;
    if !(always || bytes >= page) {
        return crate::cyborg_realloc(p, bytes as u64);
    }
    let whole = bytes.div_ceil(page).max(1) * page;
    let q = if p.is_null() { malloc(whole) } else { realloc(p, whole) };
    if q.is_null() {
        crate::out_of_memory();
    }
    if q as usize % page == 0 {
        return q;
    }
    let r = paged(whole);
    core::ptr::copy_nonoverlapping(q, r, bytes);
    free(q);
    r
}

