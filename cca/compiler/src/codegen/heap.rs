//! The heap: building a list, reaching into one, growing one, and giving
//! it back.
//!
//! The shapes are the owner's, decided 2026-09-22 (`lists.md`,
//! `types.md`, `maps.md`): a `list` is a pointer, a length and a capacity;
//! an `array` and a `str` are a pointer and a length, because neither
//! grows. Nothing here invents a layout — it fills in the one that was
//! decided.
//!
//! Allocation goes through the runtime (`cyborg_alloc` and friends), which
//! stands on the C allocator: `ffi.md` says the language has no
//! raw-allocation primitive, so the runtime owns the only one there is.
//!
//! Dropping and copying are *generated functions*, one per type, rather
//! than code written out at each use. A list of lists has to recurse, and
//! a loop that frees each element cannot be spelled inline without
//! repeating itself at every drop.

use super::*;
use crate::types::EnumId;

impl<'a, 'ctx> Cg<'a, 'ctx> {
    // ------------------------------------------------------------ runtime

    /// Declare-or-get one of the runtime's functions.
    pub(super) fn rt(&mut self, name: &str) -> FunctionValue<'ctx> {
        if let Some(f) = self.module.get_function(name) {
            return f;
        }
        let p = self.ctx.ptr_type(AddressSpace::default());
        let w = self.ctx.i64_type();
        let t = match name {
            "cyborg_alloc" => p.fn_type(&[w.into()], false),
            "cyborg_realloc" => p.fn_type(&[p.into(), w.into()], false),
            "cyborg_free" => self.ctx.void_type().fn_type(&[p.into()], false),
            "cyborg_copy" | "cyborg_move" => self.ctx.void_type().fn_type(&[p.into(), p.into(), w.into()], false),
            "cyborg_str_concat" => self.ctx.void_type().fn_type(&[p.into(), p.into(), w.into(), p.into(), w.into()], false),
            "cyborg_str_copy" => self.ctx.void_type().fn_type(&[p.into(), p.into(), w.into()], false),
            "cyborg_str_free" => self.ctx.void_type().fn_type(&[p.into(), w.into()], false),
            other => unreachable!("no runtime function named {}", other),
        };
        self.module.add_function(name, t, None)
    }

    pub(super) fn alloc(&mut self, bytes: IntValue<'ctx>) -> PointerValue<'ctx> {
        let f = self.rt("cyborg_alloc");
        let call = self.builder.build_call(f, &[bytes.into()], "mem").unwrap();
        basic_result(call).unwrap().into_pointer_value()
    }

    pub(super) fn free_mem(&mut self, ptr: PointerValue<'ctx>) {
        let f = self.rt("cyborg_free");
        let _ = self.builder.build_call(f, &[ptr.into()], "");
    }

    // ------------------------------------------------------------ shapes

    /// How many bytes one value of this type takes.
    pub(super) fn size_of_ty(&mut self, t: &Ty) -> Option<u64> {
        let l = self.ty(t)?;
        Some(self.target.get_store_size(&l))
    }

    /// The element type of a list or an array.
    pub(super) fn elem_of(t: &Ty) -> Option<Ty> {
        match t.base() {
            Ty::Seq { elem, .. } => Some((**elem).clone()),
            _ => None,
        }
    }

    /// Does giving this value up give memory back? A plain number owns
    /// nothing; a list owns its buffer and everything in it.
    pub(super) fn owns_memory(&self, t: &Ty) -> bool {
        match t.base() {
            Ty::Seq { .. } | Ty::Map { .. } => true,
            // A text may point at the binary's own bytes. Which memory it
            // is in is Precog's to know (owner, 2026-09-22); until that
            // fact reaches here, the handle carries it — the high bit of
            // the length — and the free tests it. That is the fallback
            // the same decision names.
            Ty::Str(_) => true,
            Ty::Struct { id, args } => {
                let info = &self.p.items.structs[id.0 as usize];
                info.fields.iter().any(|f| self.owns_memory(&f.ty.subst(args)))
            }
            _ => false,
        }
    }

    /// A name for the generated helpers, one per distinct type.
    fn type_key(&self, t: &Ty) -> String {
        let s = crate::types::TySpelling { ty: t, structs: &self.p.items.structs, enums: &self.p.items.enums }.to_string();
        sanitise(&s)
    }

    // ------------------------------------------------------- drop and copy

    /// `cy_drop_T(ptr)` — give back everything the value at `ptr` owns.
    /// Generated once per type, because a list of lists has to recurse.
    pub(super) fn drop_fn(&mut self, t: &Ty) -> Option<FunctionValue<'ctx>> {
        if !self.owns_memory(t) {
            return None;
        }
        let name = format!("cy_drop_{}", self.type_key(t));
        if let Some(f) = self.helpers.get(&name) {
            return Some(*f);
        }
        let p = self.ctx.ptr_type(AddressSpace::default());
        let func = self.module.add_function(&name, self.ctx.void_type().fn_type(&[p.into()], false), Some(Linkage::Internal));
        // Registered before the body is built: a list of lists reaches its
        // own drop while that drop is still being written.
        self.helpers.insert(name, func);
        let here = self.builder.get_insert_block();
        let entry = self.ctx.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);
        let arg = func.get_nth_param(0).unwrap().into_pointer_value();
        self.emit_drop_body(t, arg, func);
        let _ = self.builder.build_return(None);
        if let Some(b) = here {
            self.builder.position_at_end(b);
        }
        Some(func)
    }

    fn emit_drop_body(&mut self, t: &Ty, at: PointerValue<'ctx>, func: FunctionValue<'ctx>) {
        match t.base() {
            Ty::Seq { .. } => {
                let shape = match self.ty(t) {
                    Some(BasicTypeEnum::StructType(s)) => s,
                    _ => return,
                };
                let elem = match Self::elem_of(t) {
                    Some(e) => e,
                    None => return,
                };
                let data = self.load_field_ptr(shape, at, 0);
                // Each element first, then the buffer they sat in.
                if let Some(inner) = self.drop_fn(&elem) {
                    let len = self.load_field_int(shape, at, 1);
                    let esize = self.size_of_ty(&elem).unwrap_or(0);
                    self.each_element(data, len, esize, func, |cg, slot| {
                        let _ = cg.builder.build_call(inner, &[slot.into()], "");
                    });
                }
                self.free_mem(data);
            }
            Ty::Str(_) => {
                let shape = match self.ty(t) {
                    Some(BasicTypeEnum::StructType(s)) => s,
                    _ => return,
                };
                let ptr = self.load_field_ptr(shape, at, 0);
                let len = self.load_field_int(shape, at, 1);
                let f = self.rt("cyborg_str_free");
                let _ = self.builder.build_call(f, &[ptr.into(), len.into()], "");
            }
            Ty::Struct { id, args } => {
                let info = self.p.items.structs[id.0 as usize].clone();
                let shape = match self.ty(t) {
                    Some(BasicTypeEnum::StructType(s)) => s,
                    _ => return,
                };
                for (i, f) in info.fields.iter().enumerate() {
                    let ft = f.ty.subst(&args);
                    if let Some(inner) = self.drop_fn(&ft) {
                        if let Ok(fp) = self.builder.build_struct_gep(shape, at, i as u32, "f") {
                            let _ = self.builder.build_call(inner, &[fp.into()], "");
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// `cy_copy_T(dst, src)` — `copy` is "a second, independent value; an
    /// allocation" (`ownership-syntax.md`), so a list's buffer is copied
    /// and so is everything in it.
    pub(super) fn copy_fn(&mut self, t: &Ty) -> Option<FunctionValue<'ctx>> {
        if !self.owns_memory(t) {
            return None;
        }
        let name = format!("cy_copy_{}", self.type_key(t));
        if let Some(f) = self.helpers.get(&name) {
            return Some(*f);
        }
        let p = self.ctx.ptr_type(AddressSpace::default());
        let func = self.module.add_function(&name, self.ctx.void_type().fn_type(&[p.into(), p.into()], false), Some(Linkage::Internal));
        self.helpers.insert(name, func);
        let here = self.builder.get_insert_block();
        let entry = self.ctx.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);
        let dst = func.get_nth_param(0).unwrap().into_pointer_value();
        let src = func.get_nth_param(1).unwrap().into_pointer_value();
        self.emit_copy_body(t, dst, src, func);
        let _ = self.builder.build_return(None);
        if let Some(b) = here {
            self.builder.position_at_end(b);
        }
        Some(func)
    }

    fn emit_copy_body(&mut self, t: &Ty, dst: PointerValue<'ctx>, src: PointerValue<'ctx>, func: FunctionValue<'ctx>) {
        match t.base() {
            Ty::Seq { .. } => {
                let (shape, elem) = match (self.ty(t), Self::elem_of(t)) {
                    (Some(BasicTypeEnum::StructType(s)), Some(e)) => (s, e),
                    _ => return,
                };
                let esize = self.size_of_ty(&elem).unwrap_or(0);
                let old = self.load_field_ptr(shape, src, 0);
                let len = self.load_field_int(shape, src, 1);
                let w = self.ctx.i64_type();
                let bytes = self.builder.build_int_mul(len, w.const_int(esize, false), "bytes").unwrap();
                let fresh = self.alloc(bytes);
                let cp = self.rt("cyborg_copy");
                let _ = self.builder.build_call(cp, &[fresh.into(), old.into(), bytes.into()], "");
                // A shallow copy is right for plain elements and wrong for
                // owning ones: two lists would share one buffer, and the
                // second drop would free it twice.
                if self.copy_fn(&elem).is_some() {
                    let inner = self.copy_fn(&elem).unwrap();
                    self.each_element(fresh, len, esize, func, move |cg, slot| {
                        // src and dst are the same slot here: the shallow
                        // copy is already in place, so the helper copies
                        // what it owns over its own shallow twin.
                        let _ = cg.builder.build_call(inner, &[slot.into(), slot.into()], "");
                    });
                }
                self.store_field(shape, dst, 0, fresh.into());
                self.store_field(shape, dst, 1, len.into());
                if shape.count_fields() > 2 {
                    self.store_field(shape, dst, 2, len.into());
                }
            }
            Ty::Str(_) => {
                let shape = match self.ty(t) {
                    Some(BasicTypeEnum::StructType(s)) => s,
                    _ => return,
                };
                let ptr = self.load_field_ptr(shape, src, 0);
                let len = self.load_field_int(shape, src, 1);
                let f = self.rt("cyborg_str_copy");
                let _ = self.builder.build_call(f, &[dst.into(), ptr.into(), len.into()], "");
            }
            Ty::Struct { id, args } => {
                let info = self.p.items.structs[id.0 as usize].clone();
                let shape = match self.ty(t) {
                    Some(BasicTypeEnum::StructType(s)) => s,
                    _ => return,
                };
                for (i, f) in info.fields.iter().enumerate() {
                    let ft = f.ty.subst(&args);
                    let (Ok(dp), Ok(sp)) = (self.builder.build_struct_gep(shape, dst, i as u32, "d"), self.builder.build_struct_gep(shape, src, i as u32, "s")) else { continue };
                    match self.copy_fn(&ft) {
                        Some(inner) => {
                            let _ = self.builder.build_call(inner, &[dp.into(), sp.into()], "");
                        }
                        None => {
                            if let Some(l) = self.ty(&ft) {
                                if let Ok(v) = self.builder.build_load(l, sp, "v") {
                                    let _ = self.builder.build_store(dp, v);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // --------------------------------------------------------- operations

    /// `[a, b, c]` — one allocation, the elements written into it, and a
    /// handle built round it. An empty literal owns nothing and holds a
    /// null pointer, so dropping it is free.
    pub(super) fn make_list(&mut self, fi: usize, ops: &[Operand], ty: &Ty, slots: &[Option<PointerValue<'ctx>>]) -> Option<BasicValueEnum<'ctx>> {
        let elem = Self::elem_of(ty)?;
        let shape = match self.ty(ty)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        let esize = self.size_of_ty(&elem)?;
        let lty = self.ty(&elem)?;
        let w = self.ctx.i64_type();
        let n = ops.len() as u64;
        let data = if n == 0 {
            self.ctx.ptr_type(AddressSpace::default()).const_null()
        } else {
            let bytes = w.const_int(n * esize, false);
            self.alloc(bytes)
        };
        for (i, o) in ops.iter().enumerate() {
            let v = self.operand(fi, o, slots)?;
            let v = self.fit(v, lty);
            let off = w.const_int(i as u64 * esize, false);
            let at = unsafe { self.builder.build_gep(self.ctx.i8_type(), data, &[off], "at").ok()? };
            let _ = self.builder.build_store(at, v);
        }
        let mut agg = shape.get_undef();
        agg = self.builder.build_insert_value(agg, data, 0, "p").ok()?.into_struct_value();
        agg = self.builder.build_insert_value(agg, w.const_int(n, false), 1, "n").ok()?.into_struct_value();
        // A list carries a capacity; an array never grows, so it has none.
        if shape.count_fields() > 2 {
            agg = self.builder.build_insert_value(agg, w.const_int(n, false), 2, "c").ok()?.into_struct_value();
        }
        Some(agg.into())
    }

    /// `'xs'[pos i]` — 1-based (`lists.md`), so the machine's offset is
    /// one less. The position was bounds-checked at its own site.
    pub(super) fn index_slot(&mut self, at: PointerValue<'ctx>, ty: &Ty, idx: IntValue<'ctx>) -> Option<(PointerValue<'ctx>, Ty)> {
        let elem = Self::elem_of(ty)?;
        let shape = match self.ty(ty)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        let esize = self.size_of_ty(&elem)?;
        let w = self.ctx.i64_type();
        let data = self.load_field_ptr(shape, at, 0);
        let idx = self.builder.build_int_z_extend_or_bit_cast(idx, w, "i").ok()?;
        let zero_based = self.builder.build_int_sub(idx, w.const_int(1, false), "zb").ok()?;
        let off = self.builder.build_int_mul(zero_based, w.const_int(esize, false), "off").ok()?;
        let slot = unsafe { self.builder.build_gep(self.ctx.i8_type(), data, &[off], "elem").ok()? };
        Some((slot, elem))
    }

    /// `list.append[lend.mut 'xs', v]`. The list grows when it is full:
    /// doubling, so appending N times costs N and not N².
    pub(super) fn list_append(&mut self, fi: usize, args: &[Operand], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let place = args.first()?.place()?.clone();
        let (at, ty) = self.slot_of(fi, &place, slots)?;
        let elem = Self::elem_of(&ty)?;
        let shape = match self.ty(&ty)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        if shape.count_fields() < 3 {
            // An array never grows; the checker refused this already.
            return None;
        }
        let esize = self.size_of_ty(&elem)?;
        let lty = self.ty(&elem)?;
        let w = self.ctx.i64_type();
        let v = self.operand(fi, args.get(1)?, slots)?;
        let v = self.fit(v, lty);
        let len = self.load_field_int(shape, at, 1);
        let cap = self.load_field_int(shape, at, 2);
        let full = self.builder.build_int_compare(IntPredicate::UGE, len, cap, "full").ok()?;
        let grow = self.ctx.append_basic_block(func, "grow");
        let put = self.ctx.append_basic_block(func, "append");
        let _ = self.builder.build_conditional_branch(full, grow, put);
        self.builder.position_at_end(grow);
        // Four to begin with, then doubling: a first append should not ask
        // the allocator for one element and then two.
        let doubled = self.builder.build_int_mul(cap, w.const_int(2, false), "x2").ok()?;
        let small = self.builder.build_int_compare(IntPredicate::ULT, doubled, w.const_int(4, false), "small").ok()?;
        let fresh_cap = self.builder.build_select(small, w.const_int(4, false), doubled, "cap").ok()?.into_int_value();
        let bytes = self.builder.build_int_mul(fresh_cap, w.const_int(esize, false), "bytes").ok()?;
        let old = self.load_field_ptr(shape, at, 0);
        let re = self.rt("cyborg_realloc");
        let call = self.builder.build_call(re, &[old.into(), bytes.into()], "mem").ok()?;
        let fresh = basic_result(call)?.into_pointer_value();
        self.store_field(shape, at, 0, fresh.into());
        self.store_field(shape, at, 2, fresh_cap.into());
        let _ = self.builder.build_unconditional_branch(put);
        self.builder.position_at_end(put);
        let data = self.load_field_ptr(shape, at, 0);
        let off = self.builder.build_int_mul(len, w.const_int(esize, false), "off").ok()?;
        let slot = unsafe { self.builder.build_gep(self.ctx.i8_type(), data, &[off], "end").ok()? };
        let _ = self.builder.build_store(slot, v);
        let next = self.builder.build_int_add(len, w.const_int(1, false), "len").ok()?;
        self.store_field(shape, at, 1, next.into());
        Some(())
    }

    /// `list.remove[lend.mut 'xs', pos]` — hands the element back (decided
    /// 2026-09-21), so the list stops owning it and the caller starts.
    /// What followed it slides down; the order is the list's meaning.
    pub(super) fn list_remove(&mut self, fi: usize, args: &[Operand], slots: &[Option<PointerValue<'ctx>>]) -> Option<BasicValueEnum<'ctx>> {
        let place = args.first()?.place()?.clone();
        let (at, ty) = self.slot_of(fi, &place, slots)?;
        let elem = Self::elem_of(&ty)?;
        let shape = match self.ty(&ty)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        let esize = self.size_of_ty(&elem)?;
        let lty = self.ty(&elem)?;
        let w = self.ctx.i64_type();
        let pos = self.operand(fi, args.get(1)?, slots)?;
        let pos = self.fit(pos, w.into()).into_int_value();
        let (slot, _) = self.index_slot(at, &ty, pos)?;
        let taken = self.builder.build_load(lty, slot, "taken").ok()?;
        // Everything after it moves down one place. The regions overlap,
        // so this is a move and not a copy.
        let len = self.load_field_int(shape, at, 1);
        let zero_based = self.builder.build_int_sub(pos, w.const_int(1, false), "zb").ok()?;
        let after = self.builder.build_int_sub(len, zero_based, "after").ok()?;
        let after = self.builder.build_int_sub(after, w.const_int(1, false), "tail").ok()?;
        let bytes = self.builder.build_int_mul(after, w.const_int(esize, false), "bytes").ok()?;
        let next = unsafe { self.builder.build_gep(self.ctx.i8_type(), slot, &[w.const_int(esize, false)], "next").ok()? };
        let mv = self.rt("cyborg_move");
        let _ = self.builder.build_call(mv, &[slot.into(), next.into(), bytes.into()], "");
        let shorter = self.builder.build_int_sub(len, w.const_int(1, false), "len").ok()?;
        self.store_field(shape, at, 1, shorter.into());
        Some(taken)
    }

    /// Give a local's value back, if it still has one. A value handed away
    /// on one path and kept on another needs a flag saying which happened;
    /// `define` gives every owning local one, and LLVM removes the ones
    /// whose answer is always the same.
    pub(super) fn emit_drop(&mut self, place: &Place, slot: PointerValue<'ctx>, ty: &Ty, flag: Option<PointerValue<'ctx>>, func: FunctionValue<'ctx>) {
        if !place.proj.is_empty() {
            return;
        }
        let Some(dropper) = self.drop_fn(ty) else { return };
        match flag {
            Some(f) => {
                let alive = self.builder.build_load(self.ctx.bool_type(), f, "alive").unwrap().into_int_value();
                let yes = self.ctx.append_basic_block(func, "drop");
                let done = self.ctx.append_basic_block(func, "dropped");
                let _ = self.builder.build_conditional_branch(alive, yes, done);
                self.builder.position_at_end(yes);
                let _ = self.builder.build_call(dropper, &[slot.into()], "");
                let _ = self.builder.build_store(f, self.ctx.bool_type().const_zero());
                let _ = self.builder.build_unconditional_branch(done);
                self.builder.position_at_end(done);
            }
            None => {
                let _ = self.builder.build_call(dropper, &[slot.into()], "");
            }
        }
    }

    /// The value is here now.
    pub(super) fn mark_alive(&mut self, local: LocalId, alive: bool) {
        let Some(Some(f)) = self.cur_flags.get(local.0 as usize).copied() else { return };
        let v = self.ctx.bool_type().const_int(alive as u64, false);
        let _ = self.builder.build_store(f, v);
    }

    // ------------------------------------------------------------ helpers

    fn load_field_ptr(&mut self, shape: inkwell::types::StructType<'ctx>, at: PointerValue<'ctx>, i: u32) -> PointerValue<'ctx> {
        let p = self.builder.build_struct_gep(shape, at, i, "fp").unwrap();
        self.builder.build_load(self.ctx.ptr_type(AddressSpace::default()), p, "ptr").unwrap().into_pointer_value()
    }

    fn load_field_int(&mut self, shape: inkwell::types::StructType<'ctx>, at: PointerValue<'ctx>, i: u32) -> IntValue<'ctx> {
        let p = self.builder.build_struct_gep(shape, at, i, "fi").unwrap();
        self.builder.build_load(self.ctx.i64_type(), p, "n").unwrap().into_int_value()
    }

    fn store_field(&mut self, shape: inkwell::types::StructType<'ctx>, at: PointerValue<'ctx>, i: u32, v: BasicValueEnum<'ctx>) {
        if let Ok(p) = self.builder.build_struct_gep(shape, at, i, "fs") {
            let _ = self.builder.build_store(p, v);
        }
    }

    /// `for each of the first `len` elements at `data`` — a counted loop,
    /// so a list of lists frees every one of them.
    fn each_element<F>(&mut self, data: PointerValue<'ctx>, len: IntValue<'ctx>, esize: u64, func: FunctionValue<'ctx>, body: F)
    where
        F: FnOnce(&mut Self, PointerValue<'ctx>),
    {
        let w = self.ctx.i64_type();
        let i_slot = self.builder.build_alloca(w, "i").unwrap();
        let _ = self.builder.build_store(i_slot, w.const_zero());
        let head = self.ctx.append_basic_block(func, "each");
        let step = self.ctx.append_basic_block(func, "each_body");
        let done = self.ctx.append_basic_block(func, "each_done");
        let _ = self.builder.build_unconditional_branch(head);
        self.builder.position_at_end(head);
        let i = self.builder.build_load(w, i_slot, "i").unwrap().into_int_value();
        let more = self.builder.build_int_compare(IntPredicate::ULT, i, len, "more").unwrap();
        let _ = self.builder.build_conditional_branch(more, step, done);
        self.builder.position_at_end(step);
        let off = self.builder.build_int_mul(i, w.const_int(esize, false), "off").unwrap();
        let slot = unsafe { self.builder.build_gep(self.ctx.i8_type(), data, &[off], "at").unwrap() };
        body(self, slot);
        let next = self.builder.build_int_add(i, w.const_int(1, false), "next").unwrap();
        let _ = self.builder.build_store(i_slot, next);
        let _ = self.builder.build_unconditional_branch(head);
        self.builder.position_at_end(done);
    }
}

// ------------------------------------------------------------------ enums
//
// `enums.md`: "one of a few named cases", where a case may carry a
// detail. On the machine that is a tag saying which case, and room for
// the largest detail any case carries. The design decided what an enum
// *means*; the shape below is the lowering's own and is recorded as
// provisional in `cca/README.md`.

impl<'a, 'ctx> Cg<'a, 'ctx> {
    /// The tag, and the room beside it. The tag is the smallest whole
    /// number that can name every case, so a three-case enum pays one
    /// byte for it; the room is an array whose element type carries the
    /// alignment the widest payload needs, which is what puts the payload
    /// at an address it can actually be stored at.
    pub(super) fn enum_shape(&mut self, id: EnumId, args: &[Ty]) -> Option<inkwell::types::StructType<'ctx>> {
        let info = self.p.items.enums[id.0 as usize].clone();
        let payloads: Vec<Option<Ty>> = info.cases.iter().map(|c| c.payload.as_ref().map(|t| t.subst(args))).collect();
        self.variant_shape(&payloads)
    }

    /// One of several cases, each of which may carry something: an enum,
    /// and equally what a function that can fail hands back.
    pub(super) fn variant_shape(&mut self, payloads: &[Option<Ty>]) -> Option<inkwell::types::StructType<'ctx>> {
        let tag = self.enum_tag_type(payloads.len());
        let mut size = 0u64;
        let mut align = 1u64;
        for pt in payloads.iter().flatten() {
            let l = self.ty(pt)?;
            size = size.max(self.target.get_store_size(&l));
            align = align.max(self.target.get_abi_alignment(&l) as u64);
        }
        if size == 0 {
            return Some(self.ctx.struct_type(&[tag.into()], false));
        }
        let cell: BasicTypeEnum<'ctx> = match align {
            0 | 1 => self.ctx.i8_type().into(),
            2 => self.ctx.i16_type().into(),
            4 => self.ctx.i32_type().into(),
            8 => self.ctx.i64_type().into(),
            _ => self.ctx.i128_type().into(),
        };
        let cell_size = self.target.get_store_size(&cell).max(1);
        let cells = size.div_ceil(cell_size);
        let room = match cell {
            BasicTypeEnum::IntType(i) => i.array_type(cells as u32),
            _ => return None,
        };
        Some(self.ctx.struct_type(&[tag.into(), room.into()], false))
    }

    fn enum_tag_type(&self, cases: usize) -> inkwell::types::IntType<'ctx> {
        if cases <= 0xff {
            self.ctx.i8_type()
        } else if cases <= 0xffff {
            self.ctx.i16_type()
        } else {
            self.ctx.i32_type()
        }
    }

    /// `Status.claimed["Tankun"]` — the tag, and the detail beside it.
    pub(super) fn make_enum(&mut self, fi: usize, id: EnumId, case: usize, payload: &Option<Operand>, ty: &Ty, slots: &[Option<PointerValue<'ctx>>]) -> Option<BasicValueEnum<'ctx>> {
        let shape = match self.ty(ty)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        // Built in memory, because the detail is stored at its own type
        // and read back at the enum's: a register cannot hold both views.
        let tmp = self.builder.build_alloca(shape, "case").ok()?;
        let info = self.p.items.enums[id.0 as usize].clone();
        let tag = self.enum_tag_type(info.cases.len());
        self.store_field(shape, tmp, 0, tag.const_int(case as u64, false).into());
        if let Some(o) = payload {
            let v = self.operand(fi, o, slots)?;
            let at = self.builder.build_struct_gep(shape, tmp, 1, "detail").ok()?;
            let want = self.case_payload_ty(id, case, ty).and_then(|t| self.ty(&t));
            let v = match want {
                Some(t) => self.fit(v, t),
                None => v,
            };
            let _ = self.builder.build_store(at, v);
        }
        self.builder.build_load(shape, tmp, "value").ok()
    }

    /// What a case carries, with the enum's own type arguments put in.
    pub(super) fn case_payload_ty(&self, id: EnumId, case: usize, ty: &Ty) -> Option<Ty> {
        let args = match ty.base() {
            Ty::Enum { args, .. } => args.clone(),
            _ => Vec::new(),
        };
        self.p.items.enums[id.0 as usize].cases.get(case)?.payload.clone().map(|t| t.subst(&args))
    }

    /// The payload of one case, as a place. `match` binds it this way, and
    /// MIR spells it as a field whose index is the case.
    pub(super) fn case_slot(&mut self, at: PointerValue<'ctx>, ty: &Ty) -> Option<PointerValue<'ctx>> {
        let shape = match self.ty(ty)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        if shape.count_fields() < 2 {
            return None;
        }
        self.builder.build_struct_gep(shape, at, 1, "detail").ok()
    }

    /// `match` — every case is covered (`enums.md`), so there is no
    /// default arm to write. LLVM wants one anyway; the one it gets can
    /// never be reached.
    pub(super) fn emit_match(&mut self, fi: usize, value: &Operand, cases: &[(usize, BlockId)], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let place = value.place()?.clone();
        let (at, ty) = self.slot_of(fi, &place, slots)?;
        let shape = match self.ty(&ty)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        let id = match ty.base() {
            Ty::Enum { id, .. } => *id,
            _ => return None,
        };
        let count = self.p.items.enums[id.0 as usize].cases.len();
        let tag_ty = self.enum_tag_type(count);
        let p = self.builder.build_struct_gep(shape, at, 0, "tag").ok()?;
        let tag = self.builder.build_load(tag_ty, p, "which").ok()?.into_int_value();
        let unreached = self.ctx.append_basic_block(func, "no_such_case");
        let arms: Vec<(IntValue<'ctx>, LlBlock<'ctx>)> = cases.iter().map(|(c, b)| (tag_ty.const_int(*c as u64, false), blocks[b.0 as usize])).collect();
        let _ = self.builder.build_switch(tag, unreached, &arms);
        self.builder.position_at_end(unreached);
        let _ = self.builder.build_unreachable();
        Some(())
    }
}

// --------------------------------------------------------------- failures
//
// `failures.md`: "a function that can fail says so in its signature and
// hands the failure back as a value". On the machine that is exactly a
// variant — case 0 is the answer, each later case is one of the reasons —
// so it is the enum shape again, and the caller reads the tag to know
// which happened. The design decided the model and left the shape open.

impl<'a, 'ctx> Cg<'a, 'ctx> {
    /// What a function that can fail hands back: the answer, or one of
    /// its reasons.
    pub(super) fn fail_shape(&mut self, fi: usize) -> Option<inkwell::types::StructType<'ctx>> {
        let f = &self.p.functions[fi];
        let ret = f.ret.clone();
        let fails = f.fails.clone();
        let mut payloads: Vec<Option<Ty>> = Vec::with_capacity(fails.len() + 1);
        payloads.push(if matches!(ret, Ty::Nothing) { None } else { Some(ret) });
        for (_, t) in &fails {
            payloads.push(t.clone());
        }
        self.variant_shape(&payloads)
    }

    /// Case 0 is the answer; reason `i` is case `i + 1`.
    fn variant_value(&mut self, shape: inkwell::types::StructType<'ctx>, case: usize, cases: usize, payload: Option<BasicValueEnum<'ctx>>) -> Option<BasicValueEnum<'ctx>> {
        let tmp = self.builder.build_alloca(shape, "outcome").ok()?;
        let tag = self.enum_tag_type(cases);
        self.store_field(shape, tmp, 0, tag.const_int(case as u64, false).into());
        if let Some(v) = payload {
            let at = self.builder.build_struct_gep(shape, tmp, 1, "carried").ok()?;
            let _ = self.builder.build_store(at, v);
        }
        self.builder.build_load(shape, tmp, "gave").ok()
    }

    /// `fail name;` — leave, saying which reason and carrying what it
    /// carries.
    pub(super) fn emit_fail(&mut self, fi: usize, index: usize, payload: &Option<Operand>, slots: &[Option<PointerValue<'ctx>>]) -> Option<()> {
        let shape = self.fail_shape(fi)?;
        let cases = self.p.functions[fi].fails.len() + 1;
        let v = match payload {
            Some(o) => {
                let raw = self.operand(fi, o, slots)?;
                let want = self.p.functions[fi].fails.get(index)?.1.clone().and_then(|t| self.ty(&t));
                Some(match want {
                    Some(t) => self.fit(raw, t),
                    None => raw,
                })
            }
            None => None,
        };
        let out = self.variant_value(shape, index + 1, cases, v)?;
        let _ = self.builder.build_return(Some(&out));
        Some(())
    }

    /// `return` from a function that can fail: case 0, the answer.
    pub(super) fn emit_ok_return(&mut self, fi: usize, slots: &[Option<PointerValue<'ctx>>]) -> Option<()> {
        let shape = self.fail_shape(fi)?;
        let f = &self.p.functions[fi];
        let cases = f.fails.len() + 1;
        let answer = match f.ret_local {
            Some(r) if !matches!(f.locals[r.0 as usize].ty, Ty::Nothing) => {
                let ty = f.locals[r.0 as usize].ty.clone();
                let slot = slots[r.0 as usize]?;
                let t = self.ty(&ty)?;
                Some(self.builder.build_load(t, slot, "answer").ok()?)
            }
            _ => None,
        };
        let out = self.variant_value(shape, 0, cases, answer)?;
        let _ = self.builder.build_return(Some(&out));
        Some(())
    }

    /// A call that can fail: the caller must say what happens
    /// (`failures.md`), and MIR has already turned each `otherwise` arm
    /// into its own block. Read the tag and go where it says.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_failing_call(&mut self, fi: usize, callee: &Callee, args: &[Arg], dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let target = match callee {
            Callee::Fn(idx) => idx.0 as usize,
            _ => return None,
        };
        let outcome = self.call(fi, callee, args, slots)?;
        let shape = self.fail_shape(target)?;
        let cases = self.p.functions[target].fails.len() + 1;
        let tmp = self.builder.build_alloca(shape, "outcome").ok()?;
        let _ = self.builder.build_store(tmp, outcome);
        let tag_ty = self.enum_tag_type(cases);
        let p = self.builder.build_struct_gep(shape, tmp, 0, "which").ok()?;
        let tag = self.builder.build_load(tag_ty, p, "tag").ok()?.into_int_value();
        // Where the answer is unpacked, and one block per reason.
        let good = self.ctx.append_basic_block(func, "gave");
        let mut arms: Vec<(IntValue<'ctx>, LlBlock<'ctx>)> = vec![(tag_ty.const_zero(), good)];
        let mut landings = Vec::new();
        for arm in fail {
            let land = self.ctx.append_basic_block(func, "failed");
            arms.push((tag_ty.const_int(arm.index as u64 + 1, false), land));
            landings.push((arm.clone(), land));
        }
        let unreached = self.ctx.append_basic_block(func, "no_such_outcome");
        let _ = self.builder.build_switch(tag, unreached, &arms);
        self.builder.position_at_end(unreached);
        let _ = self.builder.build_unreachable();
        // The answer.
        self.builder.position_at_end(good);
        if let Some(d) = dest {
            let ret = self.p.functions[target].ret.clone();
            if let Some(t) = self.ty(&ret) {
                if let Ok(at) = self.builder.build_struct_gep(shape, tmp, 1, "answer") {
                    if let Ok(v) = self.builder.build_load(t, at, "v") {
                        self.store_place(fi, d, v, slots);
                        if d.proj.is_empty() && self.owns_memory(&ret) {
                            self.mark_alive(d.local, true);
                        }
                    }
                }
            }
        }
        let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
        // Each reason, with what it carried.
        for (arm, land) in landings {
            self.builder.position_at_end(land);
            if let Some(place) = &arm.payload {
                let carried = self.p.functions[target].fails.get(arm.index).and_then(|(_, t)| t.clone());
                if let Some(ct) = carried.and_then(|t| self.ty(&t)) {
                    if let Ok(at) = self.builder.build_struct_gep(shape, tmp, 1, "carried") {
                        if let Ok(v) = self.builder.build_load(ct, at, "why") {
                            self.store_place(fi, place, v, slots);
                        }
                    }
                }
            }
            let _ = self.builder.build_unconditional_branch(blocks[arm.block.0 as usize]);
        }
        Some(())
    }
}

// ------------------------------------------------------------------ text
//
// A text is a pointer and a length. The bytes are either the binary's own
// — a literal needs no heap — or built at run time, and freeing the first
// kind would crash. Which memory a text is in is Precog's to know (owner,
// 2026-09-22); until Precog carries that fact, every handle says so
// itself, in the high bit of the length. That is the fallback the same
// decision names, paid everywhere rather than only where it is needed.

impl<'a, 'ctx> Cg<'a, 'ctx> {
    /// A length with the mark taken off: what the program means by it.
    pub(super) fn real_length(&mut self, raw: IntValue<'ctx>) -> IntValue<'ctx> {
        let mask = self.ctx.i64_type().const_int(!(1u64 << 63), false);
        self.builder.build_and(raw, mask, "len").unwrap_or(raw)
    }

    /// `'a' ++ 'b'` — one new text holding the other two.
    pub(super) fn str_concat(&mut self, a: BasicValueEnum<'ctx>, b: BasicValueEnum<'ctx>, ty: &Ty) -> Option<BasicValueEnum<'ctx>> {
        let shape = match self.ty(ty)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        let (ap, alen) = self.text_parts(a)?;
        let (bp, blen) = self.text_parts(b)?;
        let out = self.builder.build_alloca(shape, "joined").ok()?;
        let f = self.rt("cyborg_str_concat");
        let _ = self.builder.build_call(f, &[out.into(), ap.into(), alen.into(), bp.into(), blen.into()], "");
        self.builder.build_load(shape, out, "text").ok()
    }

    /// The pointer and the raw length out of a text value.
    fn text_parts(&mut self, v: BasicValueEnum<'ctx>) -> Option<(PointerValue<'ctx>, IntValue<'ctx>)> {
        let agg = match v {
            BasicValueEnum::StructValue(s) => s,
            _ => return None,
        };
        let p = self.builder.build_extract_value(agg, 0, "p").ok()?.into_pointer_value();
        let n = self.builder.build_extract_value(agg, 1, "n").ok()?.into_int_value();
        Some((p, n))
    }
}

