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
            // The map and hashing side keeps its own list.
            other => return self.rt_map(other),
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
            Ty::Seq { .. } | Ty::Map { .. } | Ty::Channel(_) => true,
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
            Ty::Map { .. } => {
                let _ = self.emit_map_drop(t, at);
            }
            Ty::Channel(elem) => {
                // What is still in it goes too.
                let p = self.ctx.ptr_type(AddressSpace::default());
                let ch = self.builder.build_load(p, at, "ch").unwrap().into_pointer_value();
                let dropper = self.drop_fn(&elem).map(|f| f.as_global_value().as_pointer_value()).unwrap_or_else(|| p.const_null());
                let f = self.rt_chan_drop();
                let _ = self.builder.build_call(f, &[ch.into(), dropper.into()], "");
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
            Ty::Map { .. } => {
                let _ = self.emit_map_copy(t, dst, src, func);
            }
            Ty::Channel(_) => {
                // Two channels would be a new channel, not a second way to
                // reach this one; the design does not say which `copy` means.
                self.cannot("a `copy` of a channel: the design does not say whether that is a second channel or a second way to reach this one", None);
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

    /// `list.slice[lend 'xs', pos a, pos b]` — a new list owning copies of
    /// elements a to b (`lists.md`). The positions were checked at their
    /// own site: `1 <= a <= b + 1 <= length + 1`, so the count is never
    /// negative and the copy never leaves the list. Always a copy for now;
    /// eliding it where nobody can tell is Precog's, later.
    pub(super) fn list_slice(&mut self, fi: usize, args: &[Operand], out: &Ty, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let place = args.first()?.place()?.clone();
        let (at, ty) = self.slot_of(fi, &place, slots)?;
        let elem = Self::elem_of(&ty)?;
        let from = match self.ty(&ty)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        let shape = match self.ty(out)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        let esize = self.size_of_ty(&elem)?;
        let w = self.ctx.i64_type();
        let a = self.operand(fi, args.get(1)?, slots)?;
        let a = self.fit(a, w.into()).into_int_value();
        let b = self.operand(fi, args.get(2)?, slots)?;
        let b = self.fit(b, w.into()).into_int_value();
        let n = self.builder.build_int_sub(b, a, "span").ok()?;
        let n = self.builder.build_int_add(n, w.const_int(1, false), "count").ok()?;
        let bytes = self.builder.build_int_mul(n, w.const_int(esize, false), "bytes").ok()?;
        let data = self.load_field_ptr(from, at, 0);
        let skip = self.builder.build_int_sub(a, w.const_int(1, false), "zb").ok()?;
        let skip = self.builder.build_int_mul(skip, w.const_int(esize, false), "skip").ok()?;
        let src = unsafe { self.builder.build_gep(self.ctx.i8_type(), data, &[skip], "from").ok()? };
        let fresh = self.alloc(bytes);
        let cp = self.rt("cyborg_copy");
        let _ = self.builder.build_call(cp, &[fresh.into(), src.into(), bytes.into()], "");
        // As in a whole list's copy: an owning element is copied over its
        // own shallow twin, or two lists would free one buffer twice.
        if let Some(inner) = self.copy_fn(&elem) {
            self.each_element(fresh, n, esize, func, move |cg, slot| {
                let _ = cg.builder.build_call(inner, &[slot.into(), slot.into()], "");
            });
        }
        let mut agg = shape.get_undef();
        agg = self.builder.build_insert_value(agg, fresh, 0, "p").ok()?.into_struct_value();
        agg = self.builder.build_insert_value(agg, n, 1, "n").ok()?.into_struct_value();
        if shape.count_fields() > 2 {
            agg = self.builder.build_insert_value(agg, n, 2, "c").ok()?.into_struct_value();
        }
        Some(agg.into())
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

    pub(super) fn enum_tag_type(&self, cases: usize) -> inkwell::types::IntType<'ctx> {
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
    /// An extern whose return is a range type (`ffi.md`, "A ranged return —
    /// C's contract, checked at the door"): C's answer is compared with the
    /// range, and outside it the call fails `out_of_range`, which the caller
    /// handles. A failure, not a check: the final build keeps it.
    #[allow(clippy::too_many_arguments)]
    fn emit_ranged_extern(&mut self, fi: usize, id: crate::hir::ExternId, callee: &Callee, args: &[Arg], dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let ret = self.p.items.externs[id.0 as usize].ret.clone();
        let Some(crate::types::Bound::Int { lo, hi }) = ret.bound().cloned() else { return None };
        let signed = matches!(ret.base(), Ty::Int(crate::types::IntW::Fixed { signed: true, .. }));
        let v = self.call(fi, callee, args, slots)?.into_int_value();
        let t = v.get_type();
        let (below, above) = if signed {
            (self.builder.build_int_compare(IntPredicate::SLT, v, super::const_i128(t, lo), "below").ok()?, self.builder.build_int_compare(IntPredicate::SGT, v, super::const_i128(t, hi), "above").ok()?)
        } else {
            (self.builder.build_int_compare(IntPredicate::ULT, v, super::const_i128(t, lo), "below").ok()?, self.builder.build_int_compare(IntPredicate::UGT, v, super::const_i128(t, hi), "above").ok()?)
        };
        let outside = self.builder.build_or(below, above, "out_of_range").ok()?;
        let inside = self.ctx.append_basic_block(func, "in_range");
        // `out_of_range` is the one reason; an arm for it, or none when the
        // caller's `otherwise` gives a value or leaves the same way for all.
        let out = fail.first().map(|a| blocks[a.block.0 as usize])?;
        let _ = self.builder.build_conditional_branch(outside, out, inside);
        self.builder.position_at_end(inside);
        if let Some(d) = dest {
            self.store_place(fi, d, v.into(), slots);
        }
        let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
        Some(())
    }

    pub(super) fn emit_failing_call(&mut self, fi: usize, callee: &Callee, args: &[Arg], dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        if let Callee::Extern(id) = callee {
            return self.emit_ranged_extern(fi, *id, callee, args, dest, ok, fail, blocks, slots, func);
        }
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


// ------------------------------------------------------------------- maps
//
// `maps.md` (owner, 2026-09-22): one flat table, open addressing, never a
// linked bucket per entry; an unordered handle is three fields and an
// ordered one is four, the fourth pointing at the insertion-order list.
// The table itself is the runtime's (`cyborg_map_*`); what the compiler
// adds is everything that depends on the types — how big a key is, how to
// hash one, when two are the same, and what a key or a value owns.
//
// Hashing is *by structure*, field by field and element by element, which
// is what the owner chose: "`map[Point, str.utf8]` just works". A type
// with its own `hash` waits for the check that equal keys hash the same.

impl<'a, 'ctx> Cg<'a, 'ctx> {
    /// Is this a map that remembers the order things were put in? A map
    /// Precog has not decided about is treated as ordered, because that
    /// is the one that can do both.
    pub(super) fn map_is_ordered(&self, t: &Ty) -> bool {
        !matches!(t.base(), Ty::Map { order: crate::types::MapOrder::Unordered, .. })
    }

    fn map_key_val(t: &Ty) -> Option<(Ty, Ty)> {
        match t.base() {
            Ty::Map { key, val, .. } => Some(((**key).clone(), (**val).clone())),
            _ => None,
        }
    }

    /// What the runtime needs to know about one map's types, as a
    /// constant it can be handed.
    pub(super) fn map_spec(&mut self, t: &Ty) -> Option<PointerValue<'ctx>> {
        let (k, v) = Self::map_key_val(t)?;
        let name = format!("cy_mapspec_{}_{}", self.type_key(&k), self.type_key(&v));
        if let Some(g) = self.module.get_global(&name) {
            return Some(g.as_pointer_value());
        }
        let w = self.ctx.i64_type();
        let p = self.ctx.ptr_type(AddressSpace::default());
        // `maps.md` lets a key type bring its own `hash` and `op ==`, and
        // makes "equal keys hash the same" a check site Precog must prove
        // or the final build refuses the map. Precog does not prove that
        // yet, so a key with its own rule is refused rather than quietly
        // hashed by structure — which would put two keys the program
        // calls equal in two different slots.
        if let Some(own) = self.own_hash(&k) {
            self.cannot(format!("`{}` has its own `hash`, and a map keyed by it waits on the check that equal keys hash the same", own), None);
            return None;
        }
        let ks = self.size_of_ty(&k)?;
        let vs = self.size_of_ty(&v)?;
        let hash = self.hash_fn(&k)?;
        let same = self.same_fn(&k)?;
        let drop_k = self.drop_fn(&k).map(|f| f.as_global_value().as_pointer_value()).unwrap_or_else(|| p.const_null());
        let drop_v = self.drop_fn(&v).map(|f| f.as_global_value().as_pointer_value()).unwrap_or_else(|| p.const_null());
        let shape = self.ctx.struct_type(&[w.into(), w.into(), p.into(), p.into(), p.into(), p.into()], false);
        let init = shape.const_named_struct(&[
            w.const_int(ks, false).into(),
            w.const_int(vs, false).into(),
            hash.as_global_value().as_pointer_value().into(),
            same.as_global_value().as_pointer_value().into(),
            drop_k.into(),
            drop_v.into(),
        ]);
        let g = self.module.add_global(shape, None, &name);
        g.set_initializer(&init);
        g.set_constant(true);
        g.set_linkage(Linkage::Internal);
        Some(g.as_pointer_value())
    }

    /// Does this type bring its own `hash`? The interface is
    /// `fn hash [lent 'v': <T>] -> uint64` (`maps.md`).
    fn own_hash(&self, t: &Ty) -> Option<String> {
        let name = crate::types::TySpelling { ty: t, structs: &self.p.items.structs, enums: &self.p.items.enums }.to_string();
        let found = self.p.items.fns.iter().any(|f| {
            f.name == "hash"
                && f.params.len() == 1
                && f.params[0].ty.as_ref().map(|t| crate::types::TySpelling { ty: t, structs: &self.p.items.structs, enums: &self.p.items.enums }.to_string() == name).unwrap_or(false)
        });
        if found {
            Some(name)
        } else {
            None
        }
    }

    /// `cy_hash_T(ptr) -> u64`, by structure.
    fn hash_fn(&mut self, t: &Ty) -> Option<FunctionValue<'ctx>> {
        let name = format!("cy_hash_{}", self.type_key(t));
        if let Some(f) = self.helpers.get(&name) {
            return Some(*f);
        }
        let p = self.ctx.ptr_type(AddressSpace::default());
        let w = self.ctx.i64_type();
        let func = self.module.add_function(&name, w.fn_type(&[p.into()], false), Some(Linkage::Internal));
        self.helpers.insert(name, func);
        let here = self.builder.get_insert_block();
        let entry = self.ctx.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);
        let at = func.get_nth_param(0).unwrap().into_pointer_value();
        let h = self.emit_hash(t, at, func);
        let _ = self.builder.build_return(Some(&h.unwrap_or_else(|| w.const_zero())));
        if let Some(b) = here {
            self.builder.position_at_end(b);
        }
        h?;
        Some(func)
    }

    fn emit_hash(&mut self, t: &Ty, at: PointerValue<'ctx>, func: FunctionValue<'ctx>) -> Option<IntValue<'ctx>> {
        let w = self.ctx.i64_type();
        match t.base() {
            Ty::Int(_) | Ty::Bool | Ty::Char(_) | Ty::Duration => {
                let l = self.ty(t)?;
                let v = self.builder.build_load(l, at, "v").ok()?.into_int_value();
                let v = self.builder.build_int_z_extend_or_bit_cast(v, w, "wide").ok()?;
                let f = self.rt("cyborg_hash_int");
                basic_result(self.builder.build_call(f, &[v.into()], "h").ok()?).map(|x| x.into_int_value())
            }
            Ty::Str(_) => {
                let shape = match self.ty(t)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let ptr = self.load_field_ptr(shape, at, 0);
                let len = self.load_field_int(shape, at, 1);
                let f = self.rt("cyborg_hash_bytes");
                basic_result(self.builder.build_call(f, &[ptr.into(), len.into()], "h").ok()?).map(|x| x.into_int_value())
            }
            Ty::Struct { id, args } => {
                let info = self.p.items.structs[id.0 as usize].clone();
                let shape = match self.ty(t)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let mut acc = w.const_int(0x9e37_79b9, false);
                for (i, fd) in info.fields.iter().enumerate() {
                    let ft = fd.ty.subst(&args);
                    let fp = self.builder.build_struct_gep(shape, at, i as u32, "f").ok()?;
                    let h = self.emit_hash(&ft, fp, func)?;
                    let mix = self.rt("cyborg_hash_mix");
                    acc = basic_result(self.builder.build_call(mix, &[acc.into(), h.into()], "mix").ok()?)?.into_int_value();
                }
                Some(acc)
            }
            Ty::Seq { .. } => {
                // Every element, in order: two lists are the same key only
                // when they hold the same things in the same places.
                let elem = Self::elem_of(t)?;
                let esize = self.size_of_ty(&elem)?;
                let shape = match self.ty(t)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let data = self.load_field_ptr(shape, at, 0);
                let len = self.load_field_int(shape, at, 1);
                let acc = self.builder.build_alloca(w, "acc").ok()?;
                let _ = self.builder.build_store(acc, w.const_int(0x517c_c1b7, false));
                let inner = self.hash_fn(&elem)?;
                let mix = self.rt("cyborg_hash_mix");
                self.each_element(data, len, esize, func, move |cg, slot| {
                    let Ok(call) = cg.builder.build_call(inner, &[slot.into()], "h") else { return };
                    let Some(h) = basic_result(call) else { return };
                    let Ok(old) = cg.builder.build_load(w, acc, "old") else { return };
                    let Ok(next) = cg.builder.build_call(mix, &[old.into_int_value().into(), h.into_int_value().into()], "mix") else { return };
                    if let Some(v) = basic_result(next) {
                        let _ = cg.builder.build_store(acc, v);
                    }
                });
                self.builder.build_load(w, acc, "hash").ok().map(|v| v.into_int_value())
            }
            _ => None,
        }
    }

    /// `cy_same_T(a, b) -> bool`, by structure.
    fn same_fn(&mut self, t: &Ty) -> Option<FunctionValue<'ctx>> {
        let name = format!("cy_same_{}", self.type_key(t));
        if let Some(f) = self.helpers.get(&name) {
            return Some(*f);
        }
        let p = self.ctx.ptr_type(AddressSpace::default());
        let func = self.module.add_function(&name, self.ctx.bool_type().fn_type(&[p.into(), p.into()], false), Some(Linkage::Internal));
        self.helpers.insert(name, func);
        let here = self.builder.get_insert_block();
        let entry = self.ctx.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);
        let a = func.get_nth_param(0).unwrap().into_pointer_value();
        let b = func.get_nth_param(1).unwrap().into_pointer_value();
        let same = self.emit_same(t, a, b, func);
        let _ = self.builder.build_return(Some(&same.unwrap_or_else(|| self.ctx.bool_type().const_zero())));
        if let Some(bb) = here {
            self.builder.position_at_end(bb);
        }
        same?;
        Some(func)
    }

    fn emit_same(&mut self, t: &Ty, a: PointerValue<'ctx>, b: PointerValue<'ctx>, func: FunctionValue<'ctx>) -> Option<IntValue<'ctx>> {
        match t.base() {
            Ty::Int(_) | Ty::Bool | Ty::Char(_) | Ty::Duration => {
                let l = self.ty(t)?;
                let x = self.builder.build_load(l, a, "a").ok()?.into_int_value();
                let y = self.builder.build_load(l, b, "b").ok()?.into_int_value();
                self.builder.build_int_compare(IntPredicate::EQ, x, y, "same").ok()
            }
            Ty::Str(_) => {
                let shape = match self.ty(t)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let ap = self.load_field_ptr(shape, a, 0);
                let alen = self.load_field_int(shape, a, 1);
                let bp = self.load_field_ptr(shape, b, 0);
                let blen = self.load_field_int(shape, b, 1);
                let f = self.rt("cyborg_same_text");
                basic_result(self.builder.build_call(f, &[ap.into(), alen.into(), bp.into(), blen.into()], "same").ok()?).map(|x| x.into_int_value())
            }
            Ty::Struct { id, args } => {
                let info = self.p.items.structs[id.0 as usize].clone();
                let shape = match self.ty(t)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let mut acc = self.ctx.bool_type().const_int(1, false);
                for (i, fd) in info.fields.iter().enumerate() {
                    let ft = fd.ty.subst(&args);
                    let ap = self.builder.build_struct_gep(shape, a, i as u32, "fa").ok()?;
                    let bp = self.builder.build_struct_gep(shape, b, i as u32, "fb").ok()?;
                    let s = self.emit_same(&ft, ap, bp, func)?;
                    acc = self.builder.build_and(acc, s, "both").ok()?;
                }
                Some(acc)
            }
            Ty::Seq { .. } => {
                let elem = Self::elem_of(t)?;
                let esize = self.size_of_ty(&elem)?;
                let shape = match self.ty(t)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let alen = self.load_field_int(shape, a, 1);
                let blen = self.load_field_int(shape, b, 1);
                let adata = self.load_field_ptr(shape, a, 0);
                let bdata = self.load_field_ptr(shape, b, 0);
                let w = self.ctx.i64_type();
                let acc = self.builder.build_alloca(self.ctx.bool_type(), "same").ok()?;
                let lens = self.builder.build_int_compare(IntPredicate::EQ, alen, blen, "lens").ok()?;
                let _ = self.builder.build_store(acc, lens);
                let inner = self.same_fn(&elem)?;
                self.each_element(adata, alen, esize, func, move |cg, slot| {
                    // The matching element of the other list: the same
                    // offset, which is where the walk already is.
                    let Ok(off) = cg.builder.build_ptr_diff(cg.ctx.i8_type(), slot, adata, "off") else { return };
                    let Ok(other) = (unsafe { cg.builder.build_gep(cg.ctx.i8_type(), bdata, &[off], "other") }) else { return };
                    let Ok(call) = cg.builder.build_call(inner, &[slot.into(), other.into()], "e") else { return };
                    let Some(one) = basic_result(call) else { return };
                    let Ok(old) = cg.builder.build_load(cg.ctx.bool_type(), acc, "old") else { return };
                    if let Ok(both) = cg.builder.build_and(old.into_int_value(), one.into_int_value(), "both") {
                        let _ = cg.builder.build_store(acc, both);
                    }
                });
                let _ = w;
                self.builder.build_load(self.ctx.bool_type(), acc, "answer").ok().map(|v| v.into_int_value())
            }
            _ => None,
        }
    }
}

