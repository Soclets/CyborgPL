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

/// Run `kernel` of the library `lib` (Apple's compiler made it when the
/// program was built) as `threads` copies on the GPU, with these bindings,
/// and wait for it. `at` is the block's place, for a stop.
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
    let Some(device) = MTLCreateSystemDefaultDevice() else {
        stop(at, at_len, "this machine has no Metal GPU");
    };
    // The library, read where it lies in the program.
    let data = dispatch2::DispatchData::from_static_bytes(core::slice::from_raw_parts(lib, lib_len as usize));
    let library = match device.newLibraryWithData_error(&data) {
        Ok(l) => l,
        Err(e) => stop(at, at_len, &format!("Metal could not read the block's library: {}", e.localizedDescription())),
    };
    let name = core::str::from_utf8(core::slice::from_raw_parts(kernel, kernel_len as usize)).unwrap_or("");
    let Some(function) = library.newFunctionWithName(&NSString::from_str(name)) else {
        stop(at, at_len, &format!("the library has no kernel `{}`", name));
    };
    let pipeline = match device.newComputePipelineStateWithFunction_error(&function) {
        Ok(p) => p,
        Err(e) => stop(at, at_len, &format!("Metal could not prepare `{}`: {}", name, e.localizedDescription())),
    };
    let Some(queue) = device.newCommandQueue() else {
        stop(at, at_len, "Metal gave no command queue");
    };
    let Some(commands) = queue.commandBuffer() else {
        stop(at, at_len, "Metal gave no command buffer");
    };
    let Some(encoder) = commands.computeCommandEncoder() else {
        stop(at, at_len, "Metal gave no compute encoder");
    };
    encoder.setComputePipelineState(&pipeline);
    // Each binding: a lent list or struct as a buffer the GPU shares; a
    // copied value as a `constant`.
    let bindings = if count == 0 { &[][..] } else { core::slice::from_raw_parts(bindings, count as usize) };
    let mut back = Vec::new();
    let mut one = 0u8;
    for b in bindings {
        let ptr = if b.data.is_null() || b.bytes == 0 { NonNull::from(&mut one).cast() } else { NonNull::new_unchecked(b.data).cast() };
        let len = (b.bytes as usize).max(1);
        if b.kind == 2 {
            encoder.setBytes_length_atIndex(ptr, len, b.slot as usize);
            continue;
        }
        let Some(buffer) = device.newBufferWithBytes_length_options(ptr, len, MTLResourceOptions::StorageModeShared) else {
            stop(at, at_len, "Metal gave no buffer");
        };
        encoder.setBuffer_offset_atIndex(Some(&buffer), 0, b.slot as usize);
        if b.kind == 1 && b.bytes > 0 {
            back.push((buffer, b.data, b.bytes as usize));
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
}
