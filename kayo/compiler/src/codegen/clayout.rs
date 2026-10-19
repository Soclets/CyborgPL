//! C's unions and bit-fields (owner, 2026-10-19; `design/ffi.md` § Unions
//! and bit-fields), laid out as clang lays them out on macOS. A
//! `layout.C union`, or a `layout.C struct` with a bit-field, is held as
//! words of its alignment, and each field sits at the byte, and for a
//! bit-field the bit, that C gives it. Every other struct keeps LLVM's own
//! layout, which is C's already.

use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

use super::Cg;
use crate::mir::*;
use crate::types::{IntW, Ty};

/// Where one field sits.
#[derive(Clone, Copy, Debug)]
pub(super) struct CField {
    /// The byte its storage starts at: a bit-field's whole unit, of its
    /// declared type's size.
    pub byte: u64,
    /// A bit-field's first bit in that unit, and its width.
    pub bit: u32,
    pub width: Option<u32>,
    /// The unit's bytes: the declared type's size.
    pub unit: u64,
}

pub(super) struct CLayout {
    pub size: u64,
    pub align: u64,
    pub fields: Vec<CField>,
}

impl<'a, 'ctx> Cg<'a, 'ctx> {
    /// The C layout of a union or of a struct with a bit-field; `None` for
    /// every other type, which LLVM lays out as C does.
    pub(super) fn c_layout(&mut self, t: &Ty) -> Option<CLayout> {
        let Ty::Struct { id, args } = t.base() else { return None };
        let info = self.p.items.structs[id.0 as usize].clone();
        if !info.layout_c || !(info.union || info.fields.iter().any(|f| f.bits.is_some())) {
            return None;
        }
        let mut fields = Vec::new();
        let (mut bits_at, mut size, mut align) = (0u64, 0u64, 1u64);
        for f in &info.fields {
            let ft = f.ty.subst(args);
            let lt = self.ty(&ft)?;
            let s = self.target.get_abi_size(&lt);
            let a = self.target.get_abi_alignment(&lt) as u64;
            align = align.max(a);
            if info.union {
                fields.push(CField { byte: 0, bit: 0, width: f.bits, unit: s });
                size = size.max(s);
                continue;
            }
            match f.bits {
                // A bit-field takes the next bits, unless they would cross
                // a unit of its declared type: then it starts at the next
                // boundary of that type's alignment.
                Some(w) => {
                    let unit_bits = s * 8;
                    if bits_at % unit_bits + w as u64 > unit_bits {
                        bits_at = bits_at.div_ceil(a * 8) * a * 8;
                    }
                    let byte = (bits_at / unit_bits) * s;
                    fields.push(CField { byte, bit: (bits_at - byte * 8) as u32, width: Some(w), unit: s });
                    bits_at += w as u64;
                }
                None => {
                    bits_at = bits_at.div_ceil(a * 8) * a * 8;
                    fields.push(CField { byte: bits_at / 8, bit: 0, width: None, unit: s });
                    bits_at += s * 8;
                }
            }
        }
        if !info.union {
            size = bits_at.div_ceil(8);
        }
        let size = size.div_ceil(align).max(1) * align;
        Some(CLayout { size, align, fields })
    }

