//! C strings (owner, 2026-10-19; `design/ffi.md` § C strings):
//! `C.str.make` makes a text's bytes and a 0, a `list[u8]` the program
//! owns and lends to C as a pointer to its first byte; `C.str.read` reads a
//! C string up to its 0 into a text. The work is the runtime's
//! (`cyborg_cstr_make`, `cyborg_cstr_read`).

use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::IntPredicate;

use super::{basic_result, Cg, LlBlock};
use crate::mir::*;
use crate::types::Ty;

/// A length's top bit marks a text the binary holds; the runtime clears it.
const STATIC: u64 = 1 << 63;

impl<'a, 'ctx> Cg<'a, 'ctx> {
    /// The text an argument names: its bytes and its length.
    fn cstr_text(&mut self, fi: usize, arg: &Arg, slots: &[Option<PointerValue<'ctx>>]) -> Option<(PointerValue<'ctx>, inkwell::values::IntValue<'ctx>)> {
        let v = match arg {
            Arg::Value(o) => self.operand(fi, o, slots)?,
            Arg::Lend { place, .. } | Arg::Copy(place) | Arg::Give(place) => self.operand(fi, &Operand::Read(place.clone()), slots)?,
            Arg::Spread { .. } => return None,
        };
        let BasicValueEnum::StructValue(t) = v else { return None };
        let ptr = self.builder.build_extract_value(t, 0, "bytes").ok()?.into_pointer_value();
        let len = self.builder.build_extract_value(t, 1, "len").ok()?.into_int_value();
        Some((ptr, len))
    }

    /// `C.str.make`: the C string, and whether the text held a 0 byte.
    pub(super) fn cstr_make(&mut self, fi: usize, arg: &Arg, out: &Ty, slots: &[Option<PointerValue<'ctx>>]) -> Option<(BasicValueEnum<'ctx>, inkwell::values::IntValue<'ctx>)> {
        let (src, len) = self.cstr_text(fi, arg, slots)?;
        let p = self.ctx.ptr_type(inkwell::AddressSpace::default());
        let w = self.ctx.i64_type();
        let at = self.stack(p, "cstr").ok()?;
        let f = self.rt("cyborg_cstr_make");
        let status = basic_result(self.builder.build_call(f, &[at.into(), src.into(), len.into()], "status").ok()?)?.into_int_value();
        let bad = self.builder.build_int_compare(IntPredicate::NE, status, self.ctx.i8_type().const_zero(), "has_zero_byte").ok()?;
        let data = self.builder.build_load(p, at, "cstr").ok()?;
        let n = self.builder.build_and(len, w.const_int(!STATIC, false), "len").ok()?;
        let n = self.builder.build_int_add(n, w.const_int(1, false), "with_zero").ok()?;
        let BasicTypeEnum::StructType(shape) = self.ty(out)? else { return None };
        let mut l = shape.get_undef();
        l = self.builder.build_insert_value(l, data, 0, "data").ok()?.into_struct_value();
        l = self.builder.build_insert_value(l, n, 1, "len").ok()?.into_struct_value();
        l = self.builder.build_insert_value(l, n, 2, "cap").ok()?.into_struct_value();
        Some((l.into(), bad))
    }

    /// `C.str.read`: where the C string starts.
    fn cstr_start(&mut self, fi: usize, arg: &Arg, slots: &[Option<PointerValue<'ctx>>]) -> Option<PointerValue<'ctx>> {
        match arg {
            // A pointer C handed over.
            Arg::Value(o) if matches!(self.operand(fi, o, slots), Some(BasicValueEnum::PointerValue(_))) => self.operand(fi, o, slots).map(|v| v.into_pointer_value()),
            // A C string the program made: its first byte.
            Arg::Value(Operand::Read(place) | Operand::Move(place)) | Arg::Lend { place, .. } | Arg::Copy(place) | Arg::Give(place) => {
                let (slot, ty) = self.slot_of(fi, place, slots)?;
                let BasicTypeEnum::StructType(shape) = self.ty(&ty)? else {
                    // A pointer held in a place.
                    return Some(self.operand(fi, &Operand::Read(place.clone()), slots)?.into_pointer_value());
                };
                Some(self.load_field_ptr(shape, slot, 0))
            }
            _ => None,
        }
    }

    /// A `C.str.make` or `C.str.read` that can fail: the answer into its
    /// place on the way to `ok`, or the failure's arm.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn cstr_call(&mut self, fi: usize, op: &Op, args: &[Arg], dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let arg = args.first()?;
        let arm_of = |i: usize| fail.iter().find(|a| a.index == i).or_else(|| fail.first()).map(|a| blocks[a.block.0 as usize]);
        let good = self.ctx.append_basic_block(func, "cstr_ok");
        match op {
            Op::CStrMake => {
                let out = match dest {
                    Some(d) => self.slot_of(fi, d, slots)?.1,
                    None => Ty::Seq { elem: Box::new(Ty::int(false, 8)), kind: crate::types::SeqKind::CStr(true) },
                };
                let (l, bad) = self.cstr_make(fi, arg, &out, slots)?;
                // A bounded `C.str` (owner, 2026-10-19): whether the text
                // can hold a 0 byte (so `has_zero_byte` is reason 0), and the
                // length with its 0 that fits.
                let bounded = match (args.get(1), args.get(2), args.get(3)) {
                    (Some(Arg::Value(Operand::Const(Const::Bool(z)))), Some(Arg::Value(Operand::Const(Const::Int(lo)))), Some(Arg::Value(Operand::Const(Const::Int(hi))))) => Some((*z, *lo as u64, *hi as u64)),
                    _ => None,
                };
                let zero_arm = arm_of(0)?;
                if let Some((zero_reason, lo, hi)) = bounded {
                    if zero_reason {
                        let made = self.ctx.append_basic_block(func, "cstr_made");
                        let _ = self.builder.build_conditional_branch(bad, zero_arm, made);
                        self.builder.position_at_end(made);
                    }
                    let n = self.builder.build_extract_value(l.into_struct_value(), 1, "len").ok()?.into_int_value();
                    let w = self.ctx.i64_type();
                    let short = self.builder.build_int_compare(IntPredicate::ULT, n, w.const_int(lo, false), "short").ok()?;
                    let long = self.builder.build_int_compare(IntPredicate::UGT, n, w.const_int(hi, false), "long").ok()?;
                    let out_ = self.builder.build_or(short, long, "out_of_range").ok()?;
                    let off = self.ctx.append_basic_block(func, "cstr_out_of_range");
                    let _ = self.builder.build_conditional_branch(out_, off, good);
                    // Too long or too short: what was made is given back.
                    self.builder.position_at_end(off);
                    let data = self.builder.build_extract_value(l.into_struct_value(), 0, "data").ok()?.into_pointer_value();
                    self.free_mem(data);
                    let _ = self.builder.build_unconditional_branch(arm_of(if zero_reason { 1 } else { 0 })?);
                } else {
                    let _ = self.builder.build_conditional_branch(bad, zero_arm, good);
                }
                self.builder.position_at_end(good);
                if let Some(d) = dest {
                    self.store_place(fi, d, l, slots);
                    if d.proj.is_empty() {
                        self.mark_alive(d.local, true);
                    }
                }
            }
            Op::CStrRead => {
                let arm = arm_of(0)?;
                let src = self.cstr_start(fi, arg, slots)?;
                let text_ty = self.ty(&crate::types::Ty::utf8())?;
                let out = self.stack(text_ty, "text").ok()?;
                let f = self.rt("cyborg_cstr_read");
                let status = basic_result(self.builder.build_call(f, &[out.into(), src.into()], "status").ok()?)?.into_int_value();
                let bad = self.builder.build_int_compare(IntPredicate::NE, status, self.ctx.i8_type().const_zero(), "not_utf8").ok()?;
                let _ = self.builder.build_conditional_branch(bad, arm, good);
                self.builder.position_at_end(good);
                if let Some(d) = dest {
                    let t = self.builder.build_load(text_ty, out, "t").ok()?;
                    self.store_place(fi, d, t, slots);
                    if d.proj.is_empty() {
                        self.mark_alive(d.local, true);
                    }
                }
            }
            _ => return None,
        }
        let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
        Some(())
    }
}
