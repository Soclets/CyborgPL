//! Point 3 (owner, 2026-10-25: "How about we let it measure?"): copy against
//! no-copy for an `inline.MSL` binding, on this Mac's GPU. Run with
//! `cargo run --release -j3 --offline -p kayo-metal-bench`; the numbers of
//! 2026-10-25 are in RESULTS.md.
//! A whole block run, as `cyborg_msl_run` does it (device, library, pipeline,
//! queue, encode, commit, wait, copy back), with a one-thread kernel so the
//! transfer shows rather than the work.

use core::ptr::NonNull;
use objc2_foundation::NSString;
use objc2_metal::*;
use std::time::Instant;

const PAGE: usize = 16384;
const RUNS: usize = 31;
const BLOCK_RUNS: usize = 101;
const WARM: usize = 3;

extern "C" {
    fn posix_memalign(p: *mut *mut u8, align: usize, size: usize) -> i32;
    fn malloc(n: usize) -> *mut u8;
    fn realloc(p: *mut u8, n: usize) -> *mut u8;
    fn free(p: *mut u8);
}

fn up(n: usize) -> usize {
    n.div_ceil(PAGE).max(1) * PAGE
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Copy,
    NoCopy,
}

/// One block run; returns (whole run, buffer part) in microseconds.
unsafe fn run(lib: &[u8], data: *mut u8, bytes: usize, mode: Mode, write_back: bool) -> (f64, f64) {
    let t0 = Instant::now();
    let device = MTLCreateSystemDefaultDevice().unwrap();
    let d = dispatch2::DispatchData::from_bytes(lib);
    let library = device.newLibraryWithData_error(&d).unwrap();
    let f = library.newFunctionWithName(&NSString::from_str("touch")).unwrap();
    let pipeline = device.newComputePipelineStateWithFunction_error(&f).unwrap();
    let queue = device.newCommandQueue().unwrap();
    let cb = queue.commandBuffer().unwrap();
    let enc = cb.computeCommandEncoder().unwrap();
    enc.setComputePipelineState(&pipeline);
    let tb = Instant::now();
    let buffer = match mode {
        Mode::Copy => device.newBufferWithBytes_length_options(NonNull::new_unchecked(data).cast(), bytes, MTLResourceOptions::StorageModeShared).unwrap(),
        Mode::NoCopy => device.newBufferWithBytesNoCopy_length_options_deallocator(NonNull::new_unchecked(data).cast(), up(bytes), MTLResourceOptions::StorageModeShared, None).unwrap(),
    };
    let mut buf_us = tb.elapsed().as_secs_f64() * 1e6;
    enc.setBuffer_offset_atIndex(Some(&buffer), 0, 0);
    enc.dispatchThreads_threadsPerThreadgroup(MTLSize { width: 1, height: 1, depth: 1 }, MTLSize { width: 1, height: 1, depth: 1 });
    enc.endEncoding();
    cb.commit();
    cb.waitUntilCompleted();
    if mode == Mode::Copy && write_back {
        let tc = Instant::now();
        core::ptr::copy_nonoverlapping(buffer.contents().as_ptr() as *const u8, data, bytes);
        buf_us += tc.elapsed().as_secs_f64() * 1e6;
    }
    drop(buffer);
    (t0.elapsed().as_secs_f64() * 1e6, buf_us)
}

fn stats(mut v: Vec<f64>) -> (f64, f64) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (v[v.len() / 2], v[0])
}

unsafe fn aligned(n: usize) -> *mut u8 {
    let mut p = core::ptr::null_mut();
    assert_eq!(posix_memalign(&mut p, PAGE, up(n)), 0);
    p
}

