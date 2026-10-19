//! A `list[bool]` held as bits (`mir::bits`): element `i` is bit
//! `(i - 1) mod 8` of byte `(i - 1) / 8`. The length and the capacity stay
//! counts of elements, so `list.length`, the bounds checks and the drop
//! read what they always did.

use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::{AtomicOrdering, AtomicRMWBinOp, IntPredicate};

use super::{basic_result, Cg};
use crate::mir::*;
use crate::types::Ty;

impl<'a, 'ctx> Cg<'a, 'ctx> {
    /// `'xs'[i]` of a list held as bits.
    pub(super) fn is_bit(&self, fi: usize, p: &Place) -> bool {
        matches!(p.proj.as_slice(), [Proj::Index(_)]) && self.bits.contains(&(fi, p.local.0))
    }

    /// The byte element `i` sits in, and the one bit of it that is `i`.
    fn bit_at(&mut self, fi: usize, p: &Place, slots: &[Option<PointerValue<'ctx>>]) -> Option<(PointerValue<'ctx>, IntValue<'ctx>, IntValue<'ctx>)> {
        let [Proj::Index(o)] = p.proj.as_slice() else { return None };
        let at = slots[p.local.0 as usize]?;
        let ty = self.p.functions[fi].locals[p.local.0 as usize].ty.clone();
        let BasicTypeEnum::StructType(shape) = self.ty(&ty)? else { return None };
        let i = self.operand(fi, o, slots)?.into_int_value();
        let w = self.ctx.i64_type();
        let b = self.ctx.i8_type();
        // Checked to be 1 to the length at its site.
        let i = self.builder.build_int_cast_sign_flag(i, w, false, "i").ok()?;
        let zb = self.builder.build_int_sub(i, w.const_int(1, false), "zb").ok()?;
        let byte = self.builder.build_right_shift(zb, w.const_int(3, false), false, "byte").ok()?;
        let data = self.load_field_ptr(shape, at, 0);
        let slot = unsafe { self.builder.build_gep(b, data, &[byte], "bits").ok()? };
        let shift = self.builder.build_and(zb, w.const_int(7, false), "bit").ok()?;
        let shift = self.builder.build_int_truncate(shift, b, "bit").ok()?;
        let mask = self.builder.build_left_shift(b.const_int(1, false), shift, "mask").ok()?;
        Some((slot, shift, mask))
    }

    pub(super) fn bit_read(&mut self, fi: usize, p: &Place, slots: &[Option<PointerValue<'ctx>>]) -> Option<BasicValueEnum<'ctx>> {
        let (slot, shift, _) = self.bit_at(fi, p, slots)?;
        let b = self.ctx.i8_type();
        let byte = self.builder.build_load(b, slot, "byte").ok()?.into_int_value();
        // In a slice, the slice beside it may be setting another bit of
        // this byte: an atomic read, a plain one on the machine.
        if self.par_child.is_some() {
            let load = byte.as_instruction()?;
            load.set_alignment(1).ok()?;
            load.set_atomic_ordering(AtomicOrdering::Monotonic).ok()?;
        }
        let down = self.builder.build_right_shift(byte, shift, false, "down").ok()?;
        Some(self.builder.build_int_truncate(down, self.ctx.bool_type(), "elem").ok()?.into())
    }

    /// A write sets or clears its own bit and leaves the other seven as
    /// they were. In a parallel `for`'s slice, the slice beside it may be
    /// writing another bit of the same byte, so there it is one atomic
    /// `or` or `and`: each element is still written by one slice only.
    pub(super) fn bit_write(&mut self, fi: usize, p: &Place, v: BasicValueEnum<'ctx>, slots: &[Option<PointerValue<'ctx>>]) -> Option<()> {
        let (slot, shift, mask) = self.bit_at(fi, p, slots)?;
        let b = self.ctx.i8_type();
        let v = v.into_int_value();
        let one = if v.get_type().get_bit_width() == 1 { v } else { self.builder.build_int_compare(IntPredicate::NE, v, v.get_type().const_zero(), "set").ok()? };
        let set = self.builder.build_int_z_extend(one, b, "set").ok()?;
        let set = self.builder.build_left_shift(set, shift, "set").ok()?;
        let keep = self.builder.build_not(mask, "keep").ok()?;
        if self.par_child.is_some() {
            // A value known as it is written (`= true`, as a sieve does)
            // is one `or` or one `and`; one not known clears, then sets.
            let known = one.get_zero_extended_constant();
            if known != Some(1) {
                let _ = self.builder.build_atomicrmw(AtomicRMWBinOp::And, slot, keep, AtomicOrdering::Monotonic).ok()?;
            }
            if known != Some(0) {
                let _ = self.builder.build_atomicrmw(AtomicRMWBinOp::Or, slot, set, AtomicOrdering::Monotonic).ok()?;
            }
            return Some(());
        }
        let byte = self.builder.build_load(b, slot, "byte").ok()?.into_int_value();
        let kept = self.builder.build_and(byte, keep, "kept").ok()?;
        let next = self.builder.build_or(kept, set, "byte").ok()?;
        let _ = self.builder.build_store(slot, next);
        Some(())
    }

    /// `list.fill[v, count]` into a list held as bits: a byte for every
    /// eight elements, all of them `v`. Bits past the count are never read.
    pub(super) fn bit_fill(&mut self, fi: usize, args: &[Operand], out: &Ty, slots: &[Option<PointerValue<'ctx>>], _func: FunctionValue<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let BasicTypeEnum::StructType(shape) = self.ty(out)? else { return None };
        let w = self.ctx.i64_type();
        let b = self.ctx.i8_type();
        let c = self.operand(fi, args.get(1)?, slots)?.into_int_value();
        let signed = self.operand_signed(fi, args.get(1)?).unwrap_or(true);
        let n = self.builder.build_int_cast_sign_flag(c, w, signed, "count").ok()?;
        let whole = self.builder.build_right_shift(n, w.const_int(3, false), false, "bytes").ok()?;
        let part = self.builder.build_and(n, w.const_int(7, false), "part").ok()?;
        let part = self.builder.build_int_compare(IntPredicate::NE, part, w.const_zero(), "part").ok()?;
        let part = self.builder.build_int_z_extend(part, w, "part").ok()?;
        let bytes = self.builder.build_int_add(whole, part, "bytes").ok()?;
        let data = match args.first()? {
            Operand::Const(Const::Bool(false)) => {
                let f = self.rt("cyborg_alloc_zeroed");
                basic_result(self.builder.build_call(f, &[bytes.into()], "zeroed").ok()?)?.into_pointer_value()
            }
            v => {
                let f = self.rt("cyborg_alloc");
                let room = basic_result(self.builder.build_call(f, &[bytes.into()], "room").ok()?)?.into_pointer_value();
                let v = self.operand(fi, v, slots)?.into_int_value();
                let one = if v.get_type().get_bit_width() == 1 { v } else { self.builder.build_int_compare(IntPredicate::NE, v, v.get_type().const_zero(), "set").ok()? };
                let fill = self.builder.build_select(one, b.const_int(0xFF, false), b.const_zero(), "fill").ok()?.into_int_value();
                let _ = self.builder.build_memset(room, 1, fill, bytes);
                room
            }
        };
        let mut l = shape.get_undef();
        l = self.builder.build_insert_value(l, data, 0, "data").ok()?.into_struct_value();
        l = self.builder.build_insert_value(l, n, 1, "len").ok()?.into_struct_value();
        l = self.builder.build_insert_value(l, n, 2, "cap").ok()?.into_struct_value();
        Some(l.into())
    }
}
