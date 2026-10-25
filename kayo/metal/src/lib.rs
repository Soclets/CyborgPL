//! The runtime with Apple's GPU: every function of `kayo-runtime`, built
//! beside std, and `cyborg_msl_run`, which runs an `inline.MSL` block
//! (owner, 2026-10-01, 2026-10-22; `design/gpu.md`). Reached through the
//! Objective-C runtime by objc2 (owner, 2026-10-25: "(b)"). A program
//! links this in place of `libkayo_runtime.a` only when it holds a block.

pub use kayo_runtime;

/// One value bound to a buffer slot, as codegen lays it out.
#[repr(C)]
pub struct MslBinding {
    pub slot: u32,
    /// 0 lent, 1 lent to change, 2 a Metal `constant`.
    pub kind: u32,
    pub data: *mut u8,
    pub bytes: u64,
    /// How many bytes its memory holds from `data` (a list's capacity), or
    /// 0 for what is copied whatever its memory (a struct, an array,
    /// `std::list.not_page_aligned`).
    pub held: u64,
}

/// Stops the program, saying the GPU could not run the block, and where.
unsafe fn stop(at: *const u8, at_len: u64, why: &str) -> ! {
    use std::io::Write;
    let at = if at.is_null() { &[][..] } else { core::slice::from_raw_parts(at, at_len as usize) };
    let mut err = std::io::stderr();
    let _ = err.write_all(b"error[metal::run_failed] ");
    let _ = err.write_all(at);
    let _ = writeln!(err, "about: {}", why);
    std::process::exit(1)
}

extern "C" {
    fn getpagesize() -> i32;
    fn malloc_size(p: *const u8) -> usize;
}

fn page() -> usize {
    // SAFETY: getpagesize has no preconditions.
    unsafe { getpagesize() as usize }
}

/// The bytes Metal shares with no copy for this binding, if it can: memory
/// that starts on a page and holds whole pages past the list's bytes
/// (owner, 2026-10-25: "a list goes to the GPU without a copy when its
/// memory is page-aligned"). The pages past the list are the list's own
/// capacity, or what its allocation holds past it, so the GPU sees nothing
/// the program does not hold.
fn shared_span(b: &MslBinding) -> Option<usize> {
    if b.kind == 2 || b.data.is_null() || b.bytes == 0 {
        return None;
    }
    let page = page();
    let span = (b.bytes as usize).div_ceil(page) * page;
    if b.data as usize % page != 0 {
        return None;
    }
    // A held of 0 says copy (an array, `std::list.not_page_aligned`).
    // SAFETY: malloc_size answers 0 for memory malloc did not give.
    (b.held != 0 && (b.held as usize >= span || unsafe { malloc_size(b.data) } >= span)).then_some(span)
}

#[cfg(target_os = "macos")]
mod gpu {
    use objc2::rc::Retained;
    use objc2::runtime::ProtocolObject;
    use objc2_metal::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    pub type Device = Retained<ProtocolObject<dyn MTLDevice>>;
    pub type Queue = Retained<ProtocolObject<dyn MTLCommandQueue>>;
    pub type Pipeline = Retained<ProtocolObject<dyn MTLComputePipelineState>>;

    thread_local! {
        /// The GPU and its queue, found once on each thread that runs a block.
        pub static GPU: RefCell<Option<(Device, Queue)>> = const { RefCell::new(None) };
        /// Each block's pipeline, made once: the block's library is a
        /// constant of the program, so its address names the block.
        pub static PIPELINES: RefCell<HashMap<usize, Pipeline>> = RefCell::new(HashMap::new());
    }
}

