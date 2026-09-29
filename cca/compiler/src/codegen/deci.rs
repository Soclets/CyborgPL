//! `deci` in the generated code (`types.md`, owner 2026-09-29). A value is
//! its layout (BID) in an integer of its width, and every operation on one
//! is a call into the runtime's `deci.rs`: there is no decimal instruction
//! to lower to. What the runtime cannot answer — a result past the largest
//! value, a division by zero — stops the program where it happened, as the
//! oracle does. Precog follows nothing about a decimal yet, so these are
//! not check sites it could prove away: they are always checked.

use inkwell::values::{BasicValueEnum, FunctionValue, IntValue};
use inkwell::IntPredicate;

use super::{basic_result, Cg};
use crate::ast::BinOp;
use crate::types::Ty;

impl<'a, 'ctx> Cg<'a, 'ctx> {
    /// The runtime's decimal entries, by name.
    pub(super) fn rt_deci(&mut self, name: &str) -> Option<inkwell::values::FunctionValue<'ctx>> {
        if let Some(f) = self.module.get_function(name) {
            return Some(f);
        }
        let p = self.ctx.ptr_type(inkwell::AddressSpace::default());
        let (w, i32t, i8t) = (self.ctx.i64_type(), self.ctx.i32_type(), self.ctx.i8_type());
        let t = match name {
            "cyborg_deci_arith" => i8t.fn_type(&[i32t.into(), w.into(), w.into(), w.into(), w.into(), i32t.into(), p.into()], false),
            "cyborg_deci_arith2" => i8t.fn_type(&[i32t.into(), w.into(), w.into(), w.into(), w.into(), i32t.into(), p.into()], false),
            "cyborg_deci_narrow" => i8t.fn_type(&[w.into(), w.into(), i32t.into(), i32t.into(), p.into()], false),
            "cyborg_deci_compare" => i32t.fn_type(&[w.into(), w.into(), w.into(), w.into(), i32t.into()], false),
            "cyborg_deci_from_int" => i8t.fn_type(&[w.into(), w.into(), i32t.into(), p.into()], false),
            "cyborg_deci_from_bin" => i8t.fn_type(&[self.ctx.f64_type().into(), i32t.into(), i32t.into(), p.into()], false),
            "cyborg_deci_parse" => i8t.fn_type(&[p.into(), w.into(), i32t.into(), p.into()], false),
            "cyborg_deci_to_int" => i8t.fn_type(&[w.into(), w.into(), i32t.into(), p.into()], false),
            "cyborg_deci_to_bin" => self.ctx.f64_type().fn_type(&[w.into(), w.into(), i32t.into(), i32t.into()], false),
            "cyborg_deci_hash" => w.fn_type(&[w.into(), w.into(), i32t.into()], false),
            "cyborg_deci_widen" => self.ctx.void_type().fn_type(&[w.into(), w.into(), i32t.into(), i32t.into(), p.into()], false),
            "cyborg_deci_to_text" => self.ctx.void_type().fn_type(&[p.into(), w.into(), w.into(), i32t.into(), i32t.into()], false),
            _ => return None,
        };
        Some(self.module.add_function(name, t, None))
    }

    /// The two 64-bit halves of a value's layout, as the runtime takes it.
    pub(super) fn deci_halves(&mut self, v: IntValue<'ctx>) -> Option<(IntValue<'ctx>, IntValue<'ctx>)> {
        let w = self.ctx.i128_type();
        let i64t = self.ctx.i64_type();
        let x = if v.get_type().get_bit_width() == 128 { v } else { self.builder.build_int_z_extend(v, w, "deci").ok()? };
        let lo = self.builder.build_int_truncate(x, i64t, "lo").ok()?;
        let sh = self.builder.build_right_shift(x, w.const_int(64, false), false, "shifted").ok()?;
        let hi = self.builder.build_int_truncate(sh, i64t, "hi").ok()?;
        Some((lo, hi))
    }

    fn deci_int_type(&self, bits: u16) -> inkwell::types::IntType<'ctx> {
        match bits {
            32 => self.ctx.i32_type(),
            64 => self.ctx.i64_type(),
            _ => self.ctx.i128_type(),
        }
    }

    /// A 128-bit slot for the runtime's answer, read back at the width.
    fn deci_slot(&mut self) -> Option<inkwell::values::PointerValue<'ctx>> {
        let w = self.ctx.i128_type();
        let slot = self.builder.build_alloca(w, "deci_out").ok()?;
        let _ = self.builder.build_store(slot, w.const_zero());
        Some(slot)
    }

    fn deci_read(&mut self, slot: inkwell::values::PointerValue<'ctx>, bits: u16) -> Option<IntValue<'ctx>> {
        let w = self.ctx.i128_type();
        let x = self.builder.build_load(w, slot, "deci").ok()?.into_int_value();
        let t = self.deci_int_type(bits);
        if bits == 128 {
            Some(x)
        } else {
            self.builder.build_int_truncate(x, t, "deci").ok()
        }
    }

    /// Stop the program here, saying why, when `bad` holds: the place is
    /// the statement being generated.
    pub(super) fn deci_stop(&mut self, fi: usize, bad: IntValue<'ctx>, why: &str, func: FunctionValue<'ctx>) -> Option<()> {
        let f = &self.p.functions[fi];
        let (b, si) = (self.here.0 as usize, self.here.1 as usize);
        let block = &f.blocks[b];
        let span_id = block.stmts.get(si).map(|s| s.span).unwrap_or(block.term_span);
        let span = f.span(span_id);
        let file = self.sources.get(span.file);
        let (line, col) = file.line_col(span.start);
        let msg = format!("check: {}\nwhere: {}, line {}, column {}\n", why, file.display_name(), line, col);
        let stop = self.ctx.append_basic_block(func, "deci_stopped");
        let go = self.ctx.append_basic_block(func, "deci_ok");
        let _ = self.builder.build_conditional_branch(bad, stop, go);
        self.builder.position_at_end(stop);
        let ptr = self.global_bytes(&msg);
        let len = self.ctx.i64_type().const_int(msg.len() as u64, false);
        if let Some(handler) = self.module.get_function("cyborg_check_failed") {
            let _ = self.builder.build_call(handler, &[ptr.into(), len.into()], "");
        }
        let _ = self.builder.build_unreachable();
        self.builder.position_at_end(go);
        Some(())
    }

    /// `+ - x div` and the comparisons on two `deci`s of `bits`.
    /// `exp_signed`: for `xx`, whether the whole-number exponent is signed.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn deci_binary(&mut self, fi: usize, op: &BinOp, a: BasicValueEnum<'ctx>, b: BasicValueEnum<'ctx>, bits: u16, exp_signed: bool, func: FunctionValue<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let (alo, ahi) = self.deci_halves(a.into_int_value())?;
        let b = if *op == BinOp::Pow {
            // The exponent is a count, taken at its true value.
            self.builder.build_int_cast_sign_flag(b.into_int_value(), self.ctx.i128_type(), exp_signed, "exponent").ok()?
        } else {
            b.into_int_value()
        };
        let (blo, bhi) = self.deci_halves(b)?;
        let i32t = self.ctx.i32_type();
        let width = i32t.const_int(bits as u64, false);
        if op.is_comparison() {
            let f = self.rt_deci("cyborg_deci_compare")?;
            let c = basic_result(self.builder.build_call(f, &[alo.into(), ahi.into(), blo.into(), bhi.into(), width.into()], "order").ok()?)?.into_int_value();
            let pred = match op {
                BinOp::Eq => IntPredicate::EQ,
                BinOp::Ne => IntPredicate::NE,
                BinOp::Lt => IntPredicate::SLT,
                BinOp::Le => IntPredicate::SLE,
                BinOp::Gt => IntPredicate::SGT,
                _ => IntPredicate::SGE,
            };
            return Some(self.builder.build_int_compare(pred, c, i32t.const_zero(), "deci_cmp").ok()?.into());
        }
        let code = match op {
            BinOp::Add => 0,
            BinOp::Sub => 1,
            BinOp::Mul => 2,
            BinOp::Div => 3,
            BinOp::Mod => 4,
            BinOp::Pow => 5,
            _ => {
                self.cannot(format!("`{}` on a deci has no shape in the generated code", op.spelled()), None);
                return None;
            }
        };
        let f = self.rt_deci(if code >= 4 { "cyborg_deci_arith2" } else { "cyborg_deci_arith" })?;
        let slot = self.deci_slot()?;
        let status = basic_result(self.builder.build_call(f, &[i32t.const_int(code, false).into(), alo.into(), ahi.into(), blo.into(), bhi.into(), width.into(), slot.into()], "status").ok()?)?.into_int_value();
        let i8t = self.ctx.i8_type();
        let over = self.builder.build_int_compare(IntPredicate::EQ, status, i8t.const_int(1, false), "past_largest").ok()?;
        self.deci_stop(fi, over, &format!("`{}` went past what a `deci{}` holds", op.spelled(), bits), func)?;
        if code >= 3 {
            let zero = self.builder.build_int_compare(IntPredicate::EQ, status, i8t.const_int(2, false), "by_zero").ok()?;
            let why = match code {
                3 => "the divisor is zero, and nothing can be divided by zero",
                4 => "`mod` by zero",
                _ => "zero to a negative power is a division by zero",
            };
            self.deci_stop(fi, zero, why, func)?;
        }
        Some(self.deci_read(slot, bits)?.into())
    }

    /// Whether the `deci` of `bits` in `v` is outside a range type's
    /// bounds, which are written decimals (`deci64[0.0, 1.0]`).
    pub(super) fn deci_outside(&mut self, v: IntValue<'ctx>, bits: u16, lo: &str, hi: &str) -> Option<IntValue<'ctx>> {
        let (Ok(l), Ok(h)) = (crate::deci::literal(lo, bits), crate::deci::literal(hi, bits)) else { return None };
        let (xlo, xhi) = self.deci_halves(v)?;
        let i64t = self.ctx.i64_type();
        let i32t = self.ctx.i32_type();
        let f = self.rt_deci("cyborg_deci_compare")?;
        let width = i32t.const_int(bits as u64, false);
        let mut out = self.ctx.bool_type().const_zero();
        for (bound, below) in [(l, true), (h, false)] {
            let (blo, bhi) = (i64t.const_int(bound as u64, false), i64t.const_int((bound >> 64) as u64, false));
            let c = super::basic_result(self.builder.build_call(f, &[xlo.into(), xhi.into(), blo.into(), bhi.into(), width.into()], "order").ok()?)?.into_int_value();
            let pred = if below { IntPredicate::SLT } else { IntPredicate::SGT };
            let past = self.builder.build_int_compare(pred, c, i32t.const_zero(), "past").ok()?;
            out = self.builder.build_or(out, past, "outside").ok()?;
        }
        Some(out)
    }

    /// `-` on a `deci`: the sign bit, and nothing else (`-0` stays itself
    /// negated, as IEEE has it).
    pub(super) fn deci_neg(&mut self, v: IntValue<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let t = v.get_type();
        let bits = t.get_bit_width();
        let sign = if bits == 128 {
            t.const_int_arbitrary_precision(&[0, 1u64 << 63])
        } else {
            t.const_int(1u64 << (bits - 1), false)
        };
        Some(self.builder.build_xor(v, sign, "deci_neg").ok()?.into())
    }

    /// A conversion that cannot fail, into or out of a `deci`: from a whole
    /// number that always fits, from a bin (its exact value or, `useful`,
    /// its shortest decimal), to a wider `deci`, to a bin.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn deci_convert(&mut self, fi: usize, v: BasicValueEnum<'ctx>, from: &Ty, to: &Ty, useful: bool, from_signed: bool, func: FunctionValue<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let i32t = self.ctx.i32_type();
        match (from.base(), to.base()) {
            (Ty::Deci(_), Ty::Deci(b)) => {
                // Wider holds every value exactly: through the runtime so
                // the layout is rebuilt at the new width.
                let (lo, hi) = self.deci_halves(v.into_int_value())?;
                let from_bits = match from.base() {
                    Ty::Deci(a) => *a,
                    _ => return None,
                };
                if from_bits == *b {
                    return Some(v);
                }
                let f = self.rt_deci("cyborg_deci_widen")?;
                let slot = self.deci_slot()?;
                let _ = self.builder.build_call(f, &[lo.into(), hi.into(), i32t.const_int(from_bits as u64, false).into(), i32t.const_int(*b as u64, false).into(), slot.into()], "");
                Some(self.deci_read(slot, *b)?.into())
            }
            (Ty::Int(_), Ty::Deci(b)) => {
                let x = self.builder.build_int_cast_sign_flag(v.into_int_value(), self.ctx.i128_type(), from_signed, "whole").ok()?;
                let (lo, hi) = self.deci_halves(x)?;
                let f = self.rt_deci("cyborg_deci_from_int")?;
                let slot = self.deci_slot()?;
                let _ = self.builder.build_call(f, &[lo.into(), hi.into(), i32t.const_int(*b as u64, false).into(), slot.into()], "");
                Some(self.deci_read(slot, *b)?.into())
            }
            (Ty::Bin(_), Ty::Deci(b)) => {
                let x = v.into_float_value();
                let f64t = self.ctx.f64_type();
                let wide = if x.get_type() == f64t { x } else { self.builder.build_float_ext(x, f64t, "wide").ok()? };
                let src_bits = match self.ty(from)? {
                    inkwell::types::BasicTypeEnum::FloatType(t) if t == self.ctx.f32_type() => 32,
                    inkwell::types::BasicTypeEnum::FloatType(t) if t == self.ctx.f16_type() => 16,
                    _ => 64,
                };
                let f = self.rt_deci("cyborg_deci_from_bin")?;
                let slot = self.deci_slot()?;
                let mode = if useful { src_bits } else { 0 };
                let status = basic_result(self.builder.build_call(f, &[wide.into(), i32t.const_int(*b as u64, false).into(), i32t.const_int(mode, false).into(), slot.into()], "status").ok()?)?.into_int_value();
                let i8t = self.ctx.i8_type();
                let over = self.builder.build_int_compare(IntPredicate::EQ, status, i8t.const_int(1, false), "past_largest").ok()?;
                self.deci_stop(fi, over, &format!("this bin is past what a `deci{}` holds", b), func)?;
                let odd = self.builder.build_int_compare(IntPredicate::EQ, status, i8t.const_int(2, false), "not_finite").ok()?;
                self.deci_stop(fi, odd, "not yet: an infinity or NaN into a deci is not decided (asked)", func)?;
                Some(self.deci_read(slot, *b)?.into())
            }
            (Ty::Deci(b), Ty::Bin(_)) => {
                let (lo, hi) = self.deci_halves(v.into_int_value())?;
                let t = self.ty(to)?.into_float_type();
                let to_bits = if t == self.ctx.f32_type() { 32 } else if t == self.ctx.f16_type() { 16 } else { 64 };
                let f = self.rt_deci("cyborg_deci_to_bin")?;
                let x = basic_result(self.builder.build_call(f, &[lo.into(), hi.into(), i32t.const_int(*b as u64, false).into(), i32t.const_int(to_bits, false).into()], "bin").ok()?)?.into_float_value();
                if t == self.ctx.f64_type() {
                    Some(x.into())
                } else {
                    Some(self.builder.build_float_trunc(x, t, "at_width").ok()?.into())
                }
            }
            _ => None,
        }
    }

    /// A `deci` as text, into a new heap text: every digit it holds, or
    /// (`useful`) without the zeros after the point.
    pub(super) fn deci_text_raw(&mut self, lo: IntValue<'ctx>, hi: IntValue<'ctx>, bits: u16, useful: bool) -> Option<BasicValueEnum<'ctx>> {
        let shape = self.handle(2);
        let out = self.builder.build_alloca(shape, "text").ok()?;
        let i32t = self.ctx.i32_type();
        let f = self.rt_deci("cyborg_deci_to_text")?;
        let _ = self.builder.build_call(f, &[out.into(), lo.into(), hi.into(), i32t.const_int(bits as u64, false).into(), i32t.const_int(useful as u64, false).into()], "");
        self.builder.build_load(shape, out, "text").ok()
    }
}
