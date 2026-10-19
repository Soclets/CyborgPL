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
            "cyborg_alloc" | "cyborg_alloc_zeroed" => p.fn_type(&[w.into()], false),
            "cyborg_par_for" => w.fn_type(&[p.into(), p.into(), w.into()], false),
            "cyborg_realloc" => p.fn_type(&[p.into(), w.into()], false),
            "cyborg_free" => self.ctx.void_type().fn_type(&[p.into()], false),
            "cyborg_machine_has" => self.ctx.bool_type().fn_type(&[self.ctx.i32_type().into()], false),
            "cyborg_copy" | "cyborg_move" => self.ctx.void_type().fn_type(&[p.into(), p.into(), w.into()], false),
            "cyborg_str_concat" => self.ctx.void_type().fn_type(&[p.into(), p.into(), w.into(), p.into(), w.into()], false),
            "cyborg_str_copy" => self.ctx.void_type().fn_type(&[p.into(), p.into(), w.into()], false),
            "cyborg_str_free" => self.ctx.void_type().fn_type(&[p.into(), w.into()], false),
            "cyborg_parse_int" => self.ctx.i8_type().fn_type(&[p.into(), w.into(), p.into()], false),
            "cyborg_int_to_text" => self.ctx.void_type().fn_type(&[p.into(), w.into(), w.into()], false),
            "cyborg_wide_to_text" => self.ctx.void_type().fn_type(&[p.into(), p.into(), w.into(), self.ctx.bool_type().into()], false),
            "cyborg_str_points" | "cyborg_str_graphemes" => w.fn_type(&[p.into(), w.into()], false),
            "cyborg_str_is_ascii" => self.ctx.i8_type().fn_type(&[p.into(), w.into()], false),
            "cyborg_bin_to_text_useful" => self.ctx.void_type().fn_type(&[p.into(), self.ctx.f64_type().into(), self.ctx.i32_type().into()], false),
            "cyborg_str_from_bytes" => self.ctx.i8_type().fn_type(&[p.into(), p.into(), w.into()], false),
            "cyborg_cstr_make" => self.ctx.i8_type().fn_type(&[p.into(), p.into(), w.into()], false),
            "cyborg_cstr_read" => self.ctx.i8_type().fn_type(&[p.into(), p.into()], false),
            "cyborg_str_decode_points" | "cyborg_str_split_graphemes" => w.fn_type(&[p.into(), w.into(), p.into()], false),
            "cyborg_parse_bin" => self.ctx.i8_type().fn_type(&[p.into(), w.into(), self.ctx.i32_type().into(), p.into()], false),
            "cyborg_char_to_text" => self.ctx.void_type().fn_type(&[p.into(), self.ctx.i32_type().into()], false),
            "cyborg_text_one_point" => w.fn_type(&[p.into(), w.into()], false),
            "cyborg_bin_to_text" => self.ctx.void_type().fn_type(&[p.into(), self.ctx.f64_type().into()], false),
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
            Ty::Str(_) | Ty::Char(crate::types::Unit::Grapheme) => true,
            Ty::Struct { id, args } => {
                let info = &self.p.items.structs[id.0 as usize];
                info.fields.iter().any(|f| self.owns_memory(&f.ty.subst(args)))
            }
            Ty::Bundle(ts) => ts.iter().any(|t| self.owns_memory(t)),
            Ty::Fn(f) => f.owning,
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
            Ty::Str(_) | Ty::Char(crate::types::Unit::Grapheme) => {
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
            // A function carrying what it captured: its block says how it is
            // given back (null when there is nothing to give).
            Ty::Fn(_) => {
                let p = self.ctx.ptr_type(AddressSpace::default());
                let st = self.ctx.struct_type(&[p.into(), p.into()], false);
                let Ok(envp) = self.builder.build_struct_gep(st, at, 1, "env") else { return };
                let Ok(env) = self.builder.build_load(p, envp, "env") else { return };
                let env = env.into_pointer_value();
                let head = self.ctx.struct_type(&[p.into(), p.into()], false);
                let has = self.ctx.append_basic_block(func, "has_env");
                let ask = self.ctx.append_basic_block(func, "ask");
                let done = self.ctx.append_basic_block(func, "given_back");
                let Ok(none) = self.builder.build_is_null(env, "none") else { return };
                let _ = self.builder.build_conditional_branch(none, done, has);
                self.builder.position_at_end(has);
                let Ok(dp) = self.builder.build_struct_gep(head, env, 0, "dropp") else { return };
                let Ok(d) = self.builder.build_load(p, dp, "drop") else { return };
                let d = d.into_pointer_value();
                let Ok(nod) = self.builder.build_is_null(d, "nodrop") else { return };
                let _ = self.builder.build_conditional_branch(nod, done, ask);
                self.builder.position_at_end(ask);
                let _ = self.builder.build_indirect_call(self.ctx.void_type().fn_type(&[p.into()], false), d, &[env.into()], "");
                let _ = self.builder.build_unconditional_branch(done);
                self.builder.position_at_end(done);
            }
            // A bundle is its values side by side: each gives back its own.
            Ty::Bundle(ts) => {
                let shape = match self.ty(t) {
                    Some(BasicTypeEnum::StructType(s)) => s,
                    _ => return,
                };
                for (i, et) in ts.iter().enumerate() {
                    if let Some(inner) = self.drop_fn(et) {
                        if let Ok(fp) = self.builder.build_struct_gep(shape, at, i as u32, "e") {
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
            Ty::Str(_) | Ty::Char(crate::types::Unit::Grapheme) => {
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
            // A copy of a function carrying what it captured: the same code,
            // and its block copied by the block's own way.
            Ty::Fn(_) => {
                let p = self.ctx.ptr_type(AddressSpace::default());
                let st = self.ctx.struct_type(&[p.into(), p.into()], false);
                let (Ok(scp), Ok(sep)) = (self.builder.build_struct_gep(st, src, 0, "code"), self.builder.build_struct_gep(st, src, 1, "env")) else { return };
                let (Ok(dcp), Ok(dep)) = (self.builder.build_struct_gep(st, dst, 0, "code"), self.builder.build_struct_gep(st, dst, 1, "env")) else { return };
                let (Ok(code), Ok(env)) = (self.builder.build_load(p, scp, "code"), self.builder.build_load(p, sep, "env")) else { return };
                let _ = self.builder.build_store(dcp, code);
                let _ = self.builder.build_store(dep, env);
                let env = env.into_pointer_value();
                let has = self.ctx.append_basic_block(func, "has_env");
                let ask = self.ctx.append_basic_block(func, "ask");
                let done = self.ctx.append_basic_block(func, "copied");
                let Ok(none) = self.builder.build_is_null(env, "none") else { return };
                let _ = self.builder.build_conditional_branch(none, done, has);
                self.builder.position_at_end(has);
                let Ok(cp) = self.builder.build_struct_gep(st, env, 1, "copyp") else { return };
                let Ok(c) = self.builder.build_load(p, cp, "copy") else { return };
                let c = c.into_pointer_value();
                let Ok(noc) = self.builder.build_is_null(c, "nocopy") else { return };
                let _ = self.builder.build_conditional_branch(noc, done, ask);
                self.builder.position_at_end(ask);
                if let Ok(r) = self.builder.build_indirect_call(p.fn_type(&[p.into()], false), c, &[env.into()], "copied") {
                    if let Some(v) = basic_result(r) {
                        let _ = self.builder.build_store(dep, v);
                    }
                }
                let _ = self.builder.build_unconditional_branch(done);
                self.builder.position_at_end(done);
            }
            Ty::Bundle(ts) => {
                let shape = match self.ty(t) {
                    Some(BasicTypeEnum::StructType(s)) => s,
                    _ => return,
                };
                for (i, et) in ts.clone().iter().enumerate() {
                    let (Ok(dp), Ok(sp)) = (self.builder.build_struct_gep(shape, dst, i as u32, "d"), self.builder.build_struct_gep(shape, src, i as u32, "s")) else { continue };
                    match self.copy_fn(et) {
                        Some(inner) => {
                            let _ = self.builder.build_call(inner, &[dp.into(), sp.into()], "");
                        }
                        None => {
                            if let Some(l) = self.ty(et) {
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

    /// The arguments of a call to a variadic function: the fixed ones as
    /// they are, then the bundle the call builds from the rest
    /// (`functions.md`). All of one type, the bundle is a list — its
    /// length may be known only at run time, when a bounded list is
    /// unpacked into it — and otherwise the values side by side. An owning
    /// bundle is the callee's to give back; a lent one is built here, handed
    /// over by address, and its buffer (not what is in it) given back after
    /// the call: the second list is the extra buffers to free.
    pub(super) fn variadic_args(&mut self, fi: usize, callee: usize, args: &[Arg], slots: &[Option<PointerValue<'ctx>>]) -> Option<(Vec<BasicMetadataValueEnum<'ctx>>, Vec<PointerValue<'ctx>>)> {
        let cf = &self.p.functions[callee];
        let fixed = cf.params.len().checked_sub(1)?;
        let bp = cf.params[fixed];
        let bty = cf.locals[bp.0 as usize].ty.clone();
        let lent = matches!(cf.locals[bp.0 as usize].kind, LocalKind::Lent { .. });
        let mut vals: Vec<BasicMetadataValueEnum> = Vec::new();
        for a in &args[..fixed.min(args.len())] {
            let v = match a {
                Arg::Value(o) => self.operand(fi, o, slots)?,
                Arg::Copy(p) => self.copied(fi, p, slots)?,
                Arg::Give(p) => self.operand(fi, &Operand::Move(p.clone()), slots)?,
                Arg::Lend { place, .. } => self.slot_of(fi, place, slots)?.0.into(),
                Arg::Spread { .. } => return None,
            };
            vals.push(v.into());
        }
        let rest = &args[fixed.min(args.len())..];
        let p = self.ctx.ptr_type(AddressSpace::default());
        let w = self.ctx.i64_type();
        let mut frees = Vec::new();
        // One value of the bundle, from a place or an operand, by its verb:
        // a copy is a second, independent value.
        let one = |this: &mut Self, a: &Arg| -> Option<(Option<PointerValue<'ctx>>, Option<BasicValueEnum<'ctx>>, bool)> {
            Some(match a {
                Arg::Value(o) => (None, Some(this.operand(fi, o, slots)?), false),
                Arg::Copy(pl) => (Some(this.slot_of(fi, pl, slots)?.0), None, true),
                Arg::Give(pl) | Arg::Lend { place: pl, .. } => (Some(this.slot_of(fi, pl, slots)?.0), None, false),
                Arg::Spread { .. } => return None,
            })
        };
        match bty.base() {
            Ty::Seq { .. } => {
                let elem = Self::elem_of(&bty)?;
                let lty = self.ty(&elem)?;
                let esize = self.size_of_ty(&elem)?;
                let cap = match bty.bound() {
                    Some(crate::types::Bound::Len { hi, .. }) => *hi,
                    _ => rest.len() as u64,
                };
                let data = if cap == 0 { p.const_null() } else { self.alloc(w.const_int(cap * esize, false)) };
                let cur = self.stack(w, "filled").ok()?;
                let _ = self.builder.build_store(cur, w.const_zero());
                let copier = self.copy_fn(&elem);
                for a in rest {
                    if let Arg::Spread { place, verb, .. } = a {
                        let verb = verb.filter(|v| matches!(v, crate::ast::Verb::Copy | crate::ast::Verb::Give));
                        let (src, sty) = self.slot_of(fi, place, slots)?;
                        // Copied across as they are: only a list of the same
                        // element layout.
                        let same = Self::elem_of(&sty).map_or(false, |se| self.ty(&se) == Some(lty));
                        if !same {
                            self.cannot("unpacking a list whose elements differ in width from the variadic's has no shape in the generated code yet", None);
                            return None;
                        }
                        let shape = match self.ty(&sty)? {
                            BasicTypeEnum::StructType(s) => s,
                            _ => return None,
                        };
                        let sdata = self.load_field_ptr(shape, src, 0);
                        let n = self.load_field_int(shape, src, 1);
                        let at = self.builder.build_load(w, cur, "at").ok()?.into_int_value();
                        let off = self.builder.build_int_mul(at, w.const_int(esize, false), "off").ok()?;
                        let dst = unsafe { self.builder.build_gep(self.ctx.i8_type(), data, &[off], "dst").ok()? };
                        let bytes = self.builder.build_int_mul(n, w.const_int(esize, false), "bytes").ok()?;
                        match (verb, copier) {
                            // Each element a second value of its own.
                            (Some(crate::ast::Verb::Copy), Some(c)) => {
                                let func = self.builder.get_insert_block()?.get_parent()?;
                                let head = self.ctx.append_basic_block(func, "copy_each");
                                let body = self.ctx.append_basic_block(func, "copy_one");
                                let done = self.ctx.append_basic_block(func, "copied_all");
                                let i = self.stack(w, "i").ok()?;
                                let _ = self.builder.build_store(i, w.const_zero());
                                let _ = self.builder.build_unconditional_branch(head);
                                self.builder.position_at_end(head);
                                let iv = self.builder.build_load(w, i, "iv").ok()?.into_int_value();
                                let more = self.builder.build_int_compare(IntPredicate::ULT, iv, n, "more").ok()?;
                                let _ = self.builder.build_conditional_branch(more, body, done);
                                self.builder.position_at_end(body);
                                let o = self.builder.build_int_mul(iv, w.const_int(esize, false), "o").ok()?;
                                let s = unsafe { self.builder.build_gep(self.ctx.i8_type(), sdata, &[o], "s").ok()? };
                                let d = unsafe { self.builder.build_gep(self.ctx.i8_type(), dst, &[o], "d").ok()? };
                                let _ = self.builder.build_call(c, &[d.into(), s.into()], "");
                                let next = self.builder.build_int_add(iv, w.const_int(1, false), "next").ok()?;
                                let _ = self.builder.build_store(i, next);
                                let _ = self.builder.build_unconditional_branch(head);
                                self.builder.position_at_end(done);
                            }
                            // The same bytes: a plain value, a lend, or a
                            // give (the list is left empty below).
                            _ => {
                                let f = self.rt("cyborg_copy");
                                let _ = self.builder.build_call(f, &[dst.into(), sdata.into(), bytes.into()], "");
                            }
                        }
                        if verb == Some(crate::ast::Verb::Give) {
                            // "A give never leaves holes": the list is left empty.
                            let lenp = self.builder.build_struct_gep(shape, src, 1, "len").ok()?;
                            let _ = self.builder.build_store(lenp, w.const_zero());
                        }
                        let after = self.builder.build_int_add(at, n, "after").ok()?;
                        let _ = self.builder.build_store(cur, after);
                        continue;
                    }
                    let (from, v, copy) = one(self, a)?;
                    let at = self.builder.build_load(w, cur, "at").ok()?.into_int_value();
                    let off = self.builder.build_int_mul(at, w.const_int(esize, false), "off").ok()?;
                    let dst = unsafe { self.builder.build_gep(self.ctx.i8_type(), data, &[off], "dst").ok()? };
                    match (from, v, copy, copier) {
                        (Some(src), _, true, Some(c)) => {
                            let _ = self.builder.build_call(c, &[dst.into(), src.into()], "");
                        }
                        (Some(src), _, _, _) => {
                            let v = self.builder.build_load(lty, src, "v").ok()?;
                            let _ = self.builder.build_store(dst, v);
                        }
                        (None, Some(v), _, _) => {
                            let v = self.fit(v, lty);
                            let _ = self.builder.build_store(dst, v);
                        }
                        _ => return None,
                    }
                    let next = self.builder.build_int_add(at, w.const_int(1, false), "next").ok()?;
                    let _ = self.builder.build_store(cur, next);
                }
                let shape = match self.ty(&bty)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let len = self.builder.build_load(w, cur, "len").ok()?;
                let mut agg = shape.get_undef();
                agg = self.builder.build_insert_value(agg, data, 0, "p").ok()?.into_struct_value();
                agg = self.builder.build_insert_value(agg, len, 1, "n").ok()?.into_struct_value();
                if shape.count_fields() > 2 {
                    agg = self.builder.build_insert_value(agg, w.const_int(cap, false), 2, "c").ok()?.into_struct_value();
                }
                if lent {
                    let tmp = self.stack(shape, "bundle").ok()?;
                    let _ = self.builder.build_store(tmp, agg);
                    vals.push(tmp.into());
                    if cap > 0 {
                        frees.push(data);
                    }
                } else {
                    vals.push(BasicValueEnum::from(agg).into());
                }
            }
            Ty::Bundle(ts) => {
                let ts = ts.clone();
                let shape = match self.ty(&bty)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let tmp = self.stack(shape, "bundle").ok()?;
                let mut k = 0usize;
                let put = |this: &mut Self, k: usize, src: Option<PointerValue<'ctx>>, v: Option<BasicValueEnum<'ctx>>, copy: bool| -> Option<()> {
                    let et = ts.get(k)?.clone();
                    let lty = this.ty(&et)?;
                    let d = this.builder.build_struct_gep(shape, tmp, k as u32, "e").ok()?;
                    match (src, v) {
                        (Some(s), _) => match (copy, this.copy_fn(&et)) {
                            (true, Some(c)) => {
                                let _ = this.builder.build_call(c, &[d.into(), s.into()], "");
                            }
                            _ => {
                                let x = this.builder.build_load(lty, s, "v").ok()?;
                                let _ = this.builder.build_store(d, x);
                            }
                        },
                        (None, Some(v)) => {
                            let v = this.fit(v, lty);
                            let _ = this.builder.build_store(d, v);
                        }
                        _ => return None,
                    }
                    Some(())
                };
                for a in rest {
                    if let Arg::Spread { place, count, verb } = a {
                        let copy = *verb == Some(crate::ast::Verb::Copy);
                        let give = *verb == Some(crate::ast::Verb::Give);
                        let (src, sty) = self.slot_of(fi, place, slots)?;
                        let sshape = match self.ty(&sty)? {
                            BasicTypeEnum::StructType(s) => s,
                            _ => return None,
                        };
                        match sty.base() {
                            // A bundle passed on: its values, field by field.
                            Ty::Bundle(st) => {
                                for j in 0..st.len() {
                                    let s = self.builder.build_struct_gep(sshape, src, j as u32, "s").ok()?;
                                    put(self, k, Some(s), None, copy)?;
                                    k += 1;
                                }
                            }
                            // A list whose length is known here: its elements.
                            _ => {
                                let elem = Self::elem_of(&sty)?;
                                let esize = self.size_of_ty(&elem)?;
                                let sdata = self.load_field_ptr(sshape, src, 0);
                                for j in 0..*count {
                                    let s = unsafe { self.builder.build_gep(self.ctx.i8_type(), sdata, &[w.const_int(j as u64 * esize, false)], "s").ok()? };
                                    put(self, k, Some(s), None, copy)?;
                                    k += 1;
                                }
                                if give {
                                    let lenp = self.builder.build_struct_gep(sshape, src, 1, "len").ok()?;
                                    let _ = self.builder.build_store(lenp, w.const_zero());
                                }
                            }
                        }
                        continue;
                    }
                    let (from, v, copy) = one(self, a)?;
                    put(self, k, from, v, copy)?;
                    k += 1;
                }
                if lent {
                    vals.push(tmp.into());
                } else {
                    let v = self.builder.build_load(shape, tmp, "bundle").ok()?;
                    vals.push(v.into());
                }
            }
            _ => return None,
        }
        Some((vals, frees))
    }

    /// `'xs'[i]` — 1-based (`lists.md`), so the machine's offset is
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
        // Checked to be 1 to the length at its site, so it fits a word
        // whatever width it came at — an exact answer is 65 bits or more.
        let idx = self.builder.build_int_cast_sign_flag(idx, w, false, "i").ok()?;
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

    /// `list.slice[lend 'xs', a, b]` — a new list owning copies of
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
        // A field is dropped only as what is left of a value some of whose
        // fields were given away (`mir::ownership`); the value's own flag
        // says whether it is here, and stays as it is for the next field.
        let field = !place.proj.is_empty();
        if place.proj.iter().any(|p| !matches!(p, Proj::Field(_))) {
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
                if !field {
                    let _ = self.builder.build_store(f, self.ctx.bool_type().const_zero());
                }
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

    pub(super) fn load_field_ptr(&mut self, shape: inkwell::types::StructType<'ctx>, at: PointerValue<'ctx>, i: u32) -> PointerValue<'ctx> {
        let p = self.builder.build_struct_gep(shape, at, i, "fp").unwrap();
        self.builder.build_load(self.ctx.ptr_type(AddressSpace::default()), p, "ptr").unwrap().into_pointer_value()
    }

    pub(super) fn load_field_int(&mut self, shape: inkwell::types::StructType<'ctx>, at: PointerValue<'ctx>, i: u32) -> IntValue<'ctx> {
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
        let i_slot = self.stack(w, "i").unwrap();
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
        if info.numbers.is_some() {
            // C's enum: its number, 32 bits, and nothing beside it.
            return Some(self.ctx.struct_type(&[self.ctx.i32_type().into()], false));
        }
        let payloads: Vec<Option<Ty>> = info.cases.iter().map(|c| c.payload.as_ref().map(|t| t.subst(args))).collect();
        self.variant_shape(&payloads)
    }

    /// An enum's tag: which case, as a number. A `layout.C enum`'s tag is
    /// the case's own number, so C reads it as it wrote it; any other's is
    /// the case's place in the list.
    pub(super) fn tag_type_of(&self, id: EnumId) -> inkwell::types::IntType<'ctx> {
        let info = &self.p.items.enums[id.0 as usize];
        if info.numbers.is_some() {
            self.ctx.i32_type()
        } else {
            self.enum_tag_type(info.cases.len())
        }
    }

    pub(super) fn tag_of(&self, id: EnumId, case: usize) -> IntValue<'ctx> {
        let t = self.tag_type_of(id);
        match &self.p.items.enums[id.0 as usize].numbers {
            Some(ns) => t.const_int(ns[case] as u64, true),
            None => t.const_int(case as u64, false),
        }
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
        let tmp = self.stack(shape, "case").ok()?;
        let info = self.p.items.enums[id.0 as usize].clone();
        let _ = info;
        self.store_field(shape, tmp, 0, self.tag_of(id, case).into());
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
        let tag_ty = self.tag_type_of(id);
        let p = self.builder.build_struct_gep(shape, at, 0, "tag").ok()?;
        let tag = self.builder.build_load(tag_ty, p, "which").ok()?.into_int_value();
        let unreached = self.ctx.append_basic_block(func, "no_such_case");
        let arms: Vec<(IntValue<'ctx>, LlBlock<'ctx>)> = cases.iter().map(|(c, b)| (self.tag_of(id, *c), blocks[b.0 as usize])).collect();
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

    /// The variant a function of this `fn[…]` type hands back: the same
    /// shape its own body's `fail_shape` gives.
    pub(super) fn fn_ty_fail_shape(&mut self, ft: &crate::types::FnTy) -> Option<inkwell::types::StructType<'ctx>> {
        let mut payloads: Vec<Option<Ty>> = Vec::with_capacity(ft.fails.len() + 1);
        payloads.push(if matches!(ft.ret, Ty::Nothing) { None } else { Some(ft.ret.clone()) });
        for (_, t) in &ft.fails {
            payloads.push(t.clone());
        }
        self.variant_shape(&payloads)
    }

    /// Case 0 is the answer; reason `i` is case `i + 1`.
    fn variant_value(&mut self, shape: inkwell::types::StructType<'ctx>, case: usize, cases: usize, payload: Option<BasicValueEnum<'ctx>>) -> Option<BasicValueEnum<'ctx>> {
        let tmp = self.stack(shape, "outcome").ok()?;
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
        let e = self.p.items.externs[id.0 as usize].clone();
        let ret = e.ret.clone();
        let v = self.call(fi, callee, args, slots);
        if ret != Ty::Nothing && v.is_none() {
            return None;
        }
        let mut kept = match v {
            Some(v) => self.kept_at_door(v, &ret)?,
            None => self.ctx.bool_type().const_all_ones(),
        };
        // What C wrote through a `ptr.mut` lent to it is checked when the
        // call ends (owner, 2026-09-27): every element of the lent list.
        for ((_, pt), a) in e.params.iter().zip(args) {
            let Ty::Ptr { mutable: true, elem } = pt.base() else { continue };
            if !elem.checked_at_door(&self.p.items.structs, &self.p.items.enums) {
                continue;
            }
            let Arg::Lend { place, .. } = a else { continue };
            let k = self.kept_in_list(fi, place, elem, slots, func)?;
            kept = self.builder.build_and(kept, k, "kept").ok()?;
        }
        let outside = self.builder.build_not(kept, "out_of_range").ok()?;
        let inside = self.ctx.append_basic_block(func, "in_range");
        // `out_of_range` is the one reason; an arm for it, or none when the
        // caller's `otherwise` gives a value or leaves the same way for all.
        let out = fail.first().map(|a| blocks[a.block.0 as usize])?;
        let _ = self.builder.build_conditional_branch(outside, out, inside);
        self.builder.position_at_end(inside);
        if let (Some(d), Some(v)) = (dest, v) {
            self.store_place(fi, d, v.into(), slots);
        }
        let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
        Some(())
    }

    /// Whether every element of a list C may have written keeps its type's
    /// promise.
    fn kept_in_list(&mut self, fi: usize, place: &Place, elem: &Ty, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<IntValue<'ctx>> {
        let (at, ty) = self.slot_of(fi, place, slots)?;
        let shape = self.ty(&ty)?.into_struct_type();
        let et = self.ty(elem)?;
        let w = self.ctx.i64_type();
        let data = self.load_field_ptr(shape, at, 0);
        let lp = self.builder.build_struct_gep(shape, at, 1, "len").ok()?;
        let n = self.builder.build_load(w, lp, "n").ok()?.into_int_value();
        let flag = self.stack(self.ctx.bool_type(), "kept").ok()?;
        let _ = self.builder.build_store(flag, self.ctx.bool_type().const_all_ones());
        let iv = self.stack(w, "i").ok()?;
        let _ = self.builder.build_store(iv, w.const_zero());
        let head = self.ctx.append_basic_block(func, "door_head");
        let body = self.ctx.append_basic_block(func, "door_body");
        let done = self.ctx.append_basic_block(func, "door_done");
        let _ = self.builder.build_unconditional_branch(head);
        self.builder.position_at_end(head);
        let i = self.builder.build_load(w, iv, "i").ok()?.into_int_value();
        let more = self.builder.build_int_compare(IntPredicate::ULT, i, n, "more").ok()?;
        let _ = self.builder.build_conditional_branch(more, body, done);
        self.builder.position_at_end(body);
        let slot = unsafe { self.builder.build_gep(et, data, &[i], "elem").ok()? };
        let x = self.builder.build_load(et, slot, "x").ok()?;
        let k = self.kept_at_door(x, elem)?;
        let was = self.builder.build_load(self.ctx.bool_type(), flag, "was").ok()?.into_int_value();
        let now = self.builder.build_and(was, k, "now").ok()?;
        let _ = self.builder.build_store(flag, now);
        let next = self.builder.build_int_add(i, w.const_int(1, false), "next").ok()?;
        let _ = self.builder.build_store(iv, next);
        let _ = self.builder.build_unconditional_branch(head);
        self.builder.position_at_end(done);
        self.builder.build_load(self.ctx.bool_type(), flag, "kept").ok().map(|v| v.into_int_value())
    }

    /// A read from C's memory that comes in checked (owner, 2026-09-27): a
    /// C enum or range fails `out_of_range` when C's number breaks it.
    #[allow(clippy::too_many_arguments)]
    fn emit_checked_ptr_read(&mut self, fi: usize, args: &[Arg], dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let o = match args.first()? {
            Arg::Value(o) => o.clone(),
            _ => return None,
        };
        let p = self.operand(fi, &o, slots)?.into_pointer_value();
        let elem = self.pointee(fi, &o)?;
        let t = self.ty(&elem)?;
        let v = self.builder.build_load(t, p, "read").ok()?;
        let kept = self.kept_at_door(v, &elem)?;
        let inside = self.ctx.append_basic_block(func, "kept");
        let out = fail.first().map(|a| blocks[a.block.0 as usize])?;
        let _ = self.builder.build_conditional_branch(kept, inside, out);
        self.builder.position_at_end(inside);
        if let Some(d) = dest {
            self.store_place(fi, d, v, slots);
        }
        let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
        Some(())
    }

    /// Whether what C handed back keeps Cyborg's promises: a range holds its
    /// number, a C enum's number is one of its cases, and a `layout.C`
    /// struct's fields keep theirs (owner, 2026-09-27: a).
    fn kept_at_door(&mut self, v: BasicValueEnum<'ctx>, ty: &Ty) -> Option<IntValue<'ctx>> {
        let mut kept = self.ctx.bool_type().const_all_ones();
        if let Some(b) = ty.bound() {
            let crate::types::Bound::Int { lo, hi, .. } = b.clone() else { return None };
            let signed = matches!(ty.base(), Ty::Int(crate::types::IntW::Fixed { signed: true, .. }));
            let v = v.into_int_value();
            let t = v.get_type();
            let (lp, hp) = if signed { (IntPredicate::SGE, IntPredicate::SLE) } else { (IntPredicate::UGE, IntPredicate::ULE) };
            let from = self.builder.build_int_compare(lp, v, super::const_i128(t, lo), "from").ok()?;
            let to = self.builder.build_int_compare(hp, v, super::const_i128(t, hi), "to").ok()?;
            return self.builder.build_and(from, to, "in_range").ok();
        }
        match ty.base() {
            Ty::Enum { id, .. } => {
                let Some(ns) = self.p.items.enums[id.0 as usize].numbers.clone() else { return Some(kept) };
                let n = self.builder.build_extract_value(v.into_struct_value(), 0, "number").ok()?.into_int_value();
                let mut any = self.ctx.bool_type().const_zero();
                for x in ns {
                    let is = self.builder.build_int_compare(IntPredicate::EQ, n, n.get_type().const_int(x as u64, true), "is_case").ok()?;
                    any = self.builder.build_or(any, is, "a_case").ok()?;
                }
                Some(any)
            }
            Ty::Struct { id, args } => {
                let info = self.p.items.structs[id.0 as usize].clone();
                // A union's fields are read on your word, and a bit-field
                // holds only what its bits hold: nothing to check at the door.
                if !info.layout_c || info.union || info.fields.iter().any(|f| f.bits.is_some()) {
                    return Some(kept);
                }
                let sv = v.into_struct_value();
                for (i, f) in info.fields.iter().enumerate() {
                    let ft = f.ty.subst(args);
                    if ft.bound().is_none() && !matches!(ft.base(), Ty::Enum { .. } | Ty::Struct { .. }) {
                        continue;
                    }
                    let fv = self.builder.build_extract_value(sv, i as u32, "field").ok()?;
                    let k = self.kept_at_door(fv, &ft)?;
                    kept = self.builder.build_and(kept, k, "kept").ok()?;
                }
                Some(kept)
            }
            _ => Some(kept),
        }
    }

    /// A conversion that can fail (`conversion.md`). Its reasons are the
    /// checker's, in its order: `not_a_number` (from text) or `not_whole`
    /// (from a bin) first, then `out_of_range` for a range type. Into the
    /// exact type (a plain whole-number target, whose landing is its own
    /// site) a value past the 64-bit window becomes one no 64-bit type holds,
    /// so that site always catches it. Text is never a site: a number it
    /// holds that the type cannot is `out_of_range`, however many digits
    /// (owner, 2026-09-24: c).
    #[allow(clippy::too_many_arguments)]
    fn emit_failing_convert(&mut self, fi: usize, to: &Ty, written: bool, args: &[Arg], dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let arg = args.first()?;
        let from = match arg {
            Arg::Value(o) => o.place().map(|p| {
                let base = self.p.functions[fi].locals[p.local.0 as usize].ty.clone();
                crate::mir::build::projected_ty(&base, &p.proj, &self.p.items)
            }),
            Arg::Lend { place, .. } | Arg::Copy(place) | Arg::Give(place) => {
                let base = self.p.functions[fi].locals[place.local.0 as usize].ty.clone();
                Some(crate::mir::build::projected_ty(&base, &place.proj, &self.p.items))
            }
            _ => None,
        };
        let from = match (from, arg) {
            (Some(t), _) => t,
            (None, Arg::Value(Operand::Const(Const::Str(_)))) => Ty::utf8(),
            (None, Arg::Value(Operand::Const(Const::Bin(_)))) => Ty::Bin(crate::types::BinW::Fixed(64)),
            (None, Arg::Value(Operand::Const(Const::Deci { bits, .. }))) => Ty::Deci(*bits),
            (None, Arg::Value(Operand::Const(Const::Int(_)))) => Ty::exact(),
            (None, Arg::Value(Operand::Const(Const::Bool(_)))) => Ty::Bool,
            _ => return None,
        };
        // Bytes read as UTF-8 (owner, 2026-09-30): a new text, or
        // `not_utf8` with nothing made.
        if let (Ty::Seq { .. }, Ty::Str(_), None) = (from.base(), to.base(), to.bound()) {
            let place = match arg {
                Arg::Value(Operand::Read(p) | Operand::Move(p)) | Arg::Copy(p) | Arg::Give(p) | Arg::Lend { place: p, .. } => p.clone(),
                _ => return None,
            };
            let (slot, ty) = self.slot_of(fi, &place, slots)?;
            let st = match self.ty(&ty)? {
                BasicTypeEnum::StructType(s) => s,
                _ => return None,
            };
            let data = self.load_field_ptr(st, slot, 0);
            let n = self.load_field_int(st, slot, 1);
            let text_ty = self.ty(to)?;
            let out = self.stack(text_ty, "text").ok()?;
            let f = self.rt("cyborg_str_from_bytes");
            let status = basic_result(self.builder.build_call(f, &[out.into(), data.into(), n.into()], "status").ok()?)?.into_int_value();
            let bad = self.builder.build_int_compare(IntPredicate::NE, status, self.ctx.i8_type().const_zero(), "not_utf8").ok()?;
            let good = self.ctx.append_basic_block(func, "utf8");
            let arm = fail.iter().find(|a| a.index == 0).or_else(|| fail.first()).map(|a| blocks[a.block.0 as usize])?;
            let _ = self.builder.build_conditional_branch(bad, arm, good);
            self.builder.position_at_end(good);
            if let Some(d) = dest {
                let t = self.builder.build_load(text_ty, out, "t").ok()?;
                self.store_place(fi, d, t, slots);
                // The new text is held now, and freed at the end like any
                // (examples/bytes_to_text leaked it).
                if d.proj.is_empty() {
                    self.mark_alive(d.local, true);
                }
            }
            let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
            return Some(());
        }
        if let Ty::Str(_) = to.base() {
            return self.emit_bounded_text(fi, &from, to, arg, dest, ok, fail, blocks, slots, func);
        }
        if let Ty::Char(unit) = to.base() {
            return self.into_char(fi, &from, *unit, arg, dest, ok, fail, blocks, slots, func);
        }
        if let (Ty::Bin(crate::types::BinW::Fixed(bits)), Ty::Str(_)) = (to.base(), from.base()) {
            return self.text_into_bin(fi, *bits, to, arg, dest, ok, fail, blocks, slots, func);
        }
        if let (Ty::Brain, Ty::Str(_)) = (to.base(), from.base()) {
            return self.text_into_bin(fi, crate::types::BRAIN_BITS, to, arg, dest, ok, fail, blocks, slots, func);
        }
        if let Ty::Deci(_) = to.base() {
            return self.into_deci(fi, to, written, &from, arg, dest, ok, fail, blocks, slots, func);
        }
        // A number into a C enum (owner, 2026-09-27): one of its cases'
        // numbers, or `out_of_range`.
        if let (Ty::Enum { id, .. }, Ty::Int(_)) = (to.base(), from.base()) {
            let numbers = self.p.items.enums[id.0 as usize].numbers.clone()?;
            let signed = match arg {
                Arg::Value(o) => self.operand_signed(fi, o).unwrap_or(true),
                _ => true,
            };
            let v = match arg {
                Arg::Value(o) => self.operand(fi, o, slots)?,
                Arg::Copy(p) | Arg::Give(p) | Arg::Lend { place: p, .. } => self.operand(fi, &Operand::Read(p.clone()), slots)?,
                _ => return None,
            };
            let w = self.ctx.i128_type();
            let value = self.builder.build_int_cast_sign_flag(v.into_int_value(), w, signed, "value").ok()?;
            let mut any = self.ctx.bool_type().const_zero();
            for n in numbers {
                let is = self.builder.build_int_compare(IntPredicate::EQ, value, super::const_i128(w, n as i128), "is_case").ok()?;
                any = self.builder.build_or(any, is, "a_case").ok()?;
            }
            let out = fail.first().map(|a| blocks[a.block.0 as usize])?;
            let inside = self.ctx.append_basic_block(func, "a_case");
            let _ = self.builder.build_conditional_branch(any, inside, out);
            self.builder.position_at_end(inside);
            let shape = self.ty(to)?.into_struct_type();
            let n32 = self.builder.build_int_truncate(value, self.ctx.i32_type(), "number").ok()?;
            let e = self.builder.build_insert_value(shape.get_undef(), n32, 0, "key").ok()?.into_struct_value();
            if let Some(d) = dest {
                self.store_place(fi, d, e.into(), slots);
            }
            let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
            return Some(());
        }
        let Ty::Int(crate::types::IntW::Fixed { .. }) = to.base() else { return None };
        let w = self.ctx.i128_type();
        // Which arm each reason takes: its index in the reason list.
        let first_reason = match from.base() {
            Ty::Str(_) | Ty::Bin(_) | Ty::Deci(_) | Ty::Brain => true,
            _ => false,
        };
        let arm = |index: usize| fail.iter().find(|a| a.index == index).or_else(|| fail.first()).map(|a| blocks[a.block.0 as usize]);
        let range_index = if first_reason { 1 } else { 0 };
        let from_text = matches!(from.base(), Ty::Str(_));
        let v = match arg {
            Arg::Value(o) => self.operand(fi, o, slots)?,
            Arg::Copy(p) | Arg::Give(p) | Arg::Lend { place: p, .. } => self.operand(fi, &Operand::Read(p.clone()), slots)?,
            _ => return None,
        };
        // The true value, in 128 bits.
        let (value, too_big): (IntValue<'ctx>, Option<IntValue<'ctx>>) = match from.base() {
            Ty::Str(_) => {
                let s = v.into_struct_value();
                let ptr = self.builder.build_extract_value(s, 0, "p").ok()?;
                let raw = self.builder.build_extract_value(s, 1, "n").ok()?.into_int_value();
                let len = self.real_length(raw);
                let out = self.stack(w, "parsed").ok()?;
                let f = self.rt("cyborg_parse_int");
                let status = basic_result(self.builder.build_call(f, &[ptr.into(), len.into(), out.into()], "status").ok()?)?.into_int_value();
                // A number too big even for 128 bits is `out_of_range` like
                // any other that does not fit (owner, 2026-09-24: c).
                let bad = self.builder.build_int_compare(IntPredicate::EQ, status, self.ctx.i8_type().const_int(1, false), "not_a_number").ok()?;
                let next = self.ctx.append_basic_block(func, "is_number");
                let _ = self.builder.build_conditional_branch(bad, arm(0)?, next);
                self.builder.position_at_end(next);
                let too = self.builder.build_int_compare(IntPredicate::EQ, status, self.ctx.i8_type().const_int(2, false), "too_big").ok()?;
                (self.builder.build_load(w, out, "value").ok()?.into_int_value(), Some(too))
            }
            Ty::Bin(_) | Ty::Brain => {
                // Whole, or it fails `not_whole`: x equals its own floor and
                // is finite (x - x is 0 only for a finite x). A brain16 is
                // asked at 64 bits, where it is exact.
                let x = v.into_float_value();
                let x = if from.is_brain() { self.builder.build_float_ext(x, self.ctx.f64_type(), "wide").ok()? } else { x };
                let ft = x.get_type();
                let id = inkwell::intrinsics::Intrinsic::find("llvm.floor")?;
                let floor = id.get_declaration(self.module, &[ft.into()])?;
                let fl = basic_result(self.builder.build_call(floor, &[x.into()], "floor").ok()?)?.into_float_value();
                let whole = self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, fl, x, "whole").ok()?;
                let zero = self.builder.build_float_sub(x, x, "zero").ok()?;
                let finite = self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, zero, ft.const_zero(), "finite").ok()?;
                let good = self.builder.build_and(whole, finite, "is_whole").ok()?;
                let next = self.ctx.append_basic_block(func, "is_whole");
                let _ = self.builder.build_conditional_branch(good, next, arm(0)?);
                self.builder.position_at_end(next);
                // Past 2^126 either way is past every 64-bit type too.
                let lim = ft.const_float(2f64.powi(126));
                let big = self.builder.build_float_compare(inkwell::FloatPredicate::OGE, x, lim, "big").ok()?;
                let neg_lim = ft.const_float(-(2f64.powi(126)));
                let small = self.builder.build_float_compare(inkwell::FloatPredicate::OLE, x, neg_lim, "small").ok()?;
                let too = self.builder.build_or(big, small, "too_big").ok()?;
                let safe = self.builder.build_select(too, ft.const_zero(), x, "safe").ok()?.into_float_value();
                (self.builder.build_float_to_signed_int(safe, w, "value").ok()?, Some(too))
            }
            Ty::Int(_) => {
                let signed = match arg {
                    Arg::Value(o) => self.operand_signed(fi, o).unwrap_or(true),
                    _ => true,
                };
                (self.builder.build_int_cast_sign_flag(v.into_int_value(), w, signed, "value").ok()?, None)
            }
            Ty::Deci(bits) => {
                // Whole, or it fails `not_whole` (owner, 2026-09-29); past
                // 128 bits is past every integer type.
                let (lo, hi) = self.deci_halves(v.into_int_value())?;
                let out = self.stack(w, "whole").ok()?;
                let f = self.rt_deci("cyborg_deci_to_int")?;
                let status = basic_result(self.builder.build_call(f, &[lo.into(), hi.into(), self.ctx.i32_type().const_int(*bits as u64, false).into(), out.into()], "status").ok()?)?.into_int_value();
                let i8t = self.ctx.i8_type();
                let frac = self.builder.build_int_compare(IntPredicate::EQ, status, i8t.const_int(1, false), "not_whole").ok()?;
                let next = self.ctx.append_basic_block(func, "is_whole");
                let _ = self.builder.build_conditional_branch(frac, arm(0)?, next);
                self.builder.position_at_end(next);
                let too = self.builder.build_int_compare(IntPredicate::EQ, status, i8t.const_int(2, false), "too_big").ok()?;
                (self.builder.build_load(w, out, "value").ok()?.into_int_value(), Some(too))
            }
            _ => return None,
        };
        let out_t = self.ty(to)?.into_int_type();
        let result = if to.is_exact() && !from_text {
            // Outside the window (or too big to have been read): a value
            // no 64-bit type holds, for the landing site to stop.
            let (lo, hi) = crate::resolve::int_range(true, crate::types::EXACT_BITS);
            let below = self.builder.build_int_compare(IntPredicate::SLT, value, super::const_i128(w, lo), "below").ok()?;
            let above = self.builder.build_int_compare(IntPredicate::SGT, value, super::const_i128(w, hi), "above").ok()?;
            let mut out = self.builder.build_or(below, above, "outside").ok()?;
            if let Some(t) = too_big {
                out = self.builder.build_or(out, t, "outside").ok()?;
            }
            let marker = super::const_i128(w, i64::MIN as i128 - 1);
            let v = self.builder.build_select(out, marker, value, "landed").ok()?.into_int_value();
            self.builder.build_int_truncate(v, out_t, "exact").ok()?
        } else {
            // A range type (or a plain one reached from a whole number):
            // outside it, the call fails `out_of_range`.
            let (lo, hi) = match to.bound() {
                Some(crate::types::Bound::Int { lo, hi, .. }) => (*lo, *hi),
                _ => match to.base() {
                    Ty::Int(crate::types::IntW::Fixed { signed, bits }) => crate::resolve::int_range(*signed, *bits),
                    _ => return None,
                },
            };
            let below = self.builder.build_int_compare(IntPredicate::SLT, value, super::const_i128(w, lo), "below").ok()?;
            let above = self.builder.build_int_compare(IntPredicate::SGT, value, super::const_i128(w, hi), "above").ok()?;
            let mut out = self.builder.build_or(below, above, "out_of_range").ok()?;
            if let Some(t) = too_big {
                out = self.builder.build_or(out, t, "out_of_range").ok()?;
            }
            let inside = self.ctx.append_basic_block(func, "in_range");
            let _ = self.builder.build_conditional_branch(out, arm(range_index)?, inside);
            self.builder.position_at_end(inside);
            self.builder.build_int_truncate(value, out_t, "value").ok()?
        };
        if let Some(d) = dest {
            self.store_place(fi, d, result.into(), slots);
        }
        let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
        Some(())
    }

    /// `char.point.from_code[…]` / `char.byte.from_code[…]`: the character
    /// with that code, failing `not_a_char` for one that is not — a point
    /// is a Unicode scalar (0 to 0x10FFFF, not a surrogate), a byte 0 to
    /// 255, as the oracle has them.
    #[allow(clippy::too_many_arguments)]
    fn char_from_code(&mut self, fi: usize, unit: crate::types::Unit, args: &[Arg], dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let o = match args.first()? {
            Arg::Value(o) => o.clone(),
            Arg::Copy(p) | Arg::Give(p) | Arg::Lend { place: p, .. } => Operand::Read(p.clone()),
            _ => return None,
        };
        let v = self.operand(fi, &o, slots)?.into_int_value();
        let signed = match &o {
            Operand::Const(Const::Int(c)) => *c < 0 || v.get_type().get_bit_width() > 64,
            _ => self.operand_signed(fi, &o).unwrap_or(true),
        };
        let w = self.ctx.i128_type();
        let x = self.builder.build_int_cast_sign_flag(v, w, signed, "code").ok()?;
        let c = |n: u64| w.const_int(n, false);
        let lt = |this: &mut Self, a, b, name| this.builder.build_int_compare(IntPredicate::SLT, a, b, name).ok();
        let gt = |this: &mut Self, a, b, name| this.builder.build_int_compare(IntPredicate::SGT, a, b, name).ok();
        let below = lt(self, x, c(0), "below")?;
        let bad = match unit {
            crate::types::Unit::Byte => {
                let above = gt(self, x, c(255), "above")?;
                self.builder.build_or(below, above, "not_a_char").ok()?
            }
            _ => {
                let above = gt(self, x, c(0x10FFFF), "above")?;
                let lo = self.builder.build_int_compare(IntPredicate::SGE, x, c(0xD800), "surrogate_lo").ok()?;
                let hi = self.builder.build_int_compare(IntPredicate::SLE, x, c(0xDFFF), "surrogate_hi").ok()?;
                let surrogate = self.builder.build_and(lo, hi, "surrogate").ok()?;
                let out = self.builder.build_or(below, above, "outside").ok()?;
                self.builder.build_or(out, surrogate, "not_a_char").ok()?
            }
        };
        let arm = fail.iter().find(|a| a.index == 0).or_else(|| fail.first()).map(|a| blocks[a.block.0 as usize])?;
        let good = self.ctx.append_basic_block(func, "is_a_char");
        let _ = self.builder.build_conditional_branch(bad, arm, good);
        self.builder.position_at_end(good);
        let t = if unit == crate::types::Unit::Byte { self.ctx.i8_type() } else { self.ctx.i32_type() };
        let ch = self.builder.build_int_truncate(x, t, "char").ok()?;
        if let Some(d) = dest {
            self.store_place(fi, d, ch.into(), slots);
        }
        let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
        Some(())
    }

    /// Into a character, where that can fail (`conversion.md`): text of
    /// exactly one point (`not_one_char`), text of exactly one byte, or a
    /// point whose UTF-8 is one byte — below 128 (`not_one_byte`: `é` is
    /// two). A grapheme waits with the rest of `char`.
    #[allow(clippy::too_many_arguments)]
    fn into_char(&mut self, fi: usize, from: &Ty, unit: crate::types::Unit, arg: &Arg, dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        use crate::types::Unit;
        let o = match arg {
            Arg::Value(o) => o.clone(),
            Arg::Copy(p) | Arg::Give(p) | Arg::Lend { place: p, .. } => Operand::Read(p.clone()),
            _ => return None,
        };
        let v = self.operand(fi, &o, slots)?;
        let i64t = self.ctx.i64_type();
        // Into a grapheme: text that is one grapheme cluster, copied.
        if unit == Unit::Grapheme {
            let Ty::Str(_) = from.base() else {
                self.cannot("a conversion into a `char.grapheme` from this has no shape in the generated code yet", None);
                return None;
            };
            let (p, n) = self.text_parts(v)?;
            let f = self.rt("cyborg_str_graphemes");
            let c = basic_result(self.builder.build_call(f, &[p.into(), n.into()], "count").ok()?)?.into_int_value();
            let bad = self.builder.build_int_compare(IntPredicate::NE, c, i64t.const_int(1, false), "not_one_char").ok()?;
            let arm = fail.iter().find(|a| a.index == 0).or_else(|| fail.first()).map(|a| blocks[a.block.0 as usize])?;
            let good = self.ctx.append_basic_block(func, "one_grapheme");
            let _ = self.builder.build_conditional_branch(bad, arm, good);
            self.builder.position_at_end(good);
            let copied = self.to_text(fi, &o, v, &Ty::Char(Unit::Grapheme))?;
            if let Some(d) = dest {
                self.store_place(fi, d, copied, slots);
                if d.proj.is_empty() {
                    self.mark_alive(d.local, true);
                }
            }
            let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
            return Some(());
        }
        let (code, bad) = match (from.base(), unit) {
            (Ty::Str(_) | Ty::Char(Unit::Grapheme), Unit::Point | Unit::Free(_)) => {
                let (p, n) = self.text_parts(v)?;
                let f = self.rt("cyborg_text_one_point");
                let code = basic_result(self.builder.build_call(f, &[p.into(), n.into()], "point").ok()?)?.into_int_value();
                let bad = self.builder.build_int_compare(IntPredicate::SLT, code, i64t.const_zero(), "not_one_char").ok()?;
                (code, bad)
            }
            (Ty::Str(_) | Ty::Char(Unit::Grapheme), Unit::Byte) => {
                // One byte of UTF-8 text is an ASCII one. The byte is read
                // only once there is one to read.
                let (p, raw) = self.text_parts(v)?;
                let n = self.real_length(raw);
                let bad = self.builder.build_int_compare(IntPredicate::NE, n, i64t.const_int(1, false), "not_one_char").ok()?;
                let arm = fail.iter().find(|a| a.index == 0).or_else(|| fail.first()).map(|a| blocks[a.block.0 as usize])?;
                let one = self.ctx.append_basic_block(func, "one_byte");
                let _ = self.builder.build_conditional_branch(bad, arm, one);
                self.builder.position_at_end(one);
                let b = self.builder.build_load(self.ctx.i8_type(), p, "byte").ok()?.into_int_value();
                (self.builder.build_int_z_extend(b, i64t, "code").ok()?, self.ctx.bool_type().const_zero())
            }
            (Ty::Char(_), Unit::Byte) => {
                let c = self.builder.build_int_z_extend_or_bit_cast(v.into_int_value(), i64t, "code").ok()?;
                let bad = self.builder.build_int_compare(IntPredicate::UGE, c, i64t.const_int(128, false), "not_one_byte").ok()?;
                (c, bad)
            }
            _ => {
                self.cannot(format!("a conversion into a `{}` has no shape in the generated code yet", crate::types::TySpelling { ty: &Ty::Char(unit), structs: &self.p.items.structs, enums: &self.p.items.enums }), None);
                return None;
            }
        };
        let arm = fail.iter().find(|a| a.index == 0).or_else(|| fail.first()).map(|a| blocks[a.block.0 as usize])?;
        let good = self.ctx.append_basic_block(func, "one_char");
        let _ = self.builder.build_conditional_branch(bad, arm, good);
        self.builder.position_at_end(good);
        let t = if unit == Unit::Byte { self.ctx.i8_type() } else { self.ctx.i32_type() };
        let ch = self.builder.build_int_truncate(code, t, "char").ok()?;
        if let Some(d) = dest {
            self.store_place(fi, d, ch.into(), slots);
        }
        let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
        Some(())
    }

    /// A whole number past 128 bits as its digits (owner, 2026-10-03): the
    /// magnitude's words in memory, and the sign, to the runtime.
    pub(super) fn wide_int_text(&mut self, i: IntValue<'ctx>, signed: bool) -> Option<BasicValueEnum<'ctx>> {
        let BasicTypeEnum::StructType(shape) = self.ty(&Ty::utf8())? else { return None };
        let out = self.stack(shape, "text").ok()?;
        let bits = i.get_type().get_bit_width();
        let words = bits.div_ceil(64);
        let mt = self.ctx.custom_width_int_type(std::num::NonZeroU32::new(64 * words)?).ok()?;
        let neg = if signed { self.builder.build_int_compare(IntPredicate::SLT, i, i.get_type().const_zero(), "neg").ok()? } else { self.ctx.bool_type().const_zero() };
        let flipped = self.builder.build_int_neg(i, "flipped").ok()?;
        let mag = self.builder.build_select(neg, flipped, i, "mag").ok()?.into_int_value();
        let mag = self.builder.build_int_z_extend_or_bit_cast(mag, mt, "words").ok()?;
        let slot = self.stack(mt, "mag").ok()?;
        let _ = self.builder.build_store(slot, mag);
        let f = self.rt("cyborg_wide_to_text");
        let _ = self.builder.build_call(f, &[out.into(), slot.into(), self.ctx.i64_type().const_int(words as u64, false).into(), neg.into()], "");
        self.builder.build_load(shape, out, "text").ok()
    }

    /// Text into a bin (owner, 2026-09-24: b): the runtime reads the
    /// grammar and rounds once from the digits to the width; `not_a_number`
    /// is reason 0, `out_of_range` (past the largest, or outside a range
    /// type's bounds) reason 1.
    #[allow(clippy::too_many_arguments)]
    fn text_into_bin(&mut self, fi: usize, bits: u16, to: &Ty, arg: &Arg, dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let o = match arg {
            Arg::Value(o) => o.clone(),
            Arg::Copy(p) | Arg::Give(p) | Arg::Lend { place: p, .. } => Operand::Read(p.clone()),
            _ => return None,
        };
        let v = self.operand(fi, &o, slots)?;
        let (p, n) = self.text_parts(v)?;
        let f64t = self.ctx.f64_type();
        let out = self.stack(f64t, "parsed").ok()?;
        let f = self.rt("cyborg_parse_bin");
        let w = self.ctx.i32_type().const_int(bits as u64, false);
        let status = basic_result(self.builder.build_call(f, &[p.into(), n.into(), w.into(), out.into()], "status").ok()?)?.into_int_value();
        let arm = |index: usize| fail.iter().find(|a| a.index == index).or_else(|| fail.first()).map(|a| blocks[a.block.0 as usize]);
        let i8t = self.ctx.i8_type();
        let nan = self.builder.build_int_compare(IntPredicate::EQ, status, i8t.const_int(1, false), "not_a_number").ok()?;
        let number = self.ctx.append_basic_block(func, "is_number");
        let _ = self.builder.build_conditional_branch(nan, arm(0)?, number);
        self.builder.position_at_end(number);
        let past = self.builder.build_int_compare(IntPredicate::EQ, status, i8t.const_int(2, false), "past").ok()?;
        let x = self.builder.build_load(f64t, out, "value").ok()?.into_float_value();
        let mut outside = past;
        if let Some(crate::types::Bound::Bin { lo, hi, .. }) = to.bound() {
            let (Ok(lo), Ok(hi)) = (lo.replace('_', "").parse::<f64>(), hi.replace('_', "").parse::<f64>()) else { return None };
            let below = self.builder.build_float_compare(inkwell::FloatPredicate::OLT, x, f64t.const_float(lo), "below").ok()?;
            let above = self.builder.build_float_compare(inkwell::FloatPredicate::OGT, x, f64t.const_float(hi), "above").ok()?;
            let b = self.builder.build_or(below, above, "outside").ok()?;
            outside = self.builder.build_or(outside, b, "out_of_range").ok()?;
        }
        let fits = self.ctx.append_basic_block(func, "in_range");
        let _ = self.builder.build_conditional_branch(outside, arm(1)?, fits);
        self.builder.position_at_end(fits);
        let t = match self.ty(to)? {
            BasicTypeEnum::FloatType(t) => t,
            _ => return None,
        };
        let r = if t == f64t { x } else { self.float_cast(x, t)? };
        if let Some(d) = dest {
            self.store_place(fi, d, r.into(), slots);
        }
        let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
        Some(())
    }

    /// Into a `deci` where it can fail (owner, 2026-09-29): a whole number
    /// whose type could have more digits than the width holds fails
    /// `too_many_digits`; text fails `not_a_number`, `out_of_range` or
    /// `too_many_digits`, in that order.
    #[allow(clippy::too_many_arguments)]
    fn into_deci(&mut self, fi: usize, to: &Ty, written: bool, from: &Ty, arg: &Arg, dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        let Ty::Deci(bits) = to.base().clone() else { return None };
        let o = match arg {
            Arg::Value(o) => o.clone(),
            Arg::Copy(p) | Arg::Give(p) | Arg::Lend { place: p, .. } => Operand::Read(p.clone()),
            _ => return None,
        };
        let v = self.operand(fi, &o, slots)?;
        // Each reason's arm is its place in the checker's list.
        let literal = match &o {
            Operand::Const(Const::Int(i)) => Some(i.unsigned_abs().to_string().len() as u32),
            _ => None,
        };
        let names = crate::deci::conv_fails(from, to, literal)?;
        let arm = |name: &str| -> Option<LlBlock<'ctx>> {
            let index = names.iter().position(|n| *n == name)?;
            fail.iter().find(|a| a.index == index).or_else(|| fail.first()).map(|a| blocks[a.block.0 as usize])
        };
        let w = self.ctx.i128_type();
        let out = self.stack(w, "deci").ok()?;
        let _ = self.builder.build_store(out, w.const_zero());
        let i32t = self.ctx.i32_type();
        let width = i32t.const_int(bits as u64, false);
        let i8t = self.ctx.i8_type();
        // The runtime's status, and what each non-zero one means: a reason
        // of the list, or a stop.
        let (status, meaning): (Option<IntValue<'ctx>>, Vec<(u64, Result<&str, String>)>) = match from.base() {
            Ty::Str(_) => {
                let (p, n) = self.text_parts(v)?;
                let f = self.rt_deci("cyborg_deci_parse")?;
                let written = i8t.const_int(written as u64, false);
                let status = basic_result(self.builder.build_call(f, &[p.into(), n.into(), width.into(), out.into(), written.into()], "status").ok()?)?.into_int_value();
                (Some(status), vec![(1, Ok("not_a_number")), (2, Ok("out_of_range")), (3, Ok("too_many_digits"))])
            }
            // Past 128 bits: its digits, read as text into a deci is.
            Ty::Int(_) if v.into_int_value().get_type().get_bit_width() > 128 => {
                let signed = self.operand_signed(fi, &o).unwrap_or(true);
                let text = self.wide_int_text(v.into_int_value(), signed)?;
                let (p, n) = self.text_parts(text)?;
                let f = self.rt_deci("cyborg_deci_parse")?;
                // A whole number's digits all count, as they always have.
                let all = i8t.const_int(1, false);
                let status = basic_result(self.builder.build_call(f, &[p.into(), n.into(), width.into(), out.into(), all.into()], "status").ok()?)?.into_int_value();
                let free = self.rt("cyborg_str_free");
                let _ = self.builder.build_call(free, &[p.into(), n.into()], "");
                (Some(status), vec![(2, Ok("too_many_digits")), (3, Ok("too_many_digits"))])
            }
            Ty::Int(_) => {
                let signed = self.operand_signed(fi, &o).unwrap_or(true);
                let x = self.builder.build_int_cast_sign_flag(v.into_int_value(), w, signed, "whole").ok()?;
                let (lo, hi) = self.deci_halves(x)?;
                let f = self.rt_deci("cyborg_deci_from_int")?;
                let status = basic_result(self.builder.build_call(f, &[lo.into(), hi.into(), width.into(), out.into()], "status").ok()?)?.into_int_value();
                (Some(status), vec![(1, Ok("too_many_digits"))])
            }
            Ty::Bin(_) => {
                // Only a range type makes this one fail: the value first.
                let d = self.deci_convert(fi, v, from, &Ty::Deci(bits), false, true, func)?;
                let _ = self.builder.build_store(out, self.builder.build_int_z_extend_or_bit_cast(d.into_int_value(), w, "deci").ok()?);
                (None, vec![])
            }
            Ty::Deci(from_bits) if *from_bits > bits => {
                let (lo, hi) = self.deci_halves(v.into_int_value())?;
                let f = self.rt_deci("cyborg_deci_narrow")?;
                let status = basic_result(self.builder.build_call(f, &[lo.into(), hi.into(), i32t.const_int(*from_bits as u64, false).into(), width.into(), out.into()], "status").ok()?)?.into_int_value();
                // Past the narrower's largest is a failure too, its one:
                // a stop ignored the `otherwise` (check claude, seed 18003).
                (Some(status), vec![(1, Ok("too_many_digits")), (2, Ok("too_many_digits"))])
            }
            Ty::Deci(_) => {
                let d = self.deci_convert(fi, v, from, &Ty::Deci(bits), false, true, func)?;
                let _ = self.builder.build_store(out, self.builder.build_int_z_extend_or_bit_cast(d.into_int_value(), w, "deci").ok()?);
                (None, vec![])
            }
            _ => {
                self.cannot("this conversion into a deci has no shape in the generated code yet", None);
                return None;
            }
        };
        if let Some(status) = status {
            for (code, what) in meaning {
                let bad = self.builder.build_int_compare(IntPredicate::EQ, status, i8t.const_int(code, false), "fails").ok()?;
                match what {
                    Ok(name) => {
                        // A reason the list does not have is one this
                        // source can never give.
                        let Some(to_arm) = arm(name) else { continue };
                        let next = self.ctx.append_basic_block(func, "converted");
                        let _ = self.builder.build_conditional_branch(bad, to_arm, next);
                        self.builder.position_at_end(next);
                    }
                    Err(why) => self.deci_stop(fi, bad, &why, func)?,
                }
            }
        }
        let x = self.builder.build_load(w, out, "deci").ok()?.into_int_value();
        let t = self.ty(&Ty::Deci(bits))?.into_int_type();
        let r = if bits == 128 { x } else { self.builder.build_int_truncate(x, t, "deci").ok()? };
        // A range type: outside its bounds fails `out_of_range`.
        if let Some(crate::types::Bound::Bin { lo, hi, .. }) = to.bound() {
            let (lo, hi) = (lo.clone(), hi.clone());
            let outside = self.deci_outside(r, bits, &lo, &hi)?;
            let next = self.ctx.append_basic_block(func, "in_range");
            let _ = self.builder.build_conditional_branch(outside, arm("out_of_range")?, next);
            self.builder.position_at_end(next);
        }
        if let Some(d) = dest {
            self.store_place(fi, d, r.into(), slots);
        }
        let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
        Some(())
    }

    /// Into a bounded text (`input.md`): the text, then its length against
    /// the bound, failing `out_of_range` outside it — and the text just
    /// made is given back on that path. A number or a bool is ASCII, so its
    /// bytes, points and graphemes are one count; a text counted in points
    /// or graphemes has to walk its bytes, which waits with std.
    #[allow(clippy::too_many_arguments)]
    fn emit_bounded_text(&mut self, fi: usize, from: &Ty, to: &Ty, arg: &Arg, dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        use crate::types::Enc;
        // Into ASCII from what may not be (owner, 2026-09-30): `not_ascii`
        // is the first reason, before a bound's `out_of_range`.
        let ascii = matches!(to.base(), Ty::Str(Enc::Ascii))
            && match from.base() {
                Ty::Str(e) => *e != Enc::Ascii,
                Ty::Char(u) => *u != crate::types::Unit::Byte,
                _ => false,
            };
        let bound = match to.bound() {
            Some(crate::types::Bound::Len { lo, hi, unit }) => Some((*lo, *hi, *unit)),
            _ => None,
        };
        if !ascii && bound.is_none() {
            return None;
        }
        let operand = match arg {
            Arg::Value(o) => o.clone(),
            Arg::Copy(p) | Arg::Give(p) | Arg::Lend { place: p, .. } => Operand::Read(p.clone()),
            _ => return None,
        };
        let v = self.operand(fi, &operand, slots)?;
        let text = self.to_text(fi, &operand, v, &to.base().clone())?;
        let (p, raw) = self.text_parts(text)?;
        let arm = |index: usize| fail.iter().find(|a| a.index == index).or_else(|| fail.first()).map(|a| blocks[a.block.0 as usize]);
        // Each reason: the text built is given back, then its arm.
        let mut reason = 0usize;
        let mut leave_if = |this: &mut Self, bad: IntValue<'ctx>, name: &str| -> Option<()> {
            let fits = this.ctx.append_basic_block(func, "fits");
            let give_back = this.ctx.append_basic_block(func, name);
            let _ = this.builder.build_conditional_branch(bad, give_back, fits);
            this.builder.position_at_end(give_back);
            let free = this.rt("cyborg_str_free");
            let _ = this.builder.build_call(free, &[p.into(), raw.into()], "");
            let _ = this.builder.build_unconditional_branch(arm(reason)?);
            this.builder.position_at_end(fits);
            reason += 1;
            Some(())
        };
        if ascii {
            let f = self.rt("cyborg_str_is_ascii");
            let is = basic_result(self.builder.build_call(f, &[p.into(), raw.into()], "is_ascii").ok()?)?.into_int_value();
            let bad = self.builder.build_int_compare(IntPredicate::EQ, is, self.ctx.i8_type().const_zero(), "not_ascii").ok()?;
            leave_if(self, bad, "not_ascii")?;
        }
        if let Some((lo, hi, unit)) = bound {
            let walk = matches!(from.base(), Ty::Str(_) | Ty::Char(_)) && matches!(unit, Some(crate::types::Unit::Point) | Some(crate::types::Unit::Grapheme));
            let mut n = self.real_length(raw);
            // Text or a character counted in points or graphemes: a walk.
            if walk {
                let f = self.rt(if unit == Some(crate::types::Unit::Point) { "cyborg_str_points" } else { "cyborg_str_graphemes" });
                n = basic_result(self.builder.build_call(f, &[p.into(), raw.into()], "count").ok()?)?.into_int_value();
            }
            let i64t = self.ctx.i64_type();
            let below = self.builder.build_int_compare(IntPredicate::ULT, n, i64t.const_int(lo, false), "below").ok()?;
            let above = self.builder.build_int_compare(IntPredicate::UGT, n, i64t.const_int(hi, false), "above").ok()?;
            let out = self.builder.build_or(below, above, "out_of_range").ok()?;
            leave_if(self, out, "does_not_fit")?;
        }
        if let Some(d) = dest {
            self.store_place(fi, d, text, slots);
        }
        let _ = self.builder.build_unconditional_branch(blocks[ok.0 as usize]);
        Some(())
    }

    /// `list.fill[value, count]` (owner, 2026-09-30): room for exactly
    /// `count`, then the value in every place. A zero — `0`, `false` — is
    /// the system's zeroed pages, as C's `calloc` gives them, never written
    /// one by one; anything else is one store per place, a loop LLVM makes
    /// a `memset` where it can. The count's own site has already said it
    /// is not below 0.
    pub(super) fn list_fill(&mut self, fi: usize, args: &[Operand], out: &Ty, slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let elem = Self::elem_of(out)?;
        if elem.is_owned() {
            self.cannot("`list.fill` of a value that owns memory (text, a list, a map) has no shape in the generated code yet", None);
            return None;
        }
        let shape = match self.ty(out)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        let lty = self.ty(&elem)?;
        let esize = self.size_of_ty(&elem)?;
        let w = self.ctx.i64_type();
        let c = self.operand(fi, args.get(1)?, slots)?.into_int_value();
        let signed = self.operand_signed(fi, args.get(1)?).unwrap_or(true);
        let n = self.builder.build_int_cast_sign_flag(c, w, signed, "count").ok()?;
        let bytes = self.builder.build_int_mul(n, w.const_int(esize, false), "bytes").ok()?;
        let zero = match args.first()? {
            Operand::Const(Const::Int(0)) | Operand::Const(Const::Bool(false)) => true,
            Operand::Const(Const::Bin(t)) => t.parse::<f64>().is_ok_and(|x| x == 0.0 && x.is_sign_positive()),
            _ => false,
        };
        let data = if zero {
            let f = self.rt("cyborg_alloc_zeroed");
            basic_result(self.builder.build_call(f, &[bytes.into()], "zeroed").ok()?)?.into_pointer_value()
        } else {
            let f = self.rt("cyborg_alloc");
            let p = basic_result(self.builder.build_call(f, &[bytes.into()], "room").ok()?)?.into_pointer_value();
            let v = self.operand(fi, args.first()?, slots)?;
            let v = self.fit(v, lty);
            let before = self.builder.get_insert_block()?;
            let head = self.ctx.append_basic_block(func, "fill");
            let body = self.ctx.append_basic_block(func, "fill_one");
            let done = self.ctx.append_basic_block(func, "filled");
            let _ = self.builder.build_unconditional_branch(head);
            self.builder.position_at_end(head);
            let i = self.builder.build_phi(w, "i").ok()?;
            i.add_incoming(&[(&w.const_zero(), before)]);
            let iv = i.as_basic_value().into_int_value();
            let more = self.builder.build_int_compare(IntPredicate::ULT, iv, n, "more").ok()?;
            let _ = self.builder.build_conditional_branch(more, body, done);
            self.builder.position_at_end(body);
            let off = self.builder.build_int_mul(iv, w.const_int(esize, false), "off").ok()?;
            let at = unsafe { self.builder.build_gep(self.ctx.i8_type(), p, &[off], "at").ok()? };
            let _ = self.builder.build_store(at, v);
            let next = self.builder.build_int_add(iv, w.const_int(1, false), "next").ok()?;
            i.add_incoming(&[(&next, body)]);
            let _ = self.builder.build_unconditional_branch(head);
            self.builder.position_at_end(done);
            p
        };
        let mut l = shape.get_undef();
        l = self.builder.build_insert_value(l, data, 0, "data").ok()?.into_struct_value();
        l = self.builder.build_insert_value(l, n, 1, "len").ok()?.into_struct_value();
        l = self.builder.build_insert_value(l, n, 2, "cap").ok()?.into_struct_value();
        Some(l.into())
    }

    pub(super) fn emit_failing_call(&mut self, fi: usize, callee: &Callee, args: &[Arg], dest: &Option<Place>, ok: BlockId, fail: &[FailArm], blocks: &[LlBlock<'ctx>], slots: &[Option<PointerValue<'ctx>>], func: FunctionValue<'ctx>) -> Option<()> {
        if let Callee::Extern(id) = callee {
            return self.emit_ranged_extern(fi, *id, callee, args, dest, ok, fail, blocks, slots, func);
        }
        if let Callee::Convert(to, written) = callee {
            return self.emit_failing_convert(fi, to, *written, args, dest, ok, fail, blocks, slots, func);
        }
        if let Callee::Op(Op::PtrRead) = callee {
            return self.emit_checked_ptr_read(fi, args, dest, ok, fail, blocks, slots, func);
        }
        if let Callee::Op(Op::CharFromCode(unit)) = callee {
            return self.char_from_code(fi, *unit, args, dest, ok, fail, blocks, slots, func);
        }
        // What comes back and why it can fail: the callee's own, or — for a
        // function held as a value — its `fn[…] -> T or fail[…]` type's.
        let (ret, reasons, shape) = match callee {
            Callee::Fn(idx) => {
                let t = idx.0 as usize;
                (self.p.functions[t].ret.clone(), self.p.functions[t].fails.clone(), self.fail_shape(t)?)
            }
            Callee::Value(o) => {
                let Ty::Fn(ft) = self.operand_cy_ty(fi, o)?.base().clone() else { return None };
                let shape = self.fn_ty_fail_shape(&ft)?;
                (ft.ret.clone(), ft.fails.clone(), shape)
            }
            _ => return None,
        };
        let outcome = self.call(fi, callee, args, slots)?;
        let cases = reasons.len() + 1;
        let tmp = self.stack(shape, "outcome").ok()?;
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
            let ret = ret.clone();
            if let Some(t) = self.ty(&ret) {
                if let Ok(at) = self.builder.build_struct_gep(shape, tmp, 1, "answer") {
                    if let Ok(v) = self.builder.build_load(t, at, "v") {
                        let v = self.clamp_answer(v, &ret);
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
                let carried = reasons.get(arm.index).and_then(|(_, t)| t.clone());
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
        let out = self.stack(shape, "joined").ok()?;
        let f = self.rt("cyborg_str_concat");
        let _ = self.builder.build_call(f, &[out.into(), ap.into(), alen.into(), bp.into(), blen.into()], "");
        self.builder.build_load(shape, out, "text").ok()
    }

    /// `to.str.utf8[…]` of a number or a bool: its stored form
    /// (`conversion.md`) — decimal digits for a whole number, the full
    /// expansion for a bin, `true`/`false` — as a new text on the heap. A
    /// text converts to itself.
    pub(super) fn to_text(&mut self, fi: usize, operand: &Operand, v: BasicValueEnum<'ctx>, to: &Ty) -> Option<BasicValueEnum<'ctx>> {
        self.to_text_as(fi, operand, v, to, false)
    }

    /// `useful`: the useful form (`to.useful.str.utf8`, owner 2026-09-30) —
    /// a bin's shortest decimal, a deci without trailing zeros, an enum
    /// case's bare name — where the plain form is the exact one.
    pub(super) fn to_text_as(&mut self, fi: usize, operand: &Operand, v: BasicValueEnum<'ctx>, to: &Ty, useful: bool) -> Option<BasicValueEnum<'ctx>> {
        let from = match operand {
            Operand::Const(Const::Int(_)) => Ty::exact(),
            Operand::Const(Const::Bool(_)) => Ty::Bool,
            Operand::Const(Const::Bin(_)) => Ty::Bin(crate::types::BinW::Fixed(64)),
            Operand::Const(Const::Deci { bits, .. }) => Ty::Deci(*bits),
            Operand::Const(Const::Str(_)) => Ty::Str(crate::types::Enc::Utf8),
            Operand::Const(Const::Char(_)) => Ty::Char(crate::types::Unit::Point),
            // `size.of[…]` and `align.of[…]`: a byte count, a `uint64`.
            Operand::Const(Const::SizeOf { .. }) => Ty::u64(),
            _ => self.operand_cy_ty(fi, operand)?,
        };
        let shape = match self.ty(to)? {
            BasicTypeEnum::StructType(s) => s,
            _ => return None,
        };
        if let Ty::Str(_) | Ty::Char(crate::types::Unit::Grapheme) = from.base() {
            let (p, n) = self.text_parts(v)?;
            let out = self.stack(shape, "text").ok()?;
            let f = self.rt("cyborg_str_copy");
            let _ = self.builder.build_call(f, &[out.into(), p.into(), n.into()], "");
            return self.builder.build_load(shape, out, "text").ok();
        }
        let out = self.stack(shape, "text").ok()?;
        let i64t = self.ctx.i64_type();
        match (from.base(), v) {
            (Ty::Char(_), BasicValueEnum::IntValue(c)) => {
                let code = self.builder.build_int_z_extend_or_bit_cast(c, self.ctx.i32_type(), "code").ok()?;
                let f = self.rt("cyborg_char_to_text");
                let _ = self.builder.build_call(f, &[out.into(), code.into()], "");
            }
            (Ty::Bool, BasicValueEnum::IntValue(b)) => {
                // Two constants: the binary's own bytes, marked so.
                let yes = self.const_text("true");
                let no = self.const_text("false");
                return self.builder.build_select(b, yes, no, "text").ok();
            }
            // Past 128 bits (owner, 2026-10-03): the magnitude's words in
            // memory, and the sign.
            // And an unsigned 128 bits, whose top half `i128` reads as
            // negative.
            (Ty::Int(_), BasicValueEnum::IntValue(i)) if i.get_type().get_bit_width() > 128 || (i.get_type().get_bit_width() == 128 && self.operand_signed(fi, operand) == Some(false)) => {
                let signed = self.operand_signed(fi, operand).unwrap_or(true);
                return self.wide_int_text(i, signed);
            }
            (Ty::Int(_), BasicValueEnum::IntValue(i)) => {
                let signed = self.operand_signed(fi, operand).unwrap_or(true);
                let w = self.builder.build_int_cast_sign_flag(i, self.ctx.i128_type(), signed, "wide").ok()?;
                let lo = self.builder.build_int_truncate(w, i64t, "lo").ok()?;
                let sh = self.builder.build_right_shift(w, self.ctx.i128_type().const_int(64, false), false, "shifted").ok()?;
                let hi = self.builder.build_int_truncate(sh, i64t, "hi").ok()?;
                let f = self.rt("cyborg_int_to_text");
                let _ = self.builder.build_call(f, &[out.into(), lo.into(), hi.into()], "");
            }
            (Ty::Deci(bits), BasicValueEnum::IntValue(d)) => {
                let (lo, hi) = self.deci_halves(d)?;
                return self.deci_text_raw(lo, hi, *bits, useful);
            }
            (Ty::Bin(_) | Ty::Brain, BasicValueEnum::FloatValue(x)) => {
                let wide = if x.get_type() == self.ctx.f64_type() { x } else { self.builder.build_float_ext(x, self.ctx.f64_type(), "wide").ok()? };
                if useful {
                    let bits = match from.base() {
                        Ty::Bin(crate::types::BinW::Fixed(b)) => *b as u64,
                        Ty::Brain => crate::types::BRAIN_BITS as u64,
                        _ => 64,
                    };
                    let f = self.rt("cyborg_bin_to_text_useful");
                    let _ = self.builder.build_call(f, &[out.into(), wide.into(), self.ctx.i32_type().const_int(bits, false).into()], "");
                } else {
                    let f = self.rt("cyborg_bin_to_text");
                    let _ = self.builder.build_call(f, &[out.into(), wide.into()], "");
                }
            }
            // An enum case: `Size.big`, or usefully `big` (owner,
            // 2026-09-30). Its tag is its place in the list, or a
            // `layout.C` enum's C number.
            (Ty::Enum { id, .. }, BasicValueEnum::IntValue(tag)) => {
                let info = self.p.items.enums[id.0 as usize].clone();
                let names: Vec<String> = info.cases.iter().map(|c| if useful { c.name.clone() } else { format!("{}.{}", info.name, c.name) }).collect();
                // A C number no case has — only past a trusted door — shows
                // as the number, made only then.
                let mut any = self.ctx.bool_type().const_zero();
                let mut named = self.const_text("");
                for (k, name) in names.iter().enumerate().rev() {
                    let n = match &info.numbers {
                        Some(ns) => ns[k] as u64,
                        None => k as u64,
                    };
                    let is = self.builder.build_int_compare(IntPredicate::EQ, tag, tag.get_type().const_int(n, true), "is_case").ok()?;
                    any = self.builder.build_or(any, is, "a_case").ok()?;
                    let t = self.const_text(name);
                    named = self.builder.build_select(is, t, named, "name").ok()?;
                }
                let func = self.builder.get_insert_block()?.get_parent()?;
                let here = self.builder.get_insert_block()?;
                let odd = self.ctx.append_basic_block(func, "no_case");
                let join = self.ctx.append_basic_block(func, "named");
                let _ = self.builder.build_conditional_branch(any, join, odd);
                self.builder.position_at_end(odd);
                let w = self.builder.build_int_s_extend_or_bit_cast(tag, self.ctx.i128_type(), "wide").ok()?;
                let lo = self.builder.build_int_truncate(w, i64t, "lo").ok()?;
                let sh = self.builder.build_right_shift(w, self.ctx.i128_type().const_int(64, false), true, "shifted").ok()?;
                let hi = self.builder.build_int_truncate(sh, i64t, "hi").ok()?;
                let f = self.rt("cyborg_int_to_text");
                let _ = self.builder.build_call(f, &[out.into(), lo.into(), hi.into()], "");
                let number = self.builder.build_load(shape, out, "number").ok()?;
                let odd_end = self.builder.get_insert_block()?;
                let _ = self.builder.build_unconditional_branch(join);
                self.builder.position_at_end(join);
                let phi = self.builder.build_phi(shape, "text").ok()?;
                phi.add_incoming(&[(&named, here), (&number, odd_end)]);
                return Some(phi.as_basic_value());
            }
            _ => {
                self.cannot("this conversion has no shape in the generated code yet", None);
                return None;
            }
        }
        self.builder.build_load(shape, out, "text").ok()
    }

    /// Give back a text made on the way (a character's, before a join).
    pub(super) fn free_text(&mut self, t: BasicValueEnum<'ctx>) {
        if let Some((p, n)) = self.text_parts(t) {
            let f = self.rt("cyborg_str_free");
            let _ = self.builder.build_call(f, &[p.into(), n.into()], "");
        }
    }

    /// A text known at compile time: the binary's own bytes, marked so.
    pub(super) fn const_text(&mut self, text: &str) -> BasicValueEnum<'ctx> {
        let bytes = self.global_bytes(&text.to_string());
        let len = self.ctx.i64_type().const_int(text.len() as u64 | (1u64 << 63), false);
        let h = self.handle(2);
        h.const_named_struct(&[bytes.into(), len.into()]).into()
    }

    pub(super) fn text_parts_pub(&mut self, v: BasicValueEnum<'ctx>) -> Option<(PointerValue<'ctx>, IntValue<'ctx>)> {
        self.text_parts(v)
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
        // `maps.md` lets a key type bring its own `hash` and `op equal_to`, and
        // makes "equal keys hash the same" a check site: where it is not
        // proven or trusted, every comparison that finds two keys equal
        // asks whether they hash the same (as the oracle does), and a map
        // made where no site says otherwise asks too.
        let own = crate::mir::build::key_has_own_hash(&self.p.items, &k);
        let fi = self.cur_fi;
        let sites: Vec<SiteId> = if own { self.op_sites.iter().copied().filter(|s| matches!(self.p.functions.get(fi).map(|f| &f.site(*s).kind), Some(SiteKind::KeyHashAgrees))).collect() } else { Vec::new() };
        let open = sites.iter().copied().find(|s| self.gets_check(&self.p.functions[fi].site(*s).verdict.clone()));
        let checked = own && (sites.is_empty() || open.is_some());
        // The comparison that finds the fault runs inside the map, shared by
        // every use of it: this use says which site it is, beforehand.
        if let Some(s) = open {
            let (msg, _) = self.site_message(fi, s);
            let text = self.global_bytes(&msg);
            let (mg, lg) = self.key_site_globals();
            let _ = self.builder.build_store(mg, text);
            let _ = self.builder.build_store(lg, self.ctx.i64_type().const_int(msg.len() as u64, false));
        }
        let name = format!("cy_mapspec_{}_{}{}", self.type_key(&k), self.type_key(&v), if checked { "_checked" } else { "" });
        if let Some(g) = self.module.get_global(&name) {
            return Some(g.as_pointer_value());
        }
        let w = self.ctx.i64_type();
        let p = self.ctx.ptr_type(AddressSpace::default());
        let ks = self.size_of_ty(&k)?;
        let vs = self.size_of_ty(&v)?;
        let (hash, same) = if own {
            self.own_key_fns(&k, checked)?
        } else {
            (self.hash_fn(&k)?, self.same_fn(&k)?)
        };
        let drop_k = self.drop_fn(&k).map(|f| f.as_global_value().as_pointer_value()).unwrap_or_else(|| p.const_null());
        let drop_v = self.drop_fn(&v).map(|f| f.as_global_value().as_pointer_value()).unwrap_or_else(|| p.const_null());
        let shape = self.ctx.struct_type(&[w.into(), w.into(), p.into(), p.into(), p.into(), p.into(), w.into()], false);
        let init = shape.const_named_struct(&[
            w.const_int(ks, false).into(),
            w.const_int(vs, false).into(),
            hash.as_global_value().as_pointer_value().into(),
            same.as_global_value().as_pointer_value().into(),
            drop_k.into(),
            drop_v.into(),
            w.const_int(checked as u64, false).into(),
        ]);
        let g = self.module.add_global(shape, None, &name);
        g.set_initializer(&init);
        g.set_constant(true);
        g.set_linkage(Linkage::Internal);
        Some(g.as_pointer_value())
    }

    /// Where a map's key check says it fired: set by each use of a map
    /// whose check is open. A map written out in full has none to name.
    fn key_site_globals(&mut self) -> (PointerValue<'ctx>, PointerValue<'ctx>) {
        let p = self.ctx.ptr_type(AddressSpace::default());
        let w = self.ctx.i64_type();
        if let (Some(m), Some(l)) = (self.module.get_global("cy_key_site_msg"), self.module.get_global("cy_key_site_len")) {
            return (m.as_pointer_value(), l.as_pointer_value());
        }
        let fallback = "check: two keys this says are equal can hash differently, and the map would hold both\nwhere: a map written out in full\n";
        let text = self.global_bytes(fallback);
        let m = self.module.add_global(p, None, "cy_key_site_msg");
        m.set_initializer(&text);
        m.set_linkage(Linkage::Internal);
        let l = self.module.add_global(w, None, "cy_key_site_len");
        l.set_initializer(&w.const_int(fallback.len() as u64, false));
        l.set_linkage(Linkage::Internal);
        (m.as_pointer_value(), l.as_pointer_value())
    }

    /// A key type's own `hash` and `op equal_to` (`maps.md`), as the map's hash
    /// and sameness: each a small wrapper around the program's function.
    /// Checked, the sameness also asks, whenever `==` says two keys are
    /// equal, whether `hash` agrees — and stops the program if not, with
    /// the site's words.
    fn own_key_fns(&mut self, k: &Ty, checked: bool) -> Option<(FunctionValue<'ctx>, FunctionValue<'ctx>)> {
        let key = self.type_key(k);
        let hname = format!("cy_ownhash_{}", key);
        let sname = format!("cy_ownsame_{}{}", key, if checked { "_checked" } else { "" });
        if let (Some(h), Some(s)) = (self.helpers.get(&hname).copied(), self.helpers.get(&sname).copied()) {
            return Some((h, s));
        }
        // The program's own: `fn hash [lent 'v': K]` and `op == [K, K]`.
        let find = |this: &Self, name: &str, n: usize| -> Option<usize> {
            this.p.functions.iter().position(|f| {
                f.name == name && f.params.len() == n && f.params.iter().all(|p| f.locals[p.0 as usize].ty.base() == k.base())
            })
        };
        let (Some(hi), Some(ei)) = (find(self, "hash", 1), find(self, "equal_to", 2)) else {
            let spelled = crate::types::TySpelling { ty: k, structs: &self.p.items.structs, enums: &self.p.items.enums }.to_string();
            self.cannot(format!("`{}` has its own `hash` and `op equal_to`, and one of them is not in the program", spelled), None);
            return None;
        };
        let (uh, ue) = (*self.fns.get(&hi)?, *self.fns.get(&ei)?);
        // Each parameter as the function takes it: lent is an address,
        // owned a value loaded from it.
        let lent_of = |this: &Self, fi: usize, i: usize| -> bool {
            let f = &this.p.functions[fi];
            matches!(f.locals[f.params[i].0 as usize].kind, LocalKind::Lent { .. })
        };
        let hl = lent_of(self, hi, 0);
        let el = (lent_of(self, ei, 0), lent_of(self, ei, 1));
        let kt = self.ty(k)?;
        let p = self.ctx.ptr_type(AddressSpace::default());
        let w = self.ctx.i64_type();
        let here = self.builder.get_insert_block();
        let hf = match self.helpers.get(&hname).copied() {
            Some(f) => f,
            None => {
                let f = self.module.add_function(&hname, w.fn_type(&[p.into()], false), Some(Linkage::Internal));
                self.helpers.insert(hname.clone(), f);
                let entry = self.ctx.append_basic_block(f, "entry");
                self.builder.position_at_end(entry);
                let a = f.get_nth_param(0)?.into_pointer_value();
                let arg: BasicValueEnum = if hl { a.into() } else { self.builder.build_load(kt, a, "k").ok()? };
                let r = basic_result(self.builder.build_call(uh, &[arg.into()], "h").ok()?)?;
                let r = self.fit(r, w.into());
                let _ = self.builder.build_return(Some(&r));
                f
            }
        };
        let sf = self.module.add_function(&sname, self.ctx.bool_type().fn_type(&[p.into(), p.into()], false), Some(Linkage::Internal));
        self.helpers.insert(sname, sf);
        let entry = self.ctx.append_basic_block(sf, "entry");
        self.builder.position_at_end(entry);
        let a = sf.get_nth_param(0)?.into_pointer_value();
        let b = sf.get_nth_param(1)?.into_pointer_value();
        let av: BasicValueEnum = if el.0 { a.into() } else { self.builder.build_load(kt, a, "a").ok()? };
        let bv: BasicValueEnum = if el.1 { b.into() } else { self.builder.build_load(kt, b, "b").ok()? };
        let same = basic_result(self.builder.build_call(ue, &[av.into(), bv.into()], "same").ok()?)?.into_int_value();
        if checked {
            let ask = self.ctx.append_basic_block(sf, "equal");
            let done = self.ctx.append_basic_block(sf, "done");
            let bad = self.ctx.append_basic_block(sf, "hash_disagrees");
            let _ = self.builder.build_conditional_branch(same, ask, done);
            self.builder.position_at_end(ask);
            let ha = basic_result(self.builder.build_call(hf, &[a.into()], "ha").ok()?)?.into_int_value();
            let hb = basic_result(self.builder.build_call(hf, &[b.into()], "hb").ok()?)?.into_int_value();
            let differ = self.builder.build_int_compare(IntPredicate::NE, ha, hb, "differ").ok()?;
            let _ = self.builder.build_conditional_branch(differ, bad, done);
            self.builder.position_at_end(bad);
            // The site the last use of a map said it was.
            let (mg, lg) = self.key_site_globals();
            let text = self.builder.build_load(p, mg, "msg").ok()?;
            let len = self.builder.build_load(w, lg, "len").ok()?;
            let f = self.module.get_function("cyborg_check_failed")?;
            let _ = self.builder.build_call(f, &[text.into(), len.into()], "");
            let _ = self.builder.build_unreachable();
            self.builder.position_at_end(done);
        }
        let _ = self.builder.build_return(Some(&same));
        if let Some(bb) = here {
            self.builder.position_at_end(bb);
        }
        Some((hf, sf))
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
            Ty::Int(_) | Ty::Bool | Ty::Char(crate::types::Unit::Byte | crate::types::Unit::Point | crate::types::Unit::Free(_)) | Ty::Duration => {
                let l = self.ty(t)?;
                let v = self.builder.build_load(l, at, "v").ok()?.into_int_value();
                let v = self.builder.build_int_z_extend_or_bit_cast(v, w, "wide").ok()?;
                let f = self.rt("cyborg_hash_int");
                basic_result(self.builder.build_call(f, &[v.into()], "h").ok()?).map(|x| x.into_int_value())
            }
            // Over the value, as `==` compares it (`1.50` is `1.5`).
            Ty::Deci(bits) => {
                let l = self.ty(t)?;
                let v = self.builder.build_load(l, at, "v").ok()?.into_int_value();
                let (lo, hi) = self.deci_halves(v)?;
                let f = self.rt_deci("cyborg_deci_hash")?;
                basic_result(self.builder.build_call(f, &[lo.into(), hi.into(), self.ctx.i32_type().const_int(*bits as u64, false).into()], "h").ok()?).map(|x| x.into_int_value())
            }
            Ty::Str(_) | Ty::Char(crate::types::Unit::Grapheme) => {
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
                let acc = self.stack(w, "acc").ok()?;
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
    pub(super) fn same_fn(&mut self, t: &Ty) -> Option<FunctionValue<'ctx>> {
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
            Ty::Int(_) | Ty::Bool | Ty::Char(crate::types::Unit::Byte | crate::types::Unit::Point | crate::types::Unit::Free(_)) | Ty::Duration => {
                let l = self.ty(t)?;
                let x = self.builder.build_load(l, a, "a").ok()?.into_int_value();
                let y = self.builder.build_load(l, b, "b").ok()?.into_int_value();
                self.builder.build_int_compare(IntPredicate::EQ, x, y, "same").ok()
            }
            // By value, as `==` on two decimals is: `1.50` is `1.5`.
            Ty::Deci(bits) => {
                let l = self.ty(t)?;
                let x = self.builder.build_load(l, a, "a").ok()?;
                let y = self.builder.build_load(l, b, "b").ok()?;
                let r = self.deci_binary(0, &crate::ast::BinOp::Eq, x, y, *bits, true, func)?;
                Some(r.into_int_value())
            }
            Ty::Str(_) | Ty::Char(crate::types::Unit::Grapheme) => {
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
                let acc = self.stack(self.ctx.bool_type(), "same").ok()?;
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
            Ty::Bin(_) | Ty::Brain => {
                let l = self.ty(t)?;
                let x = self.builder.build_load(l, a, "a").ok()?.into_float_value();
                let y = self.builder.build_load(l, b, "b").ok()?.into_float_value();
                self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, x, y, "same").ok()
            }
            // The same case, and for a case that carries something, the
            // same thing carried.
            Ty::Enum { id, .. } => {
                let id = *id;
                let shape = match self.ty(t)? {
                    BasicTypeEnum::StructType(s) => s,
                    _ => return None,
                };
                let cases = self.p.items.enums[id.0 as usize].cases.len();
                let tag_ty = self.tag_type_of(id);
                let ta = self.builder.build_struct_gep(shape, a, 0, "ta").ok()?;
                let tb = self.builder.build_struct_gep(shape, b, 0, "tb").ok()?;
                let ta = self.builder.build_load(tag_ty, ta, "tag_a").ok()?.into_int_value();
                let tb = self.builder.build_load(tag_ty, tb, "tag_b").ok()?.into_int_value();
                let same_tag = self.builder.build_int_compare(IntPredicate::EQ, ta, tb, "same_tag").ok()?;
                let carried: Vec<(usize, Ty)> = (0..cases).filter_map(|c| self.case_payload_ty(id, c, t).map(|p| (c, p))).collect();
                if carried.is_empty() {
                    return Some(same_tag);
                }
                let acc = self.stack(self.ctx.bool_type(), "same").ok()?;
                let _ = self.builder.build_store(acc, same_tag);
                let done = self.ctx.append_basic_block(func, "same_done");
                let check = self.ctx.append_basic_block(func, "same_case");
                let _ = self.builder.build_conditional_branch(same_tag, check, done);
                self.builder.position_at_end(check);
                let mut arms = Vec::new();
                for (c, _) in &carried {
                    arms.push((tag_ty.const_int(*c as u64, false), self.ctx.append_basic_block(func, "same_payload")));
                }
                let _ = self.builder.build_switch(ta, done, &arms);
                for ((_, pt), (_, blk)) in carried.iter().zip(arms.iter()) {
                    self.builder.position_at_end(*blk);
                    let pa = self.case_slot(a, t)?;
                    let pb = self.case_slot(b, t)?;
                    let s = self.emit_same(pt, pa, pb, func)?;
                    let _ = self.builder.build_store(acc, s);
                    let _ = self.builder.build_unconditional_branch(done);
                }
                self.builder.position_at_end(done);
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
        let slot = self.stack(v.get_type(), name).ok()?;
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
        let tmp = self.stack(shape, "map").ok()?;
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
        let out = self.stack(vlt, "found").ok()?;
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
        let i_slot = self.stack(w, "i").ok()?;
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
        let kfresh = self.stack(klt, "kcopy").ok()?;
        let vfresh = self.stack(vlt, "vcopy").ok()?;
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
        let i_slot = self.stack(w, "i").ok()?;
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