/// Run `kernel` of the library `lib` (Apple's compiler made it when the
/// program was built) as `threads` copies on the GPU, with these bindings,
/// and wait for it. `at` is the block's place, for a stop. The GPU, its
/// queue and the block's pipeline are kept for the block's next run.
///
/// # Safety
/// The pointers are codegen's: the library and the name in the program's
/// constants, each binding's memory the program's own.
#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn cyborg_msl_run(lib: *const u8, lib_len: u64, kernel: *const u8, kernel_len: u64, threads: u64, bindings: *const MslBinding, count: u64, at: *const u8, at_len: u64) {
    use core::ptr::NonNull;
    use objc2_foundation::NSString;
    use objc2_metal::*;
    if threads == 0 {
        return;
    }
    let (device, queue) = gpu::GPU.with(|g| {
        let mut g = g.borrow_mut();
        if g.is_none() {
            let Some(device) = MTLCreateSystemDefaultDevice() else {
                stop(at, at_len, "this machine has no Metal GPU");
            };
            let Some(queue) = device.newCommandQueue() else {
                stop(at, at_len, "Metal gave no command queue");
            };
            *g = Some((device, queue));
        }
        g.clone().unwrap()
    });
    let name = core::str::from_utf8(core::slice::from_raw_parts(kernel, kernel_len as usize)).unwrap_or("");
    let pipeline = gpu::PIPELINES.with(|p| {
        if let Some(found) = p.borrow().get(&(lib as usize)) {
            return found.clone();
        }
        // The library, read where it lies in the program.
        let data = dispatch2::DispatchData::from_static_bytes(core::slice::from_raw_parts(lib, lib_len as usize));
        let library = match device.newLibraryWithData_error(&data) {
            Ok(l) => l,
            Err(e) => stop(at, at_len, &format!("Metal could not read the block's library: {}", e.localizedDescription())),
        };
        let Some(function) = library.newFunctionWithName(&NSString::from_str(name)) else {
            stop(at, at_len, &format!("the library has no kernel `{}`", name));
        };
        let pipeline = match device.newComputePipelineStateWithFunction_error(&function) {
            Ok(p) => p,
            Err(e) => stop(at, at_len, &format!("Metal could not prepare `{}`: {}", name, e.localizedDescription())),
        };
        p.borrow_mut().insert(lib as usize, pipeline.clone());
        pipeline
    });
    let Some(commands) = queue.commandBuffer() else {
        stop(at, at_len, "Metal gave no command buffer");
    };
    let Some(encoder) = commands.computeCommandEncoder() else {
        stop(at, at_len, "Metal gave no compute encoder");
    };
    encoder.setComputePipelineState(&pipeline);
    // Each binding: a lent list in page-aligned memory as it is, any other
    // lent value copied into a buffer the GPU shares, and back after
    // `lend.mut`; a copied value as a `constant`.
    let bindings = if count == 0 { &[][..] } else { core::slice::from_raw_parts(bindings, count as usize) };
    let mut back = Vec::new();
    let mut kept = Vec::new();
    let mut one = 0u8;
    for b in bindings {
        let ptr = if b.data.is_null() || b.bytes == 0 { NonNull::from(&mut one).cast() } else { NonNull::new_unchecked(b.data).cast() };
        let len = (b.bytes as usize).max(1);
        if b.kind == 2 {
            encoder.setBytes_length_atIndex(ptr, len, b.slot as usize);
            continue;
        }
        if let Some(span) = shared_span(b) {
            // The program's own memory: the kernel's writes land in it.
            let Some(buffer) = device.newBufferWithBytesNoCopy_length_options_deallocator(ptr, span, MTLResourceOptions::StorageModeShared, None) else {
                stop(at, at_len, "Metal would not share the list's memory");
            };
            encoder.setBuffer_offset_atIndex(Some(&buffer), 0, b.slot as usize);
            kept.push(buffer);
            continue;
        }
        let Some(buffer) = device.newBufferWithBytes_length_options(ptr, len, MTLResourceOptions::StorageModeShared) else {
            stop(at, at_len, "Metal gave no buffer");
        };
        encoder.setBuffer_offset_atIndex(Some(&buffer), 0, b.slot as usize);
        if b.kind == 1 && b.bytes > 0 {
            back.push((buffer, b.data, b.bytes as usize));
        } else {
            kept.push(buffer);
        }
    }
    // `threads` copies, in groups as wide as the pipeline takes.
    let width = pipeline.maxTotalThreadsPerThreadgroup().min(threads as usize).max(1);
    encoder.dispatchThreads_threadsPerThreadgroup(MTLSize { width: threads as usize, height: 1, depth: 1 }, MTLSize { width, height: 1, depth: 1 });
    encoder.endEncoding();
    commands.commit();
    commands.waitUntilCompleted();
    if let Some(e) = commands.error() {
        stop(at, at_len, &format!("the GPU stopped running `{}`: {}", name, e.localizedDescription()));
    }
    // What the kernel wrote, back where the program holds it.
    for (buffer, data, bytes) in back {
        core::ptr::copy_nonoverlapping(buffer.contents().as_ptr() as *const u8, data, bytes);
    }
    // The shared buffers go before the block ends, while the memory they
    // wrap is still the program's.
    drop(kept);
}