fn main() {
    let lib: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/kernel.metallib"));
    let sizes: [(usize, &str); 6] = [(1 << 10, "1 KB"), (4 << 10, "4 KB"), (16 << 10, "16 KB"), (64 << 10, "64 KB"), (1 << 20, "1 MB"), (64 << 20, "64 MB")];
    println!("## Block runs ({} runs each after {} warm-up, the four ways interleaved, median / best, microseconds)", BLOCK_RUNS, WARM);
    println!("| size | copy, lend | no-copy, lend | copy, lend.mut | no-copy, lend.mut | buffer part: copy lend / no-copy lend / copy lend.mut / no-copy lend.mut |");
    println!("|---|---|---|---|---|---|");
    unsafe {
        for (n, name) in sizes {
            // The program's list: from malloc for copy, page-aligned for no-copy;
            // both written first, as a list the program filled.
            let plain = malloc(n);
            let pa = aligned(n);
            for i in 0..n {
                *plain.add(i) = i as u8;
                *pa.add(i) = i as u8;
            }
            let modes = [(Mode::Copy, false), (Mode::NoCopy, false), (Mode::Copy, true), (Mode::NoCopy, true)];
            let mut whole = vec![Vec::new(); 4];
            let mut part = vec![Vec::new(); 4];
            for _ in 0..WARM {
                for (mode, wb) in modes {
                    run(lib, if mode == Mode::Copy { plain } else { pa }, n, mode, wb);
                }
            }
            for _ in 0..BLOCK_RUNS {
                for (k, (mode, wb)) in modes.into_iter().enumerate() {
                    let p = if mode == Mode::Copy { plain } else { pa };
                    let before = *p;
                    let (w, b) = run(lib, p, n, mode, wb);
                    // The kernel's write reached the program's memory exactly when it should.
                    assert_eq!(*p != before, wb || mode == Mode::NoCopy, "write-back mismatch at {}", name);
                    whole[k].push(w);
                    part[k].push(b);
                }
            }
            let cells: Vec<String> = whole.into_iter().map(|v| { let (m, b) = stats(v); format!("{:.0} / {:.0}", m, b) }).collect();
            let parts: Vec<String> = part.into_iter().map(|v| format!("{:.1}", stats(v).0)).collect();
            println!("| {} | {} | {} |", name, cells.join(" | "), parts.join(" / "));
            free(plain);
            free(pa);
        }

        // What page alignment costs a list.
        println!();
        println!("## Allocating and growing ({} runs, median, microseconds)", RUNS);
        println!("| size | malloc | page-aligned | grow by doubling from 8 B: realloc | grow: page-aligned (new, copy, free) | malloc already page-aligned? |");
        println!("|---|---|---|---|---|---|");
        for (n, name) in sizes {
            let time = |f: &mut dyn FnMut()| {
                let mut v = Vec::new();
                for _ in 0..RUNS {
                    let t = Instant::now();
                    f();
                    v.push(t.elapsed().as_secs_f64() * 1e6);
                }
                stats(v).0
            };
            let a = time(&mut || {
                let p = malloc(n);
                core::ptr::write_volatile(p, 1);
                free(p);
            });
            let b = time(&mut || {
                let p = aligned(n);
                core::ptr::write_volatile(p, 1);
                free(p);
            });
            let c = time(&mut || {
                let mut cap = 8;
                let mut p = malloc(cap);
                *p = 1;
                while cap < n {
                    cap *= 2;
                    p = realloc(p, cap);
                    core::ptr::write_volatile(p.add(cap - 1), 1);
                }
                free(p);
            });
            let d = time(&mut || {
                let mut cap = 8;
                let mut p = aligned(cap);
                *p = 1;
                while cap < n {
                    let old = cap;
                    cap *= 2;
                    let q = aligned(cap);
                    core::ptr::copy_nonoverlapping(p, q, old);
                    free(p);
                    p = q;
                    core::ptr::write_volatile(p.add(cap - 1), 1);
                }
                free(p);
            });
            // Held together, so malloc cannot hand back the block just freed.
            let held: Vec<*mut u8> = (0..16).map(|_| malloc(n)).collect();
            let hits = held.iter().filter(|p| **p as usize % PAGE == 0).count();
            for p in held {
                free(p);
            }
            println!("| {} | {:.2} | {:.2} | {:.2} | {:.2} | {}/16 |", name, a, b, c, d, hits);
        }

        println!();
        println!("## Memory a page-aligned list takes (page {} B)", PAGE);
        println!("| list | rounded to | wasted |");
        println!("|---|---|---|");
        for n in [8usize, 64, 1 << 10, 4 << 10, 16 << 10, (16 << 10) + 1, 1 << 20] {
            println!("| {} B | {} B | {} B ({:.0}%) |", n, up(n), up(n) - n, (up(n) - n) as f64 * 100.0 / up(n) as f64);
        }
    }
}
