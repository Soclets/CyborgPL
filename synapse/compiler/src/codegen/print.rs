//! `cord::print` and `cord::exit`, the two pieces of cord Synapse builds
//! (owner, 2026-10-09: "`b`, not the entire fucking neuron"). On an OS, a
//! print's pieces go to stdout or stderr in one `writev`, and `exit` is the
//! OS's; with no OS, the bytes go to the `output` the target names and
//! `exit` calls its `exit`, or sleeps when it names none.

use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::AddressSpace;

use super::Cg;
use crate::mir::{Const, Operand};
use crate::types::Ty;

impl<'ctx> Cg<'_, 'ctx> {
    /// Which print a name is: the stream (1 or 2), the useful form, and
    /// whether a newline ends it.
    fn print_shape(name: &str) -> (u64, bool, bool) {
        let fd = if name.ends_with(".stderr") || name.ends_with("standard_error") { 2 } else { 1 };
        (fd, name.contains("useful"), name.contains("auto_newline"))
    }

    /// One print: each value as text, its stored or useful form, then all
    /// of it written at once.
    pub(super) fn emit_print(&mut self, fi: usize, name: &str, args: &[Operand], slots: &[Option<PointerValue<'ctx>>]) -> Option<()> {
        let (fd, useful, newline) = Self::print_shape(name);
        let utf8 = Ty::utf8();
        // (pointer, length without the mark, made here so freed after).
        let mut pieces: Vec<(PointerValue<'ctx>, inkwell::values::IntValue<'ctx>, Option<BasicValueEnum<'ctx>>)> = Vec::new();
        let mask = self.ctx.i64_type().const_int(!(1u64 << 63), false);
        for a in args {
            let v = self.operand(fi, a, slots)?;
            let is_text = match a {
                Operand::Const(Const::Str(_)) => true,
                Operand::Const(_) => false,
                _ => matches!(self.operand_cy_ty(fi, a).as_ref().map(|t| t.base().clone()), Some(Ty::Str(_))),
            };
            let (t, made) = if is_text { (v, None) } else {
                let t = self.to_text_as(fi, a, v, &utf8, useful)?;
                (t, Some(t))
            };
            let (p, n) = self.text_parts_pub(t)?;
            let n = self.builder.build_and(n, mask, "len").ok()?;
            pieces.push((p, n, made));
        }
        if newline {
            let t = self.const_text("\n");
            let (p, n) = self.text_parts_pub(t)?;
            let n = self.builder.build_and(n, mask, "len").ok()?;
            pieces.push((p, n, None));
        }
        if crate::target::bare() {
            self.cannot("`cord::print` with no OS is not built yet: its bytes go to the target's `output`, next", None);
            return None;
        }
        // One `writev`: the pieces as C's `struct iovec`s.
        let p = self.ctx.ptr_type(AddressSpace::default());
        let i64t = self.ctx.i64_type();
        let i32t = self.ctx.i32_type();
        let iovec = self.ctx.struct_type(&[p.into(), i64t.into()], false);
        let n = pieces.len().max(1) as u32;
        let table = self.stack(iovec.array_type(n), "iov").ok()?;
        for (k, (ptr, len, _)) in pieces.iter().enumerate() {
            let zero = i32t.const_zero();
            let at = unsafe { self.builder.build_gep(iovec.array_type(n), table, &[zero, i32t.const_int(k as u64, false)], "iov_k").ok()? };
            let pp = self.builder.build_struct_gep(iovec, at, 0, "base").ok()?;
            let _ = self.builder.build_store(pp, *ptr);
            let lp = self.builder.build_struct_gep(iovec, at, 1, "len").ok()?;
            let _ = self.builder.build_store(lp, *len);
        }
        if !pieces.is_empty() {
            let writev = self.libc("writev", i64t.fn_type(&[i32t.into(), p.into(), i32t.into()], false));
            let args: Vec<BasicMetadataValueEnum> = vec![i32t.const_int(fd, false).into(), table.into(), i32t.const_int(pieces.len() as u64, false).into()];
            let _ = self.builder.build_call(writev, &args, "");
        }
        for (_, _, made) in pieces {
            if let Some(t) = made {
                self.free_text(t);
            }
        }
        Some(())
    }

    /// `cord::exit[code]`: the OS's `exit`; with no OS, the board's, or the
    /// CPU sleeps as when MAIN ends (owner, 2026-10-09: "c").
    pub(super) fn emit_exit(&mut self, fi: usize, args: &[Operand], slots: &[Option<PointerValue<'ctx>>]) -> Option<()> {
        let code = self.operand(fi, args.first()?, slots)?.into_int_value();
        let i32t = self.ctx.i32_type();
        if crate::target::bare() {
            self.cannot("`cord::exit` with no OS is not built yet: it calls the target's `exit`, next", None);
            return None;
        }
        let code = self.builder.build_int_z_extend_or_bit_cast(code, i32t, "code").ok()?;
        let exit = self.libc("exit", self.ctx.void_type().fn_type(&[i32t.into()], false));
        let _ = self.builder.build_call(exit, &[code.into()], "");
        // It never comes back: what the block goes on to do is not reached.
        let func = self.builder.get_insert_block()?.get_parent()?;
        let _ = self.builder.build_unreachable();
        let after = self.ctx.append_basic_block(func, "after_exit");
        self.builder.position_at_end(after);
        Some(())
    }

    /// A C library function the OS has, declared once.
    fn libc(&mut self, name: &str, t: inkwell::types::FunctionType<'ctx>) -> FunctionValue<'ctx> {
        self.module.get_function(name).unwrap_or_else(|| self.module.add_function(name, t, None))
    }
}
