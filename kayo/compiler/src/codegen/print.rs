//! `cord::print` and `cord::exit`, the two pieces of cord Synapse builds
//! (owner, 2026-10-09: "`b`, not the entire fucking neuron"). A print
//! fills Axon's output buffer (`runtime/src/print.rs`), which needs no
//! heap: on an OS it goes to stdout or stderr; with no OS, to the `output`
//! the target names. `exit` is the OS's; with no OS, the target's `exit`,
//! or the CPU sleeps as when MAIN ends.
//!
//! Each value is written as the oracle prints it: a list `[1, 2]`, a
//! struct `Locker[name = field kit, number = 7]`, a case with data
//! `Status.claimed[Bo]` and one without `free` (useful) or `Status.free`
//! (stored), a vector `simd[1, 2]`; text inside them as it is, unquoted.

use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use inkwell::{AddressSpace, IntPredicate};

use super::Cg;
use crate::mir::{Const, Operand};
use crate::types::{BinW, IntW, SeqKind, Ty, Unit};

/// Axon's `Out`: where the bytes go, how many are held, and room for 512.
const CAP: u32 = 512;

impl<'ctx> Cg<'_, 'ctx> {
    fn out_type(&self) -> StructType<'ctx> {
        let i32t = self.ctx.i32_type();
        self.ctx.struct_type(&[self.ctx.i64_type().into(), i32t.into(), i32t.into(), self.ctx.i8_type().array_type(CAP).into()], false)
    }

    /// One of Axon's `cyborg_out_*`, declared once.
    fn out_fn(&mut self, name: &str) -> FunctionValue<'ctx> {
        if let Some(f) = self.module.get_function(name) {
            return f;
        }
        let p = self.ctx.ptr_type(AddressSpace::default());
        let (w, i32t, b, v) = (self.ctx.i64_type(), self.ctx.i32_type(), self.ctx.bool_type(), self.ctx.void_type());
        let t = match name {
            "cyborg_out_init" => v.fn_type(&[p.into(), i32t.into(), w.into()], false),
            "cyborg_out_flush" => v.fn_type(&[p.into()], false),
            "cyborg_out_bytes" => v.fn_type(&[p.into(), p.into(), w.into()], false),
            "cyborg_out_int" => v.fn_type(&[p.into(), w.into(), w.into(), b.into()], false),
            "cyborg_out_bin" => v.fn_type(&[p.into(), self.ctx.f64_type().into(), i32t.into(), b.into()], false),
            "cyborg_out_char" => v.fn_type(&[p.into(), i32t.into()], false),
            "cyborg_out_deci" => v.fn_type(&[p.into(), w.into(), w.into(), i32t.into(), b.into()], false),
            "cyborg_out_bin_percent" => v.fn_type(&[p.into(), self.ctx.f64_type().into(), i32t.into(), i32t.into()], false),
            "cyborg_out_deci_percent" => v.fn_type(&[p.into(), w.into(), w.into(), i32t.into(), i32t.into()], false),
            "cyborg_out_deci_padded" => v.fn_type(&[p.into(), w.into(), w.into(), i32t.into(), i32t.into()], false),
            other => unreachable!("no the runtime output function {}", other),
        };
        self.module.add_function(name, t, None)
    }

    /// Which print a name is: the stream (1 or 2), the useful form, and
    /// whether a newline ends it.
    fn print_shape(name: &str) -> (u64, bool, bool) {
        let fd = if name.ends_with(".stderr") || name.ends_with("standard_error") { 2 } else { 1 };
        (fd, name.contains("useful"), name.contains("auto_newline"))
    }

    /// One print: a buffer on the stack, each value into it, then out.
    pub(super) fn emit_print(&mut self, fi: usize, name: &str, args: &[Operand], slots: &[Option<PointerValue<'ctx>>]) -> Option<()> {
        let (fd, useful, newline) = Self::print_shape(name);
        let out = self.stack(self.out_type(), "out").ok()?;
        let (sink, to) = self.out_sink(fd)?;
        let init = self.out_fn("cyborg_out_init");
        let _ = self.builder.build_call(init, &[out.into(), self.ctx.i32_type().const_int(sink, false).into(), to.into()], "");
        for a in args {
            let ty = match a {
                // At the width `operand` builds it (64 bits, or 128 when 64
                // cannot hold it signed), read back as that, signed.
                Operand::Const(Const::Int(v)) if i64::try_from(*v).is_ok() => Ty::int(true, 64),
                Operand::Const(Const::Int(_)) => Ty::int(true, 128),
                // Past 128 bits: at the width `operand` builds it.
                Operand::Const(Const::Big(v)) => Ty::int(true, crate::bigint::Big::parse(v)?.bits() as u32 + 1),
                Operand::Const(Const::Bool(_)) => Ty::Bool,
                Operand::Const(Const::Bin(_)) => Ty::Bin(BinW::Fixed(64)),
                Operand::Const(Const::Deci { bits, .. }) => Ty::Deci(*bits),
                Operand::Const(Const::Str(_)) => Ty::utf8(),
                Operand::Const(Const::Char(_)) => Ty::Char(Unit::Point),
                Operand::Const(Const::SizeOf { .. }) => Ty::u64(),
                Operand::Const(Const::Duration(_)) => Ty::Duration,
                _ => self.operand_cy_ty(fi, a)?,
            };
            // A text known now goes in as its bytes.
            if let Operand::Const(Const::Str(s)) = a {
                self.out_text(out, s);
                continue;
            }
            let v = self.operand(fi, a, slots)?;
            let signed = match a {
                Operand::Const(_) => None,
                _ => self.operand_signed(fi, a),
            };
            let at = self.stack(v.get_type(), "printed").ok()?;
            let _ = self.builder.build_store(at, v);
            self.print_at(out, at, &ty, useful, signed)?;
        }
        if newline {
            self.out_text(out, "\n");
        }
        let flush = self.out_fn("cyborg_out_flush");
        let _ = self.builder.build_call(flush, &[out.into()], "");
        Some(())
    }

    /// Where a print's bytes go: a file descriptor on an OS; with no OS the
    /// target's output, one byte or a chunk a call.
    fn out_sink(&mut self, fd: u64) -> Option<(u64, IntValue<'ctx>)> {
        let w = self.ctx.i64_type();
        if !crate::target::bare() {
            return Some((0, w.const_int(fd, false)));
        }
        let Some((id, form)) = self.p.items.board_output else {
            self.cannot(format!("`std::print` with no OS writes to the output the target names, and this one names none: `{} {{` / `layout = …` / `output = my.serial.write` / `}}` under `my targets:`", crate::target::name()), None);
            return None;
        };
        let f = self.board_fn(id)?;
        let addr = self.builder.build_ptr_to_int(f.as_global_value().as_pointer_value(), w, "output").ok()?;
        Some((if form == crate::resolve::OutputForm::Byte { 1 } else { 2 }, addr))
    }

    /// The LLVM function of a board's `output` or `exit`: the instance the
    /// checker made of it.
    fn board_fn(&mut self, id: crate::resolve::FnId) -> Option<FunctionValue<'ctx>> {
        let i = self.p.functions.iter().position(|f| f.inst.and_then(|inst| self.p.instances.get(inst.0 as usize)).is_some_and(|inst| inst.key.fn_id == id));
        match i.and_then(|i| self.fns.get(&i).copied()) {
            Some(f) => Some(f),
            None => {
                self.cannot("the target's output or exit was never made into code", None);
                None
            }
        }
    }

    fn out_text(&mut self, out: PointerValue<'ctx>, s: &str) {
        let p = self.global_bytes(s);
        let n = self.ctx.i64_type().const_int(s.len() as u64, false);
        let f = self.out_fn("cyborg_out_bytes");
        let _ = self.builder.build_call(f, &[out.into(), p.into(), n.into()], "");
    }

    /// The value of type `ty` at `at`, in its stored or useful form.
    /// `signed` says how an integer place reads when its type does not.
    fn print_at(&mut self, out: PointerValue<'ctx>, at: PointerValue<'ctx>, ty: &Ty, useful: bool, signed: Option<bool>) -> Option<()> {
        let w = self.ctx.i64_type();
        // A percent shows its `%` in `print.useful` only (owner,
        // 2026-10-17): `print.stored` shows the number it holds, 0.07.
        // `min_places` pads `print.useful` (owner, 2026-10-17): `5.00`.
        let min = if useful { crate::check::expr::places_min(ty).unwrap_or(0) } else { 0 };
        let min_v = self.ctx.i32_type().const_int(min as u64, false);
        if useful && crate::check::expr::percent_inner(ty).is_none() && min > 0 {
            if let Ty::Deci(bits) = ty.base() {
                let lt = self.ty(ty)?.into_int_type();
                let v = self.builder.build_load(lt, at, "d").ok()?.into_int_value();
                let (lo, hi) = self.deci_halves(v)?;
                let f = self.out_fn("cyborg_out_deci_padded");
                let _ = self.builder.build_call(f, &[out.into(), lo.into(), hi.into(), self.ctx.i32_type().const_int(*bits as u64, false).into(), min_v.into()], "");
                return Some(());
            }
        }
        if useful && crate::check::expr::percent_inner(ty).is_some() {
            match ty.base() {
                Ty::Bin(_) => {
                    let lt = self.ty(ty)?.into_float_type();
                    let v = self.builder.build_load(lt, at, "x").ok()?.into_float_value();
                    let x = if lt == self.ctx.f64_type() { v } else { self.builder.build_float_ext(v, self.ctx.f64_type(), "wide").ok()? };
                    let bits = match ty.base() {
                        Ty::Bin(BinW::Fixed(b)) => *b as u64,
                        _ => 64,
                    };
                    let f = self.out_fn("cyborg_out_bin_percent");
                    let _ = self.builder.build_call(f, &[out.into(), x.into(), self.ctx.i32_type().const_int(bits, false).into(), min_v.into()], "");
                    return Some(());
                }
                Ty::Deci(bits) => {
                    let lt = self.ty(ty)?.into_int_type();
                    let v = self.builder.build_load(lt, at, "d").ok()?.into_int_value();
                    let (lo, hi) = self.deci_halves(v)?;
                    let f = self.out_fn("cyborg_out_deci_percent");
                    let _ = self.builder.build_call(f, &[out.into(), lo.into(), hi.into(), self.ctx.i32_type().const_int(*bits as u64, false).into(), min_v.into()], "");
                    return Some(());
                }
                _ => {}
            }
        }
        match ty.base() {
            Ty::Str(_) | Ty::Char(Unit::Grapheme) => {
                let h = self.handle(2);
                let v = self.builder.build_load(h, at, "text").ok()?;
                let (p, n) = self.text_parts_pub(v)?;
                let n = self.builder.build_and(n, w.const_int(!(1u64 << 63), false), "len").ok()?;
                let f = self.out_fn("cyborg_out_bytes");
                let _ = self.builder.build_call(f, &[out.into(), p.into(), n.into()], "");
            }
            Ty::Bool => {
                let v = self.builder.build_load(self.ctx.bool_type(), at, "b").ok()?.into_int_value();
                let func = self.builder.get_insert_block()?.get_parent()?;
                let (yes, no, join) = (self.ctx.append_basic_block(func, "true"), self.ctx.append_basic_block(func, "false"), self.ctx.append_basic_block(func, "printed"));
                let _ = self.builder.build_conditional_branch(v, yes, no);
                self.builder.position_at_end(yes);
                self.out_text(out, "true");
                let _ = self.builder.build_unconditional_branch(join);
                self.builder.position_at_end(no);
                self.out_text(out, "false");
                let _ = self.builder.build_unconditional_branch(join);
                self.builder.position_at_end(join);
            }
            Ty::Int(iw) => {
                let lt = self.ty(ty)?.into_int_type();
                let v = self.builder.build_load(lt, at, "n").ok()?.into_int_value();
                let signed = match iw {
                    IntW::Fixed { signed, .. } => *signed,
                    IntW::Free(_) => signed.unwrap_or(true),
                };
                if lt.get_bit_width() > 128 {
                    // Past 128 bits, its digits as `to.str.utf8` makes them.
                    if crate::target::bare() {
                        self.cannot("printing a whole number past 128 bits with no OS is not built yet", None);
                        return None;
                    }
                    let t = self.wide_int_text(v, signed)?;
                    let (p, n) = self.text_parts_pub(t)?;
                    let n = self.builder.build_and(n, w.const_int(!(1u64 << 63), false), "len").ok()?;
                    let f = self.out_fn("cyborg_out_bytes");
                    let _ = self.builder.build_call(f, &[out.into(), p.into(), n.into()], "");
                    self.free_text(t);
                    return Some(());
                }
                let i128t = self.ctx.i128_type();
                let x = self.builder.build_int_cast_sign_flag(v, i128t, signed, "wide").ok()?;
                let lo = self.builder.build_int_truncate(x, w, "lo").ok()?;
                let sh = self.builder.build_right_shift(x, i128t.const_int(64, false), false, "shifted").ok()?;
                let hi = self.builder.build_int_truncate(sh, w, "hi").ok()?;
                let f = self.out_fn("cyborg_out_int");
                let _ = self.builder.build_call(f, &[out.into(), lo.into(), hi.into(), self.ctx.bool_type().const_int(signed as u64, false).into()], "");
            }
            Ty::Char(_) => {
                let lt = self.ty(ty)?.into_int_type();
                let v = self.builder.build_load(lt, at, "c").ok()?.into_int_value();
                let code = self.builder.build_int_z_extend_or_bit_cast(v, self.ctx.i32_type(), "code").ok()?;
                let f = self.out_fn("cyborg_out_char");
                let _ = self.builder.build_call(f, &[out.into(), code.into()], "");
            }
            Ty::Bin(_) | Ty::Brain => {
                let lt = self.ty(ty)?.into_float_type();
                if lt == self.ctx.f128_type() {
                    self.cannot("printing a `f128` is not built yet", None);
                    return None;
                }
                let v = self.builder.build_load(lt, at, "x").ok()?.into_float_value();
                let x = if lt == self.ctx.f64_type() { v } else { self.builder.build_float_ext(v, self.ctx.f64_type(), "wide").ok()? };
                let bits = match ty.base() {
                    Ty::Bin(BinW::Fixed(b)) => *b as u64,
                    Ty::Brain => crate::types::BRAIN_BITS as u64,
                    _ => 64,
                };
                let f = self.out_fn("cyborg_out_bin");
                let _ = self.builder.build_call(f, &[out.into(), x.into(), self.ctx.i32_type().const_int(bits, false).into(), self.ctx.bool_type().const_int(useful as u64, false).into()], "");
            }
            Ty::Deci(bits) => {
                let lt = self.ty(ty)?.into_int_type();
                let v = self.builder.build_load(lt, at, "d").ok()?.into_int_value();
                let (lo, hi) = self.deci_halves(v)?;
                let f = self.out_fn("cyborg_out_deci");
                let _ = self.builder.build_call(f, &[out.into(), lo.into(), hi.into(), self.ctx.i32_type().const_int(*bits as u64, false).into(), self.ctx.bool_type().const_int(useful as u64, false).into()], "");
            }
            Ty::Struct { id, args } => {
                let info = self.p.items.structs[id.0 as usize].clone();
                let shape = self.ty(ty)?.into_struct_type();
                self.out_text(out, &format!("{}[", info.name));
                for (k, field) in info.fields.iter().enumerate() {
                    let ft = field.ty.subst(args);
                    self.out_text(out, &format!("{}{} = ", if k > 0 { ", " } else { "" }, field.name));
                    let fat = self.builder.build_struct_gep(shape, at, k as u32, "field").ok()?;
                    self.print_at(out, fat, &ft, useful, None)?;
                }
                self.out_text(out, "]");
            }
            Ty::Enum { id, args } => {
                let (id, args) = (*id, args.clone());
                self.print_case(out, at, id, &args, useful)?;
            }
            Ty::Seq { elem, kind } => {
                let elem = (**elem).clone();
                let shape = self.handle(if *kind == SeqKind::Array { 2 } else { 3 });
                let data = self.builder.build_struct_gep(shape, at, 0, "data").ok()?;
                let data = self.builder.build_load(self.ctx.ptr_type(AddressSpace::default()), data, "p").ok()?.into_pointer_value();
                let len = self.builder.build_struct_gep(shape, at, 1, "len").ok()?;
                let len = self.builder.build_load(w, len, "n").ok()?.into_int_value();
                let esize = self.size_of_ty(&elem)?;
                self.out_text(out, "[");
                self.each(len, |this, i| {
                    let first = this.builder.build_int_compare(IntPredicate::EQ, i, w.const_zero(), "first").ok()?;
                    let func = this.builder.get_insert_block()?.get_parent()?;
                    let (sep, item) = (this.ctx.append_basic_block(func, "comma"), this.ctx.append_basic_block(func, "item"));
                    let _ = this.builder.build_conditional_branch(first, item, sep);
                    this.builder.position_at_end(sep);
                    this.out_text(out, ", ");
                    let _ = this.builder.build_unconditional_branch(item);
                    this.builder.position_at_end(item);
                    let off = this.builder.build_int_mul(i, w.const_int(esize, false), "off").ok()?;
                    let eat = unsafe { this.builder.build_gep(this.ctx.i8_type(), data, &[off], "elem").ok()? };
                    this.print_at(out, eat, &elem, useful, None)
                })?;
                self.out_text(out, "]");
            }
            Ty::Simd { elem, lanes } => {
                let (elem, lanes) = ((**elem).clone(), *lanes);
                let vt = self.ty(ty)?.into_vector_type();
                let v = self.builder.build_load(vt, at, "v").ok()?.into_vector_value();
                let et: BasicTypeEnum<'ctx> = vt.get_element_type();
                let lane = self.stack(et, "lane").ok()?;
                self.out_text(out, "simd[");
                for k in 0..lanes {
                    if k > 0 {
                        self.out_text(out, ", ");
                    }
                    let x = self.builder.build_extract_element(v, self.ctx.i32_type().const_int(k as u64, false), "x").ok()?;
                    let _ = self.builder.build_store(lane, x);
                    self.print_at(out, lane, &elem, useful, None)?;
                }
                self.out_text(out, "]");
            }
            // In the largest unit it is a whole number of, singular for 1:
            // `90 secs`, `2 mins`, `1 milli`; nothing is `0 hours`
            // (the oracle's form).
            Ty::Duration => {
                let n = self.builder.build_load(w, at, "nanos").ok()?.into_int_value();
                let func = self.builder.get_insert_block()?.get_parent()?;
                let join = self.ctx.append_basic_block(func, "duration_printed");
                let units: [(u64, &str, &str); 6] = [(3_600_000_000_000, "hour", "hours"), (60_000_000_000, "min", "mins"), (1_000_000_000, "sec", "secs"), (1_000_000, "milli", "millis"), (1_000, "micro", "micros"), (1, "nano", "nanos")];
                let zero = self.builder.build_int_compare(IntPredicate::EQ, n, w.const_zero(), "none").ok()?;
                let (is_zero, some) = (self.ctx.append_basic_block(func, "zero"), self.ctx.append_basic_block(func, "some"));
                let _ = self.builder.build_conditional_branch(zero, is_zero, some);
                self.builder.position_at_end(is_zero);
                self.out_text(out, "0 hours");
                let _ = self.builder.build_unconditional_branch(join);
                self.builder.position_at_end(some);
                for (k, (unit, one, many)) in units.iter().enumerate() {
                    let u = w.const_int(*unit, false);
                    let whole = if k + 1 == units.len() {
                        self.ctx.bool_type().const_int(1, false)
                    } else {
                        let r = self.builder.build_int_unsigned_rem(n, u, "r").ok()?;
                        self.builder.build_int_compare(IntPredicate::EQ, r, w.const_zero(), "whole").ok()?
                    };
                    let (this, next) = (self.ctx.append_basic_block(func, "unit"), self.ctx.append_basic_block(func, "smaller"));
                    let _ = self.builder.build_conditional_branch(whole, this, next);
                    self.builder.position_at_end(this);
                    let q = self.builder.build_int_unsigned_div(n, u, "count").ok()?;
                    let f = self.out_fn("cyborg_out_int");
                    let _ = self.builder.build_call(f, &[out.into(), q.into(), w.const_zero().into(), self.ctx.bool_type().const_zero().into()], "");
                    let single = self.builder.build_int_compare(IntPredicate::EQ, q, w.const_int(1, false), "one").ok()?;
                    let (b1, bn) = (self.ctx.append_basic_block(func, "one"), self.ctx.append_basic_block(func, "many"));
                    let _ = self.builder.build_conditional_branch(single, b1, bn);
                    self.builder.position_at_end(b1);
                    self.out_text(out, &format!(" {}", one));
                    let _ = self.builder.build_unconditional_branch(join);
                    self.builder.position_at_end(bn);
                    self.out_text(out, &format!(" {}", many));
                    let _ = self.builder.build_unconditional_branch(join);
                    self.builder.position_at_end(next);
                }
                let _ = self.builder.build_unreachable();
                self.builder.position_at_end(join);
            }
            Ty::Map { .. } => {
                self.cannot("printing a map is not built yet", None);
                return None;
            }
            _ => {
                let spelled = crate::types::TySpelling { ty, structs: &self.p.items.structs, enums: &self.p.items.enums }.to_string();
                self.cannot(format!("printing {} is not built yet", crate::types::a_type(&spelled)), None);
                return None;
            }
        }
        Some(())
    }

    /// An enum's case: `free` (useful) or `Status.free` (stored), and one
    /// carrying data `Status.claimed[Bo]` in both forms, as the oracle
    /// prints them. A `layout.C` value no case has shows its number.
    fn print_case(&mut self, out: PointerValue<'ctx>, at: PointerValue<'ctx>, id: crate::types::EnumId, args: &[Ty], useful: bool) -> Option<()> {
        let info = self.p.items.enums[id.0 as usize].clone();
        let shape = self.enum_shape(id, args)?;
        let tt = self.tag_type_of(id);
        let tag_at = self.builder.build_struct_gep(shape, at, 0, "tag").ok()?;
        let tag = self.builder.build_load(tt, tag_at, "tag").ok()?.into_int_value();
        let func = self.builder.get_insert_block()?.get_parent()?;
        let join = self.ctx.append_basic_block(func, "case_printed");
        let other = self.ctx.append_basic_block(func, "no_case");
        let mut arms = Vec::new();
        for k in 0..info.cases.len() {
            arms.push((self.tag_of(id, k), self.ctx.append_basic_block(func, "case")));
        }
        let _ = self.builder.build_switch(tag, other, &arms);
        for (k, (_, b)) in arms.iter().enumerate() {
            self.builder.position_at_end(*b);
            let case = info.cases[k].clone();
            match &case.payload {
                None => {
                    let name = if useful { case.name.clone() } else { format!("{}.{}", info.name, case.name) };
                    self.out_text(out, &name);
                }
                Some(pt) => {
                    let pt = pt.subst(args);
                    self.out_text(out, &format!("{}.{}[", info.name, case.name));
                    let pat = self.builder.build_struct_gep(shape, at, 1, "data").ok()?;
                    self.print_at(out, pat, &pt, useful, None)?;
                    self.out_text(out, "]");
                }
            }
            let _ = self.builder.build_unconditional_branch(join);
        }
        self.builder.position_at_end(other);
        let i128t = self.ctx.i128_type();
        let w = self.ctx.i64_type();
        let x = self.builder.build_int_s_extend_or_bit_cast(tag, i128t, "wide").ok()?;
        let lo = self.builder.build_int_truncate(x, w, "lo").ok()?;
        let sh = self.builder.build_right_shift(x, i128t.const_int(64, false), true, "shifted").ok()?;
        let hi = self.builder.build_int_truncate(sh, w, "hi").ok()?;
        let f = self.out_fn("cyborg_out_int");
        let _ = self.builder.build_call(f, &[out.into(), lo.into(), hi.into(), self.ctx.bool_type().const_int(1, false).into()], "");
        let _ = self.builder.build_unconditional_branch(join);
        self.builder.position_at_end(join);
        Some(())
    }

    /// `body` once for each `i` from 0 below `n`.
    fn each(&mut self, n: IntValue<'ctx>, mut body: impl FnMut(&mut Self, IntValue<'ctx>) -> Option<()>) -> Option<()> {
        let w = self.ctx.i64_type();
        let i_at = self.stack(w, "i").ok()?;
        let _ = self.builder.build_store(i_at, w.const_zero());
        let func = self.builder.get_insert_block()?.get_parent()?;
        let (head, step, done) = (self.ctx.append_basic_block(func, "each"), self.ctx.append_basic_block(func, "each_body"), self.ctx.append_basic_block(func, "each_done"));
        let _ = self.builder.build_unconditional_branch(head);
        self.builder.position_at_end(head);
        let i = self.builder.build_load(w, i_at, "i").ok()?.into_int_value();
        let more = self.builder.build_int_compare(IntPredicate::ULT, i, n, "more").ok()?;
        let _ = self.builder.build_conditional_branch(more, step, done);
        self.builder.position_at_end(step);
        body(self, i)?;
        let next = self.builder.build_int_add(i, w.const_int(1, false), "next").ok()?;
        let _ = self.builder.build_store(i_at, next);
        let _ = self.builder.build_unconditional_branch(head);
        self.builder.position_at_end(done);
        Some(())
    }

    /// `cord::exit[code]`: the OS's `exit`; with no OS, the board's, or the
    /// CPU sleeps as when MAIN ends (owner, 2026-10-09: "c").
    pub(super) fn emit_exit(&mut self, fi: usize, args: &[Operand], slots: &[Option<PointerValue<'ctx>>]) -> Option<()> {
        let code = self.operand(fi, args.first()?, slots)?.into_int_value();
        let i32t = self.ctx.i32_type();
        if crate::target::bare() {
            if let Some(id) = self.p.items.board_exit {
                let f = self.board_fn(id)?;
                let code = self.builder.build_int_truncate_or_bit_cast(code, self.ctx.i8_type(), "code").ok()?;
                let _ = self.builder.build_call(f, &[code.into()], "");
            }
            // With none named, or if the board's comes back: sleep forever.
            let func = self.builder.get_insert_block()?.get_parent()?;
            let sleep = self.ctx.append_basic_block(func, "sleep");
            let _ = self.builder.build_unconditional_branch(sleep);
            self.builder.position_at_end(sleep);
            let wfe = self.ctx.void_type().fn_type(&[], false);
            let asm = self.ctx.create_inline_asm(wfe, "wfe".into(), "".into(), true, false, None, false);
            let _ = self.builder.build_indirect_call(wfe, asm, &[], "");
            let _ = self.builder.build_unconditional_branch(sleep);
        } else {
            let code = self.builder.build_int_z_extend_or_bit_cast(code, i32t, "code").ok()?;
            let exit = self.libc("exit", self.ctx.void_type().fn_type(&[i32t.into()], false));
            let _ = self.builder.build_call(exit, &[code.into()], "");
            let _ = self.builder.build_unreachable();
        }
        // It never comes back: what the block goes on to do is not reached.
        let func = self.builder.get_insert_block()?.get_parent()?;
        let after = self.ctx.append_basic_block(func, "after_exit");
        self.builder.position_at_end(after);
        Some(())
    }

    /// A C library function the OS has, declared once.
    fn libc(&mut self, name: &str, t: inkwell::types::FunctionType<'ctx>) -> FunctionValue<'ctx> {
        self.module.get_function(name).unwrap_or_else(|| self.module.add_function(name, t, None))
    }
}
