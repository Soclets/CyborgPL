//! C's calling convention for what crosses by value (`ffi.md`). LLVM passes
//! a struct the way its fields would go one by one, which is not how C
//! passes it: C packs `div_t`'s two `int`s into one register, and a
//! function that read two got the second from whatever was left in the
//! next one. So a `layout.c` struct and a C enum cross the way clang lowers
//! them for this machine, and Cyborg sees its own shape on either side.
//!
//! AAPCS64 (Apple's arm64 follows it for these): a struct of one to four of
//! the same float goes in float registers; up to 16 bytes goes in one or two
//! integer registers; anything larger is passed by address and returned
//! through the address the caller gives (`sret`). A C enum is C's `int`.

use super::*;

/// How one value crosses to C.
#[derive(Clone, Copy)]
pub(super) enum CPass<'ctx> {
    /// As Cyborg lays it out: a number, a pointer.
    Direct,
    /// Reinterpreted as this type, the bytes unchanged.
    Coerce(BasicTypeEnum<'ctx>),
    /// By address: a copy the caller makes, or, for a return, room the
    /// caller gives.
    Indirect(BasicTypeEnum<'ctx>),
}

impl<'a, 'ctx> Cg<'a, 'ctx> {
    /// How `t` crosses, as a parameter or as a return. `None`: this machine's
    /// convention for it is not written here yet.
    pub(super) fn c_pass(&mut self, t: &Ty, ret: bool) -> Option<CPass<'ctx>> {
        match t.base() {
            Ty::Enum { id, .. } if self.p.items.enums[id.0 as usize].numbers.is_some() => Some(CPass::Coerce(self.ctx.i32_type().into())),
            Ty::Struct { id, .. } if self.p.items.structs[id.0 as usize].layout_c => {
                if !cfg!(target_arch = "aarch64") {
                    return None;
                }
                let lt = self.ty(t)?;
                if let Some((elem, n)) = self.hfa(lt) {
                    return Some(CPass::Coerce(elem.array_type(n).into()));
                }
                let size = self.target.get_abi_size(&lt);
                if size > 16 {
                    return Some(CPass::Indirect(lt));
                }
                // A return of up to 8 bytes is exactly its size; anything
                // else rounds to whole registers.
                if ret && size <= 8 {
                    return Some(CPass::Coerce(self.int_bits((size.max(1) * 8) as u32)?.into()));
                }
                let reg: u64 = if self.target.get_abi_alignment(&lt) >= 16 { 16 } else { 8 };
                let regs = size.div_ceil(reg).max(1);
                let it = self.int_bits((reg * 8) as u32)?;
                Some(CPass::Coerce(if regs == 1 { it.into() } else { it.array_type(regs as u32).into() }))
            }
            _ => Some(CPass::Direct),
        }
    }

    fn int_bits(&self, bits: u32) -> Option<inkwell::types::IntType<'ctx>> {
        self.ctx.custom_width_int_type(std::num::NonZeroU32::new(bits)?).ok()
    }

    /// A homogeneous float aggregate: one to four of the same float, however
    /// the structs and arrays around them nest.
    fn hfa(&self, t: BasicTypeEnum<'ctx>) -> Option<(inkwell::types::FloatType<'ctx>, u32)> {
        fn leaves<'ctx>(t: BasicTypeEnum<'ctx>, out: &mut Vec<BasicTypeEnum<'ctx>>) {
            match t {
                BasicTypeEnum::StructType(s) => s.get_field_types().into_iter().for_each(|f| leaves(f, out)),
                BasicTypeEnum::ArrayType(a) => {
                    for _ in 0..a.len() {
                        leaves(a.get_element_type(), out);
                    }
                }
                other => out.push(other),
            }
        }
        let mut ls = Vec::new();
        leaves(t, &mut ls);
        let BasicTypeEnum::FloatType(first) = *ls.first()? else { return None };
        (ls.len() <= 4 && ls.iter().all(|l| *l == BasicTypeEnum::FloatType(first))).then_some((first, ls.len() as u32))
    }

    /// The same bytes read as another type: through memory, with room for
    /// the larger of the two.
    pub(super) fn reinterpret(&mut self, v: BasicValueEnum<'ctx>, to: BasicTypeEnum<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let from = v.get_type();
        let room = if self.target.get_abi_size(&from) >= self.target.get_abi_size(&to) { from } else { to };
        let tmp = self.here_alloca(room, "c_abi")?;
        let _ = self.builder.build_store(tmp, v);
        self.builder.build_load(to, tmp, "c_abi").ok()
    }

    /// Room in the current function's entry, so a loop does not grow the
    /// stack.
    pub(super) fn here_alloca(&mut self, t: BasicTypeEnum<'ctx>, name: &str) -> Option<PointerValue<'ctx>> {
        let func = self.builder.get_insert_block()?.get_parent()?;
        self.entry_alloca(func, t, name)
    }

    /// Cyborg's value at the door, as C takes it.
    pub(super) fn to_c(&mut self, v: BasicValueEnum<'ctx>, how: CPass<'ctx>) -> Option<BasicMetadataValueEnum<'ctx>> {
        match how {
            CPass::Direct => Some(v.into()),
            CPass::Coerce(t) => self.reinterpret(v, t).map(|x| x.into()),
            CPass::Indirect(_) => {
                let tmp = self.here_alloca(v.get_type(), "c_copy")?;
                let _ = self.builder.build_store(tmp, v);
                Some(tmp.into())
            }
        }
    }
}