impl<'a, 'ctx> Cg<'a, 'ctx> {
    fn rt_map(&mut self, name: &str) -> FunctionValue<'ctx> {
        if let Some(f) = self.module.get_function(name) {
            return f;
        }
        let p = self.ctx.ptr_type(AddressSpace::default());
        let w = self.ctx.i64_type();
        let b = self.ctx.bool_type();
        let t = match name {
            "cyborg_map_set" => self.ctx.void_type().fn_type(&[p.into(), p.into(), p.into(), p.into(), p.into()], false),
            "cyborg_map_get" => b.fn_type(&[p.into(), p.into(), p.into(), p.into()], false),
            "cyborg_map_remove" => b.fn_type(&[p.into(), p.into(), p.into(), p.into(), p.into()], false),
            "cyborg_map_free" => self.ctx.void_type().fn_type(&[p.into(), p.into(), p.into()], false),
            "cyborg_map_slot" => w.fn_type(&[p.into(), p.into(), p.into(), w.into()], false),
            "cyborg_map_key_at" | "cyborg_map_val_at" => p.fn_type(&[p.into(), p.into(), w.into()], false),
            "cyborg_hash_int" => w.fn_type(&[w.into()], false),
            "cyborg_hash_bytes" => w.fn_type(&[p.into(), w.into()], false),
            "cyborg_hash_mix" => w.fn_type(&[w.into(), w.into()], false),
            "cyborg_same_text" => b.fn_type(&[p.into(), w.into(), p.into(), w.into()], false),
            other => unreachable!("no runtime function named {}", other),
        };
        self.module.add_function(name, t, None)
    }

    /// The address of the handle's order-list field, or null when the map
    /// keeps no order.
    fn map_order_ptr(&mut self, at: PointerValue<'ctx>, ty: &Ty) -> PointerValue<'ctx> {
        let null = self.ctx.ptr_type(AddressSpace::default()).const_null();
        if !self.map_is_ordered(ty) {
            return null;
        }
        let Some(BasicTypeEnum::StructType(shape)) = self.ty(ty) else { return null };
        if shape.count_fields() < 4 {
            return null;
        }
        self.builder.build_struct_gep(shape, at, 3, "order").unwrap_or(null)
    }

    /// A value in memory, which is what every map call wants.
    fn spill(&mut self, v: BasicValueEnum<'ctx>, name: &str) -> Option<PointerValue<'ctx>> {
        let slot = self.builder.build_alloca(v.get_type(), name).ok()?;
        let _ = self.builder.build_store(slot, v);
        Some(slot)
    }

    /// `map["ann" = 5551, …]`, and `map.empty`. The table is built by
    /// setting each pair, so one key written twice behaves as it would
    /// anywhere else (the checker refuses that in a literal).
    pub(super) fn make_map(&mut self, fi: usize, pairs: &[(Operand, Operand)], ty: &Ty, slots: &[Option<PointerValue<'ctx>>]) -> Option<BasicValueEnum<'ctx>> {
        let shape = match self.ty(ty)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        let (kt, vt) = Self::map_key_val(ty)?;
        let (klt, vlt) = (self.ty(&kt)?, self.ty(&vt)?);
        let spec = self.map_spec(ty)?;
        let w = self.ctx.i64_type();
        let null = self.ctx.ptr_type(AddressSpace::default()).const_null();
        let tmp = self.builder.build_alloca(shape, "map").ok()?;
        // Empty: no table, nothing in it, nowhere to put anything yet.
        self.store_field(shape, tmp, 0, null.into());
        self.store_field(shape, tmp, 1, w.const_zero().into());
        self.store_field(shape, tmp, 2, w.const_zero().into());
        if shape.count_fields() > 3 {
            self.store_field(shape, tmp, 3, null.into());
        }
        let order = self.map_order_ptr(tmp, ty);
        let set = self.rt_map("cyborg_map_set");
        for (k, v) in pairs {
            let kv = self.operand(fi, k, slots)?;
            let kv = self.fit(kv, klt);
            let vv = self.operand(fi, v, slots)?;
            let vv = self.fit(vv, vlt);
            let kp = self.spill(kv, "key")?;
            let vp = self.spill(vv, "val")?;
            let _ = self.builder.build_call(set, &[tmp.into(), order.into(), spec.into(), kp.into(), vp.into()], "");
        }
        self.builder.build_load(shape, tmp, "built").ok()
    }

    /// `map.set[lend.mut 'm', k, v]`.
    pub(super) fn map_set(&mut self, fi: usize, args: &[Operand], slots: &[Option<PointerValue<'ctx>>]) -> Option<()> {
        let place = args.first()?.place()?.clone();
        let (at, ty) = self.slot_of(fi, &place, slots)?;
        let (kt, vt) = Self::map_key_val(&ty)?;
        let (klt, vlt) = (self.ty(&kt)?, self.ty(&vt)?);
        let spec = self.map_spec(&ty)?;
        let order = self.map_order_ptr(at, &ty);
        let kv = self.operand(fi, args.get(1)?, slots)?;
        let kv = self.fit(kv, klt);
        let vv = self.operand(fi, args.get(2)?, slots)?;
        let vv = self.fit(vv, vlt);
        let kp = self.spill(kv, "key")?;
        let vp = self.spill(vv, "val")?;
        let set = self.rt_map("cyborg_map_set");
        let _ = self.builder.build_call(set, &[at.into(), order.into(), spec.into(), kp.into(), vp.into()], "");
        Some(())
    }

    /// `map.has[lend 'm', k]`.
    pub(super) fn map_has(&mut self, fi: usize, args: &[Operand], slots: &[Option<PointerValue<'ctx>>]) -> Option<BasicValueEnum<'ctx>> {
        let place = args.first()?.place()?.clone();
        let (at, ty) = self.slot_of(fi, &place, slots)?;
        let (kt, _) = Self::map_key_val(&ty)?;
        let klt = self.ty(&kt)?;
        let spec = self.map_spec(&ty)?;
        let kv = self.operand(fi, args.get(1)?, slots)?;
        let kv = self.fit(kv, klt);
        let kp = self.spill(kv, "key")?;
        let null = self.ctx.ptr_type(AddressSpace::default()).const_null();
        let get = self.rt_map("cyborg_map_get");
        basic_result(self.builder.build_call(get, &[at.into(), spec.into(), kp.into(), null.into()], "has").ok()?)
    }

    /// `map.get` and `map.remove`: a missing key is a failure
    /// (`maps.md`), and `missing` is the only reason either can give, so
    /// the answer is "was it there" and the value beside it.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn map_lookup(&mut self, fi: usize, op: &Op, args: &[Arg], dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let place = match args.first()? {
            Arg::Lend { place, .. } | Arg::Copy(place) | Arg::Give(place) => place.clone(),
            Arg::Value(o) => o.place()?.clone(),
            _ => return None,
        };
        let (at, ty) = self.slot_of(fi, &place, slots)?;
        let (kt, vt) = Self::map_key_val(&ty)?;
        let (klt, vlt) = (self.ty(&kt)?, self.ty(&vt)?);
        let spec = self.map_spec(&ty)?;
        let key_arg = args.get(1)?;
        let kv = match key_arg {
            Arg::Value(o) => self.operand(fi, o, slots)?,
            Arg::Copy(p) | Arg::Give(p) => self.operand(fi, &Operand::Read(p.clone()), slots)?,
            Arg::Lend { place, .. } => {
                let (p, t) = self.slot_of(fi, place, slots)?;
                let l = self.ty(&t)?;
                self.builder.build_load(l, p, "k").ok()?
            }
            _ => return None,
        };
        let kv = self.fit(kv, klt);
        let kp = self.spill(kv, "key")?;
        let out = self.builder.build_alloca(vlt, "found").ok()?;
        let there = match op {
            Op::MapRemove => {
                let order = self.map_order_ptr(at, &ty);
                let f = self.rt_map("cyborg_map_remove");
                basic_result(self.builder.build_call(f, &[at.into(), order.into(), spec.into(), kp.into(), out.into()], "took").ok()?)?
            }
            _ => {
                let f = self.rt_map("cyborg_map_get");
                basic_result(self.builder.build_call(f, &[at.into(), spec.into(), kp.into(), out.into()], "got").ok()?)?
            }
        };
        let good = self.ctx.append_basic_block(func, "found");
        let missing = match fail.first() {
            Some(arm) => blocks[arm.block.0 as usize],
            None => return None,
        };
        let _ = self.builder.build_conditional_branch(there.into_int_value(), good, missing);
        self.builder.position_at_end(good);
        if let Some(d) = dest {
            let v = self.builder.build_load(vlt, out, "value").ok()?;
            self.store_place(fi, d, v, slots);
            if d.proj.is_empty() && self.owns_memory(&vt) {
                self.mark_alive(d.local, true);
            }
        }
        let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
        Some(())
    }

    /// Give a whole map back, and everything its keys and values own.
    pub(super) fn emit_map_drop(&mut self, t: &Ty, at: PointerValue<'ctx>) -> Option<()> {
        let spec = self.map_spec(t)?;
        let order = self.map_order_ptr(at, t);
        let f = self.rt_map("cyborg_map_free");
        let _ = self.builder.build_call(f, &[at.into(), order.into(), spec.into()], "");
        Some(())
    }

    /// A second, independent map: every key and value copied into a table
    /// of its own, in the order they were put in.
    pub(super) fn emit_map_copy(&mut self, t: &Ty, dst: PointerValue<'ctx>, src: PointerValue<'ctx>, func: FunctionValue<'ctx>) -> Option<()> {
        let shape = match self.ty(t)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        let (kt, vt) = Self::map_key_val(t)?;
        let (klt, vlt) = (self.ty(&kt)?, self.ty(&vt)?);
        let spec = self.map_spec(t)?;
        let w = self.ctx.i64_type();
        let null = self.ctx.ptr_type(AddressSpace::default()).const_null();
        self.store_field(shape, dst, 0, null.into());
        self.store_field(shape, dst, 1, w.const_zero().into());
        self.store_field(shape, dst, 2, w.const_zero().into());
        if shape.count_fields() > 3 {
            self.store_field(shape, dst, 3, null.into());
        }
        let dst_order = self.map_order_ptr(dst, t);
        let src_order = if self.map_is_ordered(t) {
            let p = self.builder.build_struct_gep(shape, src, 3, "so").ok()?;
            self.builder.build_load(self.ctx.ptr_type(AddressSpace::default()), p, "order").ok()?.into_pointer_value()
        } else {
            null
        };
        let count = self.load_field_int(shape, src, 1);
        let slot_of = self.rt_map("cyborg_map_slot");
        let key_at = self.rt_map("cyborg_map_key_at");
        let val_at = self.rt_map("cyborg_map_val_at");
        let set = self.rt_map("cyborg_map_set");
        let copy_k = self.copy_fn(&kt);
        let copy_v = self.copy_fn(&vt);
        // A counted walk over the entries, in order.
        let i_slot = self.builder.build_alloca(w, "i").ok()?;
        let _ = self.builder.build_store(i_slot, w.const_zero());
        let head = self.ctx.append_basic_block(func, "entries");
        let body = self.ctx.append_basic_block(func, "entry");
        let done = self.ctx.append_basic_block(func, "entries_done");
        let _ = self.builder.build_unconditional_branch(head);
        self.builder.position_at_end(head);
        let i = self.builder.build_load(w, i_slot, "i").ok()?.into_int_value();
        let more = self.builder.build_int_compare(IntPredicate::ULT, i, count, "more").ok()?;
        let _ = self.builder.build_conditional_branch(more, body, done);
        self.builder.position_at_end(body);
        let at = basic_result(self.builder.build_call(slot_of, &[src.into(), spec.into(), src_order.into(), i.into()], "at").ok()?)?.into_int_value();
        let kp = basic_result(self.builder.build_call(key_at, &[src.into(), spec.into(), at.into()], "k").ok()?)?.into_pointer_value();
        let vp = basic_result(self.builder.build_call(val_at, &[src.into(), spec.into(), at.into()], "v").ok()?)?.into_pointer_value();
        let kfresh = self.builder.build_alloca(klt, "kcopy").ok()?;
        let vfresh = self.builder.build_alloca(vlt, "vcopy").ok()?;
        match copy_k {
            Some(f) => {
                let _ = self.builder.build_call(f, &[kfresh.into(), kp.into()], "");
            }
            None => {
                let v = self.builder.build_load(klt, kp, "kv").ok()?;
                let _ = self.builder.build_store(kfresh, v);
            }
        }
        match copy_v {
            Some(f) => {
                let _ = self.builder.build_call(f, &[vfresh.into(), vp.into()], "");
            }
            None => {
                let v = self.builder.build_load(vlt, vp, "vv").ok()?;
                let _ = self.builder.build_store(vfresh, v);
            }
        }
        let _ = self.builder.build_call(set, &[dst.into(), dst_order.into(), spec.into(), kfresh.into(), vfresh.into()], "");
        let next = self.builder.build_int_add(i, w.const_int(1, false), "next").ok()?;
        let _ = self.builder.build_store(i_slot, next);
        let _ = self.builder.build_unconditional_branch(head);
        self.builder.position_at_end(done);
        Some(())
    }
}

impl<'a, 'ctx> Cg<'a, 'ctx> {
    /// `map.keys`, `map.values`, `map.pairs` — the entries laid out in the
    /// order they were put in (`maps.md`), as an array to walk. A
    /// `map.unordered` never gets here: the checker refuses to walk one,
    /// because choosing it *is* the promise not to.
    ///
    /// The array holds copies, not borrows: it owns what is in it, so
    /// giving it back at the end of the walk must not take the map's own
    /// keys with it.
    pub(super) fn map_walk(&mut self, fi: usize, op: &Op, args: &[Operand], out_ty: &Ty, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let place = args.first()?.place()?.clone();
        let (at, map_ty) = self.slot_of(fi, &place, slots)?;
        let (kt, vt) = Self::map_key_val(&map_ty)?;
        let shape = match self.ty(&map_ty)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        let out_shape = match self.ty(out_ty)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        let elem = Self::elem_of(out_ty)?;
        let elt = self.ty(&elem)?;
        let esize = self.size_of_ty(&elem)?;
        let spec = self.map_spec(&map_ty)?;
        let w = self.ctx.i64_type();
        let null = self.ctx.ptr_type(AddressSpace::default()).const_null();
        let count = self.load_field_int(shape, at, 1);
        let order = if self.map_is_ordered(&map_ty) {
            let p = self.builder.build_struct_gep(shape, at, 3, "op").ok()?;
            self.builder.build_load(self.ctx.ptr_type(AddressSpace::default()), p, "order").ok()?.into_pointer_value()
        } else {
            null
        };
        let bytes = self.builder.build_int_mul(count, w.const_int(esize, false), "bytes").ok()?;
        let data = self.alloc(bytes);
        let slot_of = self.rt_map("cyborg_map_slot");
        let key_at = self.rt_map("cyborg_map_key_at");
        let val_at = self.rt_map("cyborg_map_val_at");
        let i_slot = self.builder.build_alloca(w, "i").ok()?;
        let _ = self.builder.build_store(i_slot, w.const_zero());
        let head = self.ctx.append_basic_block(func, "walk");
        let body = self.ctx.append_basic_block(func, "walk_one");
        let done = self.ctx.append_basic_block(func, "walked");
        let _ = self.builder.build_unconditional_branch(head);
        self.builder.position_at_end(head);
        let i = self.builder.build_load(w, i_slot, "i").ok()?.into_int_value();
        let more = self.builder.build_int_compare(IntPredicate::ULT, i, count, "more").ok()?;
        let _ = self.builder.build_conditional_branch(more, body, done);
        self.builder.position_at_end(body);
        let which = basic_result(self.builder.build_call(slot_of, &[at.into(), spec.into(), order.into(), i.into()], "at").ok()?)?.into_int_value();
        let off = self.builder.build_int_mul(i, w.const_int(esize, false), "off").ok()?;
        let dst = unsafe { self.builder.build_gep(self.ctx.i8_type(), data, &[off], "into").ok()? };
        match op {
            Op::MapPairs => {
                let kp = basic_result(self.builder.build_call(key_at, &[at.into(), spec.into(), which.into()], "k").ok()?)?.into_pointer_value();
                let vp = basic_result(self.builder.build_call(val_at, &[at.into(), spec.into(), which.into()], "v").ok()?)?.into_pointer_value();
                // `pair` is `key` then `value`, in that order.
                let kdst = self.builder.build_struct_gep(elt.into_struct_type(), dst, 0, "pk").ok()?;
                let vdst = self.builder.build_struct_gep(elt.into_struct_type(), dst, 1, "pv").ok()?;
                self.copy_one(&kt, kdst, kp);
                self.copy_one(&vt, vdst, vp);
            }
            Op::MapValues => {
                let vp = basic_result(self.builder.build_call(val_at, &[at.into(), spec.into(), which.into()], "v").ok()?)?.into_pointer_value();
                self.copy_one(&vt, dst, vp);
            }
            _ => {
                let kp = basic_result(self.builder.build_call(key_at, &[at.into(), spec.into(), which.into()], "k").ok()?)?.into_pointer_value();
                self.copy_one(&kt, dst, kp);
            }
        }
        let next = self.builder.build_int_add(i, w.const_int(1, false), "next").ok()?;
        let _ = self.builder.build_store(i_slot, next);
        let _ = self.builder.build_unconditional_branch(head);
        self.builder.position_at_end(done);
        let mut agg = out_shape.get_undef();
        agg = self.builder.build_insert_value(agg, data, 0, "p").ok()?.into_struct_value();
        agg = self.builder.build_insert_value(agg, count, 1, "n").ok()?.into_struct_value();
        if out_shape.count_fields() > 2 {
            agg = self.builder.build_insert_value(agg, count, 2, "c").ok()?.into_struct_value();
        }
        Some(agg.into())
    }

    /// One value copied from `src` to `dst`: deeply when it owns
    /// something, plainly when it does not.
    fn copy_one(&mut self, t: &Ty, dst: PointerValue<'ctx>, src: PointerValue<'ctx>) {
        match self.copy_fn(t) {
            Some(f) => {
                let _ = self.builder.build_call(f, &[dst.into(), src.into()], "");
            }
            None => {
                if let Some(l) = self.ty(t) {
                    if let Ok(v) = self.builder.build_load(l, src, "v") {
                        let _ = self.builder.build_store(dst, v);
                    }
                }
            }
        }
    }
}