    /// The LLVM shape of such a type: words of its alignment, as many as
    /// its size holds.
    pub(super) fn c_layout_shape(&mut self, l: &CLayout) -> BasicTypeEnum<'ctx> {
        let word = self.ctx.custom_width_int_type(std::num::NonZeroU32::new((l.align * 8) as u32).unwrap()).unwrap();
        self.ctx.struct_type(&[word.array_type((l.size / l.align) as u32).into()], false).into()
    }

    /// A place's last step into a union's field or a bit-field: the place
    /// around it, which field, and where it sits.
    pub(super) fn c_field(&mut self, fi: usize, p: &Place) -> Option<(Place, usize, CField, Ty)> {
        let Some(Proj::Field(i)) = p.proj.last() else { return None };
        let outer = Place { local: p.local, proj: p.proj[..p.proj.len() - 1].to_vec() };
        let ot = crate::mir::build::projected_ty(&self.p.functions[fi].locals[p.local.0 as usize].ty, &outer.proj, &self.p.items);
        let l = self.c_layout(&ot)?;
        let Ty::Struct { id, args } = ot.base() else { return None };
        let ft = self.p.items.structs[id.0 as usize].fields[*i].ty.subst(args);
        Some((outer, *i, l.fields[*i], ft))
    }

    /// Where a field's storage starts, from the start of its struct.
    pub(super) fn c_field_at(&mut self, base: PointerValue<'ctx>, f: CField) -> Option<PointerValue<'ctx>> {
        let w = self.ctx.i64_type();
        unsafe { self.builder.build_gep(self.ctx.i8_type(), base, &[w.const_int(f.byte, false)], "c_field").ok() }
    }

    /// The unit a bit-field sits in, as an integer of its size.
    fn c_unit(&self, f: CField) -> inkwell::types::IntType<'ctx> {
        self.ctx.custom_width_int_type(std::num::NonZeroU32::new((f.unit * 8) as u32).unwrap()).unwrap()
    }

    /// A bit-field's value: its bits, shifted down and widened as its type
    /// is signed or not.
    pub(super) fn c_bits_read(&mut self, at: PointerValue<'ctx>, f: CField, ft: &Ty) -> Option<BasicValueEnum<'ctx>> {
        let w = f.width?;
        let unit = self.c_unit(f);
        let word = self.builder.build_load(unit, at, "unit").ok()?.into_int_value();
        let top = unit.get_bit_width() - f.bit - w;
        let up = self.builder.build_left_shift(word, unit.const_int(top as u64, false), "up").ok()?;
        let signed = matches!(ft.base(), Ty::Int(IntW::Fixed { signed: true, .. }));
        let down = self.builder.build_right_shift(up, unit.const_int((unit.get_bit_width() - w) as u64, false), signed, "down").ok()?;
        let BasicTypeEnum::IntType(want) = self.ty(ft)? else { return None };
        Some(self.builder.build_int_cast_sign_flag(down, want, signed, "field").ok()?.into())
    }

    /// Write a bit-field: its bits replaced, the unit's others kept.
    pub(super) fn c_bits_write(&mut self, at: PointerValue<'ctx>, f: CField, v: IntValue<'ctx>) -> Option<()> {
        let w = f.width?;
        let unit = self.c_unit(f);
        let ones = if w as u32 >= unit.get_bit_width() { u64::MAX } else { (1u64 << w) - 1 };
        let mask = unit.const_int(ones << f.bit, false);
        let v = self.builder.build_int_cast_sign_flag(v, unit, false, "bits").ok()?;
        let v = self.builder.build_and(v, unit.const_int(ones, false), "bits").ok()?;
        let v = self.builder.build_left_shift(v, unit.const_int(f.bit as u64, false), "bits").ok()?;
        let word = self.builder.build_load(unit, at, "unit").ok()?.into_int_value();
        let kept = self.builder.build_and(word, self.builder.build_not(mask, "keep").ok()?, "kept").ok()?;
        let next = self.builder.build_or(kept, v, "unit").ok()?;
        self.builder.build_store(at, next).ok()?;
        Some(())
    }

    /// Building one: zeroed, then each field given written where C puts it
    /// (a union's one).
    pub(super) fn c_make(&mut self, fi: usize, t: &Ty, l: &CLayout, fields: &[Operand], slots: &[Option<PointerValue<'ctx>>]) -> Option<BasicValueEnum<'ctx>> {
        let shape = self.c_layout_shape(l);
        let room = self.stack(shape, "c_value").ok()?;
        self.builder.build_store(room, shape.const_zero()).ok()?;
        let Ty::Struct { id, args } = t.base() else { return None };
        let info = self.p.items.structs[id.0 as usize].clone();
        for (i, o) in fields.iter().enumerate() {
            if matches!(o, Operand::Const(Const::Nothing)) {
                continue;
            }
            let v = self.operand(fi, o, slots)?;
            let f = l.fields[i];
            let ft = info.fields[i].ty.subst(args);
            let at = self.c_field_at(room, f)?;
            if f.width.is_some() {
                self.c_bits_write(at, f, v.into_int_value())?;
            } else {
                let lt = self.ty(&ft)?;
                let v = self.fit(v, lt);
                self.builder.build_store(at, v).ok()?;
            }
        }
        self.builder.build_load(shape, room, "c_value").ok()
    }
}
