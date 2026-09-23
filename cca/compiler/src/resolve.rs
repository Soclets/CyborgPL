//! Item collection: the module tree of a Limb, every declared function,
//! struct, enum, interface, operator, const and extern, and how a path
//! written in one module reaches an item in another (`limbs.md`).

use std::collections::HashMap;

use crate::ast::{self, Item, ItemKind, Vis};
use crate::diag::{err, Diagnostic, Result};
use crate::source::Span;
use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FnId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemRef {
    Fn(FnId),
    ExternFn(crate::hir::ExternId),
    ExternLet(crate::hir::ExternId),
    Struct(StructId),
    Enum(EnumId),
    Iface(IfaceId),
    Const(crate::hir::ConstId),
    Module(ModuleId),
}

impl ItemRef {
    pub fn what(self) -> &'static str {
        match self {
            ItemRef::Fn(_) => "function",
            ItemRef::ExternFn(_) => "extern function",
            ItemRef::ExternLet(_) => "extern variable",
            ItemRef::Struct(_) => "struct",
            ItemRef::Enum(_) => "enum",
            ItemRef::Iface(_) => "interface",
            ItemRef::Const(_) => "const",
            ItemRef::Module(_) => "module",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    /// `limb`, `shapes`, `shapes.circle`.
    pub path: String,
    pub parent: Option<ModuleId>,
    pub vis: Vis,
    pub items: HashMap<String, (ItemRef, Vis, Span)>,
    /// Consts live in the value namespace, keyed by their quoted name.
    pub consts: HashMap<String, (crate::hir::ConstId, Vis, Span)>,
}

/// A parameter as declared, before instantiation.
#[derive(Debug, Clone)]
pub struct ParamDecl {
    pub kind: ParamKind,
    pub amp: bool,
    pub mutability: Option<ast::Mutability>,
    pub variadic: bool,
    pub name: String,
    /// `None` for an untyped parameter: an implicit blank.
    pub ty: Option<Ty>,
    /// The blank this parameter is, or is declared with.
    pub from_blank: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FnDeclInfo {
    pub name: String,
    pub path: String,
    pub module: ModuleId,
    pub vis: Vis,
    pub params: Vec<ParamDecl>,
    pub ret: Ty,
    pub fails: Vec<(String, Option<Ty>)>,
    /// Generic blanks by first appearance: name and interface bound.
    pub blanks: Vec<(String, Option<IfaceId>)>,
    pub body: Option<ast::Block>,
    /// `op` declarations: the operator's spelling.
    pub op: Option<String>,
    /// An anonymous `fn` literal (captures are resolved by the checker).
    pub is_literal: bool,
    pub span: Span,
    pub name_span: Span,
}

#[derive(Debug, Clone)]
pub struct ExternInfo {
    pub name: String,
    pub params: Vec<(String, Ty)>,
    pub variadic: bool,
    pub ret: Ty,
    /// A ranged return: fails `out_of_range` at the door.
    pub ranged: bool,
    /// For `extern let`: its type, and whether it may be written.
    pub is_let: bool,
    pub mutable: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConstInfo {
    pub name: String,
    pub module: ModuleId,
    pub vis: Vis,
    pub ty: Option<Ty>,
    pub value: ast::Expr,
    pub span: Span,
}

#[derive(Debug, Default)]
pub struct Items {
    pub modules: Vec<Module>,
    pub fns: Vec<FnDeclInfo>,
    pub externs: Vec<ExternInfo>,
    pub structs: Vec<StructInfo>,
    pub enums: Vec<EnumInfo>,
    pub ifaces: Vec<IfaceInfo>,
    pub consts: Vec<ConstInfo>,
    /// User operators by spelling; several may share one (picked by the left operand's type).
    pub ops: HashMap<String, Vec<FnId>>,
    /// `Vec satisfies Ordered;` claims, checked after collection.
    pub claims: Vec<(Ty, IfaceId, Span)>,
    pub main: Option<(ast::Block, ModuleId, Span)>,
    /// Pending struct/enum/iface bodies to resolve after every name is known.
    pending_structs: Vec<(StructId, ast::StructDecl, ModuleId)>,
    pending_enums: Vec<(EnumId, ast::EnumDecl, ModuleId)>,
    pending_ifaces: Vec<(IfaceId, ast::IfaceDecl, ModuleId)>,
    pending_fns: Vec<(FnId, ast::FnDecl, ModuleId, Option<String>)>,
    pending_externs: Vec<(crate::hir::ExternId, ast::FnDecl, ModuleId)>,
    pending_extern_lets: Vec<(crate::hir::ExternId, ast::Type, ModuleId)>,
    pending_consts: Vec<(crate::hir::ConstId, Option<ast::Type>, ModuleId)>,
    pending_claims: Vec<(ast::Type, ast::Path, ModuleId, Span)>,
}

pub const ROOT: ModuleId = ModuleId(0);

impl Items {
    pub fn module_path(&self, m: ModuleId) -> &str {
        &self.modules[m.0 as usize].path
    }

    /// Is `m` the same as `d` or nested inside it?
    pub fn inside(&self, m: ModuleId, d: ModuleId) -> bool {
        let mut cur = Some(m);
        while let Some(c) = cur {
            if c == d {
                return true;
            }
            cur = self.modules[c.0 as usize].parent;
        }
        false
    }

    fn add_module(&mut self, name: &str, parent: Option<ModuleId>, vis: Vis) -> ModuleId {
        let path = match parent {
            None => "limb".to_string(),
            Some(p) if p == ROOT => name.to_string(),
            Some(p) => format!("{}.{}", self.modules[p.0 as usize].path, name),
        };
        let id = ModuleId(self.modules.len() as u32);
        self.modules.push(Module { name: name.to_string(), path, parent, vis, items: HashMap::new(), consts: HashMap::new() });
        id
    }

    fn declare(&mut self, m: ModuleId, name: &str, r: ItemRef, vis: Vis, span: Span) -> Result<()> {
        let module = &mut self.modules[m.0 as usize];
        if let Some((prev, _, prev_span)) = module.items.get(name) {
            let prev_what = prev.what();
            let what = r.what();
            let msg = if prev_what == what {
                format!("`{}` is defined twice in {}: a {} of that name is already declared", name, describe_module(&module.path), what)
            } else if matches!((prev, r), (ItemRef::Module(_), _) | (_, ItemRef::Module(_))) {
                format!("`{}` is both a module and a {}: a module and a type cannot share a name — rename one", name, if prev_what == "module" { what } else { prev_what })
            } else {
                format!("`{}` is defined twice in {}: {} and {} cannot share a name", name, describe_module(&module.path), with_article(prev_what), with_article(what))
            };
            return err(Diagnostic::error(msg).at(span).why_at(*prev_span, format!("the first `{}` is declared here", name)));
        }
        module.items.insert(name.to_string(), (r, vis, span));
        Ok(())
    }

    /// Collect one parsed file as module `m`.
    pub fn collect_file(&mut self, file: &ast::File, m: ModuleId, is_root: bool) -> Result<()> {
        for item in &file.items {
            self.collect_item(item, m, is_root)?;
        }
        Ok(())
    }

    fn collect_item(&mut self, item: &Item, m: ModuleId, is_root: bool) -> Result<()> {
        let vis = item.vis;
        match &item.kind {
            ItemKind::Main(block) => {
                if !is_root {
                    return err(Diagnostic::error("`MAIN` lives at the root of the Limb (`limb.cpls`), not in a module").at(item.span));
                }
                if self.main.is_some() {
                    return err(Diagnostic::error("a program has one `MAIN`; this is the second").at(item.span));
                }
                self.main = Some((block.clone(), m, item.span));
            }
            ItemKind::Fn(decl) => {
                let id = FnId(self.fns.len() as u32);
                self.fns.push(placeholder_fn(&decl.name.name, m, vis, item.span, decl.name.span));
                self.declare(m, &decl.name.name, ItemRef::Fn(id), vis, decl.name.span)?;
                self.pending_fns.push((id, decl.clone(), m, None));
            }
            ItemKind::Op(op) => {
                let id = FnId(self.fns.len() as u32);
                self.fns.push(placeholder_fn(&op.op.name, m, vis, item.span, op.op.span));
                let decl = ast::FnDecl { name: op.op.clone(), params: op.params.clone(), ret: op.ret.clone(), fails: Vec::new(), body: Some(op.body.clone()) };
                self.pending_fns.push((id, decl, m, Some(op.op.name.clone())));
                self.ops.entry(op.op.name.clone()).or_default().push(id);
            }
            ItemKind::ExternFn(decl) => {
                let id = crate::hir::ExternId(self.externs.len() as u32);
                self.externs.push(ExternInfo { name: decl.name.name.clone(), params: Vec::new(), variadic: false, ret: Ty::Nothing, ranged: false, is_let: false, mutable: false, span: item.span });
                self.declare(m, &decl.name.name, ItemRef::ExternFn(id), vis, decl.name.span)?;
                self.pending_externs.push((id, decl.clone(), m));
            }
            ItemKind::ExternLet { mutable, name, ty } => {
                let id = crate::hir::ExternId(self.externs.len() as u32);
                self.externs.push(ExternInfo { name: name.name.clone(), params: Vec::new(), variadic: false, ret: Ty::Nothing, ranged: false, is_let: true, mutable: *mutable, span: item.span });
                // An extern variable is a value: it lives in the value namespace with the consts.
                let module = &mut self.modules[m.0 as usize];
                if module.consts.contains_key(&name.name) {
                    return err(Diagnostic::error(format!("'{}' is declared twice at the top level", name.name)).at(name.span));
                }
                module.items.insert(format!("'{}'", name.name), (ItemRef::ExternLet(id), vis, name.span));
                self.pending_extern_lets.push((id, ty.clone(), m));
            }
            ItemKind::Struct(decl) => {
                let id = StructId(self.structs.len() as u32);
                self.structs.push(StructInfo { name: decl.name.name.clone(), path: qualify(self, m, &decl.name.name), layout_c: decl.layout_c, fields: Vec::new(), blanks: Vec::new(), span: decl.name.span });
                self.declare(m, &decl.name.name, ItemRef::Struct(id), vis, decl.name.span)?;
                self.pending_structs.push((id, decl.clone(), m));
            }
            ItemKind::Enum(decl) => {
                let id = EnumId(self.enums.len() as u32);
                self.enums.push(EnumInfo { name: decl.name.name.clone(), path: qualify(self, m, &decl.name.name), cases: Vec::new(), blanks: Vec::new(), span: decl.name.span });
                self.declare(m, &decl.name.name, ItemRef::Enum(id), vis, decl.name.span)?;
                self.pending_enums.push((id, decl.clone(), m));
            }
            ItemKind::Interface(decl) => {
                let id = IfaceId(self.ifaces.len() as u32);
                self.ifaces.push(IfaceInfo { name: decl.name.name.clone(), path: qualify(self, m, &decl.name.name), entries: Vec::new(), span: decl.name.span });
                self.declare(m, &decl.name.name, ItemRef::Iface(id), vis, decl.name.span)?;
                self.pending_ifaces.push((id, decl.clone(), m));
            }
            ItemKind::Const(decl) => {
                let id = crate::hir::ConstId(self.consts.len() as u32);
                self.consts.push(ConstInfo { name: decl.name.name.clone(), module: m, vis, ty: None, value: decl.value.clone(), span: decl.name.span });
                let module = &mut self.modules[m.0 as usize];
                if module.consts.contains_key(&decl.name.name) {
                    return err(Diagnostic::error(format!("'{}' is declared twice at the top level", decl.name.name)).at(decl.name.span));
                }
                module.consts.insert(decl.name.name.clone(), (id, vis, decl.name.span));
                self.pending_consts.push((id, decl.ty.clone(), m));
            }
            ItemKind::Module { name, items } => {
                let child = self.add_module(&name.name, Some(m), vis);
                self.declare(m, &name.name, ItemRef::Module(child), vis, name.span)?;
                for it in items {
                    self.collect_item(it, child, false)?;
                }
            }
            ItemKind::Satisfies { ty, iface } => {
                self.pending_claims.push((ty.clone(), iface.clone(), m, item.span));
            }
        }
        Ok(())
    }

    /// Resolve every declared signature and shape, now that all names exist.
    pub fn resolve_all(&mut self) -> Result<()> {
        let structs = std::mem::take(&mut self.pending_structs);
        for (id, decl, m) in &structs {
            let mut blanks = Vec::new();
            let mut fields = Vec::new();
            for f in &decl.fields {
                let ty = self.resolve_type(&f.ty, *m, &mut blanks, true)?;
                if fields.iter().any(|x: &FieldInfo| x.name == f.name.name) {
                    return err(Diagnostic::error(format!("field `{}` is declared twice in `{}`", f.name.name, decl.name.name)).at(f.name.span));
                }
                if decl.layout_c && !ty.is_c_shaped(&self.structs) {
                    return err(Diagnostic::error(format!("field `{}` of the `layout.c` struct `{}` is not C-shaped: only numbers, `bool`, `ptr[…]` and other `layout.c` structs can cross to C", f.name.name, decl.name.name)).at(f.ty.span));
                }
                fields.push(FieldInfo { name: f.name.name.clone(), immut: f.immut, ty });
            }
            let s = &mut self.structs[id.0 as usize];
            s.fields = fields;
            s.blanks = blanks.into_iter().map(|(n, _)| n).collect();
        }
        let enums = std::mem::take(&mut self.pending_enums);
        for (id, decl, m) in &enums {
            let mut blanks = Vec::new();
            let mut cases = Vec::new();
            for c in &decl.cases {
                let payload = match &c.payload {
                    Some(t) => Some(self.resolve_type(t, *m, &mut blanks, true)?),
                    None => None,
                };
                if cases.iter().any(|x: &CaseInfo| x.name == c.name.name) {
                    return err(Diagnostic::error(format!("case `{}` is declared twice in `{}`", c.name.name, decl.name.name)).at(c.name.span));
                }
                cases.push(CaseInfo { name: c.name.name.clone(), payload });
            }
            let e = &mut self.enums[id.0 as usize];
            e.cases = cases;
            e.blanks = blanks.into_iter().map(|(n, _)| n).collect();
        }
        let ifaces = std::mem::take(&mut self.pending_ifaces);
        for (id, decl, m) in &ifaces {
            let mut entries = Vec::new();
            for e in &decl.entries {
                let mut blanks = Vec::new();
                let mut params = Vec::new();
                for p in &e.params {
                    let ty = match &p.ty {
                        Some(t) => self.resolve_type(t, *m, &mut blanks, true)?,
                        None => return err(Diagnostic::error("a parameter in an interface needs its type").at(p.span)),
                    };
                    params.push((conv_kind(p.kind), ty));
                }
                let ret = match &e.ret {
                    Some(t) => self.resolve_type(t, *m, &mut blanks, true)?,
                    None => Ty::Nothing,
                };
                let fails = self.resolve_fails(&e.fails, *m, &mut blanks)?;
                let (is_op, name) = match &e.name {
                    ast::IfaceEntryName::Fn(n) => (false, n.name.clone()),
                    ast::IfaceEntryName::Op(n) => (true, n.name.clone()),
                };
                entries.push(IfaceEntryInfo { is_op, name, params, ret, fails });
            }
            self.ifaces[id.0 as usize].entries = entries;
        }
        let fns = std::mem::take(&mut self.pending_fns);
        for (id, decl, m, op) in &fns {
            let info = self.resolve_fn_decl(decl, *m, op.clone(), self.fns[id.0 as usize].vis, self.fns[id.0 as usize].span)?;
            self.fns[id.0 as usize] = info;
        }
        let externs = std::mem::take(&mut self.pending_externs);
        for (id, decl, m) in &externs {
            let mut params = Vec::new();
            let mut variadic = false;
            let mut blanks = Vec::new();
            for p in &decl.params {
                if p.variadic {
                    variadic = true;
                    continue;
                }
                let Some(t) = &p.ty else {
                    return err(Diagnostic::error(format!("the `extern fn {}` parameter '{}' needs a type: C's signature is written in full", decl.name.name, p.name.name)).at(p.span));
                };
                let ty = self.resolve_type(t, *m, &mut blanks, false)?;
                if !blanks.is_empty() {
                    return err(Diagnostic::error("a blank `<T>` cannot cross to C: an `extern fn` is written in C's own types").at(t.span));
                }
                if ty.bound().is_some() {
                    return err(Diagnostic::error(format!("a range type cannot cross to C: '{}' is written as the plain `{}`", p.name.name, spell(&ty.base().clone(), self))).at(t.span));
                }
                self.check_c_shaped(&ty, t.span, &format!("the `extern fn {}` parameter '{}'", decl.name.name, p.name.name))?;
                params.push((p.name.name.clone(), ty));
            }
            let (ret, ranged) = match &decl.ret {
                Some(t) => {
                    let ty = self.resolve_type(t, *m, &mut blanks, false)?;
                    if !blanks.is_empty() {
                        return err(Diagnostic::error("a blank `<T>` cannot cross to C: an `extern fn` is written in C's own types").at(t.span));
                    }
                    let ranged = ty.bound().is_some();
                    self.check_c_shaped(ty.base(), t.span, &format!("the `extern fn {}` return", decl.name.name))?;
                    (ty, ranged)
                }
                None => (Ty::Nothing, false),
            };
            if !decl.fails.is_empty() {
                return err(Diagnostic::error(format!("an `extern fn` cannot `fail`: C has no failures — `{}` returns a plain value, and a wrapper turns a bad one into a failure", decl.name.name)).at(decl.fails[0].name.span));
            }
            let e = &mut self.externs[id.0 as usize];
            e.params = params;
            e.variadic = variadic;
            e.ret = ret;
            e.ranged = ranged;
        }
        let lets = std::mem::take(&mut self.pending_extern_lets);
        for (id, ty, m) in &lets {
            let mut blanks = Vec::new();
            let t = self.resolve_type(ty, *m, &mut blanks, false)?;
            self.check_c_shaped(&t, ty.span, &format!("the `extern let '{}'`", self.externs[id.0 as usize].name))?;
            self.externs[id.0 as usize].ret = t;
        }
        let consts = std::mem::take(&mut self.pending_consts);
        for (id, ty, m) in &consts {
            if let Some(t) = ty {
                let mut blanks = Vec::new();
                let rt = self.resolve_type(t, *m, &mut blanks, false)?;
                self.consts[id.0 as usize].ty = Some(rt);
            }
        }
        let claims = std::mem::take(&mut self.pending_claims);
        for (ty, iface, m, span) in &claims {
            let mut blanks = Vec::new();
            let t = self.resolve_type(ty, *m, &mut blanks, false)?;
            let i = match self.lookup_path(iface, *m)? {
                ItemRef::Iface(i) => i,
                other => return err(Diagnostic::error(format!("`{}` is a {}, not an interface", iface.spelled(), other.what())).at(iface.span)),
            };
            self.claims.push((t, i, *span));
        }
        Ok(())
    }

    fn check_c_shaped(&self, ty: &Ty, span: Span, what: &str) -> Result<()> {
        if ty.is_c_shaped(&self.structs) {
            return Ok(());
        }
        let why = match ty.base() {
            Ty::Struct { id, .. } => format!("`{}` is not `layout.c`: only a `layout.c struct` has C's shape", self.structs[id.0 as usize].name),
            Ty::Str(_) => "a `str` is not C-shaped: C sees bytes, so hand it a `ptr[uint8]` (a lend of the text)".to_string(),
            Ty::Seq { .. } => "a list is not C-shaped: C sees a pointer to the elements — `ptr[T]` and a length".to_string(),
            Ty::Fn(_) => "the callback is not C-shaped: parameters by value (no `lent`), C-shaped types, no `fail`".to_string(),
            _ => format!("`{}` is not C-shaped: only numbers, `bool`, `ptr[…]` and `layout.c` structs cross to C", spell(ty, self)),
        };
        err(Diagnostic::error(format!("{} is not C-shaped", what)).at(span).why(why))
    }

    fn resolve_fn_decl(&mut self, decl: &ast::FnDecl, m: ModuleId, op: Option<String>, vis: Vis, span: Span) -> Result<FnDeclInfo> {
        let mut blanks: Vec<(String, Option<IfaceId>)> = Vec::new();
        let mut params = Vec::new();
        for p in &decl.params {
            let (ty, from_blank) = match &p.ty {
                Some(t) => {
                    let rt = self.resolve_type(t, m, &mut blanks, true)?;
                    let fb = matches!(t.kind, ast::TypeKind::Var { .. });
                    (Some(rt), fb)
                }
                None => {
                    // An untyped parameter is an implicit blank named after the parameter.
                    (None, true)
                }
            };
            params.push(ParamDecl { kind: conv_kind(p.kind), amp: p.amp, mutability: p.mutability, variadic: p.variadic, name: p.name.name.clone(), ty, from_blank, span: p.span });
        }
        let ret = match &decl.ret {
            Some(t) => self.resolve_type(t, m, &mut blanks, true)?,
            None => Ty::Nothing,
        };
        let fails = self.resolve_fails(&decl.fails, m, &mut blanks)?;
        if let Some(o) = &op {
            if params.len() != 2 {
                return err(Diagnostic::error(format!("an operator takes exactly two operands; `{}` is declared with {}", o, params.len())).at(decl.name.span));
            }
            let language_op = matches!(o.as_str(), "+" | "-" | "x" | "xx" | "/" | "mod" | "==" | "!==" | "<" | ">" | "<==" | ">==" | "++" | "and" | "or" | "not" | "xor" | "shl" | "shr" | "+~" | "-~" | "x~" | "xx~");
            if matches!(o.as_str(), "and" | "or" | "not") {
                return err(Diagnostic::error(format!("`{}` cannot be redefined: `not`, `and` and `or` stay the language's", o)).at(decl.name.span));
            }
            if language_op {
                if let Some(Some(t)) = params.first().map(|p| &p.ty) {
                    if !matches!(t.base(), Ty::Struct { .. } | Ty::Enum { .. } | Ty::Var(_)) {
                        return err(Diagnostic::error(format!("`{}` is the language's operator, and it is never changed for the language's own types: a struct or enum may define `op {}` for itself, `{}` may not", o, o, spell(t, self))).at(decl.name.span));
                    }
                }
            }
        }
        let path = if op.is_some() { decl.name.name.clone() } else { qualify(self, m, &decl.name.name) };
        Ok(FnDeclInfo { name: decl.name.name.clone(), path, module: m, vis, params, ret, fails, blanks, body: decl.body.clone(), op, is_literal: false, span, name_span: decl.name.span })
    }

    pub fn resolve_fails(&mut self, fails: &[ast::FailSpec], m: ModuleId, blanks: &mut Vec<(String, Option<IfaceId>)>) -> Result<Vec<(String, Option<Ty>)>> {
        let mut out: Vec<(String, Option<Ty>)> = Vec::new();
        for f in fails {
            if out.iter().any(|(n, _)| *n == f.name.name) {
                return err(Diagnostic::error(format!("the failure `{}` is listed twice", f.name.name)).at(f.name.span));
            }
            let payload = match &f.payload {
                Some(t) => Some(self.resolve_type(t, m, blanks, true)?),
                None => None,
            };
            out.push((f.name.name.clone(), payload));
        }
        Ok(out)
    }

    /// Look up the first segment of a path in module `m`, then walk.
    pub fn lookup_path(&self, path: &ast::Path, m: ModuleId) -> Result<ItemRef> {
        if let Some(limb) = &path.limb {
            if limb.name == "std" {
                return err(Diagnostic::error(format!("`{}` is not available yet: there is no `std` Limb for now — the base language comes first", path.spelled())).at(path.span));
            }
            return err(Diagnostic::error(format!("there is no Limb named `{}`: a Limb is a dependency listed in `Synthia.cplm`, and none is", limb.name)).at(limb.span));
        }
        let mut cur = m;
        let mut segs = path.segs.iter().peekable();
        let first = segs.peek().unwrap();
        if first.name == "limb" {
            segs.next();
            cur = ROOT;
            if segs.peek().is_none() {
                return err(Diagnostic::error("`limb` is followed by `.` and the root item it reaches: `limb.helper[…]`").at(path.span));
            }
        }
        let mut result = None;
        while let Some(seg) = segs.next() {
            let module = &self.modules[cur.0 as usize];
            let Some((item, vis, _)) = module.items.get(&seg.name) else {
                let kind = if segs.peek().is_none() { "item" } else { "module" };
                return err(Diagnostic::error(format!("there is no {} `{}` in {}", kind, seg.name, describe_module(&module.path))).at(seg.span).why(self.hint_for_missing(&seg.name, cur)));
            };
            self.check_visible(*item, *vis, cur, m, seg)?;
            match item {
                ItemRef::Module(child) if segs.peek().is_some() => {
                    cur = *child;
                }
                _ => {
                    if segs.peek().is_some() {
                        return err(Diagnostic::error(format!("`{}` is a {}, not a module: nothing comes after it in a path", seg.name, item.what())).at(seg.span));
                    }
                    result = Some(*item);
                }
            }
        }
        Ok(result.unwrap())
    }

    /// A const reached by path: `a.'k'`, `limb.'scale'`.
    pub fn lookup_const_path(&self, path: &ast::Path, name: &ast::QName, m: ModuleId) -> Result<crate::hir::ConstId> {
        if let Some(limb) = &path.limb {
            if limb.name == "std" {
                return err(Diagnostic::error("`std::` is not available yet: there is no `std` Limb for now").at(path.span));
            }
            return err(Diagnostic::error(format!("there is no Limb named `{}`", limb.name)).at(limb.span));
        }
        let target = if path.is(&["limb"]) {
            ROOT
        } else {
            match self.lookup_path(path, m)? {
                ItemRef::Module(t) => t,
                other => return err(Diagnostic::error(format!("`{}` is a {}, not a module: a const is reached through its module — `module.'name'`", path.spelled(), other.what())).at(path.span)),
            }
        };
        self.lookup_const_in(target, name, m)
    }

    pub fn lookup_const_in(&self, target: ModuleId, name: &ast::QName, from: ModuleId) -> Result<crate::hir::ConstId> {
        let module = &self.modules[target.0 as usize];
        let Some((id, vis, _)) = module.consts.get(&name.name) else {
            return err(Diagnostic::error(format!("there is no const '{}' in {}", name.name, describe_module(&module.path))).at(name.span));
        };
        if !self.visible(*vis, target, from) {
            return err(Diagnostic::error(format!("'{}' is private to {}: add `pub` (other Limbs too) or `pub.limb` (this Limb) where it is declared", name.name, describe_module(&module.path))).at(name.span));
        }
        Ok(*id)
    }

    fn visible(&self, vis: Vis, declared_in: ModuleId, from: ModuleId) -> bool {
        match vis {
            Vis::Pub | Vis::Limb => true,
            Vis::Private => self.inside(from, declared_in),
        }
    }

    fn check_visible(&self, item: ItemRef, vis: Vis, declared_in: ModuleId, from: ModuleId, seg: &ast::Ident) -> Result<()> {
        if self.visible(vis, declared_in, from) {
            return Ok(());
        }
        let where_ = describe_module(&self.modules[declared_in.0 as usize].path);
        let what = item.what();
        let full = match item {
            ItemRef::Module(c) => format!("module `{}`", self.modules[c.0 as usize].path),
            _ => format!("{} `{}`", what, seg.name),
        };
        err(Diagnostic::error(format!("{} is private to {}: it is visible there and in the modules nested inside it", full, where_))
            .at(seg.span)
            .fix(None, format!("write `pub.limb` before it to open it to the whole Limb, or `pub` for other Limbs too")))
    }

    fn hint_for_missing(&self, name: &str, m: ModuleId) -> String {
        // Is it somewhere else? Say where.
        for (i, module) in self.modules.iter().enumerate() {
            if module.items.contains_key(name) && ModuleId(i as u32) != m {
                let from_here = if i == 0 { format!("limb.{}", name) } else { format!("limb.{}.{}", module.path, name) };
                return format!("`{}` is declared in {} — from here that is `{}`; a bare name is this module's, a dotted path is relative to it", name, describe_module(&module.path), from_here);
            }
        }
        "a bare name is this module's own; another module's item is reached by its path, `limb.` starts at the root".to_string()
    }

    /// Resolve a written type in module `m`. `blanks` collects `<T>` by first
    /// appearance; `allow_blanks` is false where a blank cannot appear.
    pub fn resolve_type(&self, t: &ast::Type, m: ModuleId, blanks: &mut Vec<(String, Option<IfaceId>)>, allow_blanks: bool) -> Result<Ty> {
        match &t.kind {
            ast::TypeKind::Var { name, bound } => {
                if !allow_blanks {
                    return err(Diagnostic::error(format!("a blank `<{}>` cannot appear here", name.name)).at(t.span));
                }
                let bound_id = match bound {
                    Some(p) => match self.lookup_path(p, m) {
                        Ok(ItemRef::Iface(i)) => Some(i),
                        Ok(other) => return err(Diagnostic::error(format!("`{}` is a {}, not an interface: a bound names an interface — `<{}: Ordered>`", p.spelled(), other.what(), name.name)).at(p.span)),
                        Err(_) => return err(Diagnostic::error(format!("there is no interface named `{}`", p.spelled())).at(p.span)),
                    },
                    None => None,
                };
                if let Some(pos) = blanks.iter().position(|(n, _)| *n == name.name) {
                    if let Some(b) = bound_id {
                        match blanks[pos].1 {
                            Some(prev) if prev != b => {
                                return err(Diagnostic::error(format!("one blank, one interface: `<{}>` is already bound to `{}`, and cannot also be `{}`", name.name, self.ifaces[prev.0 as usize].name, self.ifaces[b.0 as usize].name)).at(t.span));
                            }
                            _ => blanks[pos].1 = Some(b),
                        }
                    }
                    Ok(Ty::Var(pos as u32))
                } else {
                    blanks.push((name.name.clone(), bound_id));
                    Ok(Ty::Var((blanks.len() - 1) as u32))
                }
            }
            ast::TypeKind::Fn { params, ret, fails } => {
                let mut ps = Vec::new();
                for p in params {
                    ps.push((conv_kind(p.kind), self.resolve_type(&p.ty, m, blanks, allow_blanks)?));
                }
                let r = match ret {
                    Some(r) => self.resolve_type(r, m, blanks, allow_blanks)?,
                    None => Ty::Nothing,
                };
                let mut fs = Vec::new();
                for f in fails {
                    let payload = match &f.payload {
                        Some(p) => Some(self.resolve_type(p, m, blanks, allow_blanks)?),
                        None => None,
                    };
                    fs.push((f.name.name.clone(), payload));
                }
                Ok(Ty::Fn(Box::new(FnTy { params: ps, ret: r, fails: fs })))
            }
            ast::TypeKind::Bound { inner, lo, hi, unit, span } => {
                let base = self.resolve_type(inner, m, blanks, allow_blanks)?;
                self.resolve_bound(base, lo, hi, unit.as_ref(), *span)
            }
            ast::TypeKind::Count(n) => err(Diagnostic::error(format!("`{}` is a number, not a type: only a `simd[T, N]` takes one, as its lane count", n)).at(t.span)),
            ast::TypeKind::Path { path, args } => self.resolve_type_path(path, args, m, blanks, allow_blanks, t.span),
        }
    }

    fn resolve_bound(&self, base: Ty, lo: &ast::Expr, hi: &ast::Expr, unit: Option<&ast::Ident>, span: Span) -> Result<Ty> {
        if base.bound().is_some() {
            return err(Diagnostic::error("a type takes one bound").at(span));
        }
        let bound = match &base {
            Ty::Int(_) => {
                if let Some(u) = unit {
                    return err(Diagnostic::error(format!("a number's bound has no unit; `{}` belongs on a text's length bound", u.name)).at(u.span));
                }
                let l = int_literal(lo).ok_or_else(|| Box::new(Diagnostic::error("an integer's bounds are integers: `int32[0, 100]`").at(lo.span)))?;
                let h = int_literal(hi).ok_or_else(|| Box::new(Diagnostic::error("an integer's bounds are integers: `int32[0, 100]`").at(hi.span)))?;
                if l > h {
                    return err(Diagnostic::error(format!("the low bound comes first: `[{}, {}]`", h, l)).at(span));
                }
                if let Ty::Int(IntW::Fixed { signed, bits }) = &base {
                    let (min, max) = int_range(*signed, *bits);
                    if l < min || h > max {
                        return err(Diagnostic::error(format!("the bound [{}, {}] does not fit {} (range {} to {})", l, h, spell(&base, self), min, max)).at(span));
                    }
                }
                Bound::Int { lo: l, hi: h }
            }
            Ty::Bin(_) => {
                if let Some(u) = unit {
                    return err(Diagnostic::error(format!("a number's bound has no unit; `{}` belongs on a text's length bound", u.name)).at(u.span));
                }
                let l = bin_literal(lo).ok_or_else(|| Box::new(Diagnostic::error("a bin's bounds are bins: `bin64[0.0, 1.0]` — an integer's bounds are integers, and the two do not mix").at(lo.span)))?;
                let h = bin_literal(hi).ok_or_else(|| Box::new(Diagnostic::error("a bin's bounds are bins: `bin64[0.0, 1.0]`").at(hi.span)))?;
                if l.parse::<f64>().unwrap_or(0.0) > h.parse::<f64>().unwrap_or(0.0) {
                    return err(Diagnostic::error(format!("the low bound comes first: `[{}, {}]`", h, l)).at(span));
                }
                Bound::Bin { lo: l, hi: h }
            }
            Ty::Str(_) => {
                let u = match unit {
                    Some(u) => match u.name.as_str() {
                        "bytes" | "byte" => Unit::Byte,
                        "points" | "point" => Unit::Point,
                        "graphemes" | "grapheme" => Unit::Grapheme,
                        other => return err(Diagnostic::error(format!("`{}` is not a unit of text length: `bytes`, `points` or `graphemes`", other)).at(u.span)),
                    },
                    None => return err(Diagnostic::error("a text's length bound must say which unit it counts — `bytes`, `points` or `graphemes` — after the high bound: `str.utf8[1, 40 graphemes]`; a text has three lengths and no single one").at(span)),
                };
                let (l, h) = len_bounds(lo, hi, span)?;
                Bound::Len { lo: l, hi: h, unit: Some(u) }
            }
            Ty::Seq { kind: SeqKind::Array, .. } => {
                return err(Diagnostic::error("a length bound belongs on a `list`: an `array` has one length, already known").at(span));
            }
            Ty::Seq { .. } | Ty::Map { .. } | Ty::Channel(_) => {
                if let Some(u) = unit {
                    return err(Diagnostic::error(format!("a count bound has no unit; `{}` belongs on a text's length bound", u.name)).at(u.span));
                }
                let (l, h) = len_bounds(lo, hi, span)?;
                Bound::Len { lo: l, hi: h, unit: None }
            }
            other => return err(Diagnostic::error(format!("`{}` takes no bound: bounds go on numbers (`int32[0, 100]`), text, lists, maps and channels", spell(other, self))).at(span)),
        };
        Ok(Ty::Bounded { inner: Box::new(base), bound })
    }

    fn resolve_type_path(&self, path: &ast::Path, args: &[ast::Type], m: ModuleId, blanks: &mut Vec<(String, Option<IfaceId>)>, allow_blanks: bool, span: Span) -> Result<Ty> {
        let head = path.head();
        let no_args = |what: &str| -> Result<()> {
            if !args.is_empty() {
                return err(Diagnostic::error(format!("`{}` takes no type parameters", what)).at(span));
            }
            Ok(())
        };
        if path.limb.is_none() {
            if let Some(t) = primitive(head) {
                if let Ty::Int(IntW::Fixed { bits: 128, .. }) = t {
                    // Every number Precog reasons about is held in 128
                    // bits signed, so a `uint128` has values it cannot
                    // write down and an `int128` has sums it cannot. A
                    // proof about a number the prover cannot represent
                    // would not be a proof, so the type is refused rather
                    // than carried badly. (`design/types.md` marks the
                    // 128-bit widths proposed, not decided.)
                    return err(Diagnostic::error(format!("CCA carries whole numbers up to 64 bits, and `{}` is 128", head))
                        .at(span)
                        .why("Precog holds every number it reasons about in 128 bits signed; a 128-bit whole number has values that will not fit there, and a proof it cannot represent is not a proof")
                        .why("`design/types.md` lists the 128-bit widths as proposed, not decided")
                        .try_(format!("`{}64`", if head.starts_with('u') { "uint" } else { "int" })));
                }
                if path.segs.len() > 1 {
                    return err(Diagnostic::error(format!("`{}` takes no narrowing: `{}` is already a complete type", path.spelled(), head)).at(span));
                }
                no_args(head)?;
                return Ok(t);
            }
            match head {
                "str" => {
                    no_args("str")?;
                    let Some(enc) = path.seg(1) else {
                        return err(Diagnostic::error("`str` is never bare: say the encoding — `str.utf8`, `str.utf16`, `str.utf32`, `str.ascii`").at(span));
                    };
                    let e = match enc {
                        "utf8" => Enc::Utf8,
                        "utf16" => Enc::Utf16,
                        "utf32" => Enc::Utf32,
                        "ascii" => Enc::Ascii,
                        other => return err(Diagnostic::error(format!("`str.{}` is not an encoding: `utf8`, `utf16`, `utf32` or `ascii`", other)).at(span)),
                    };
                    if path.segs.len() > 2 {
                        return err(Diagnostic::error(format!("`{}` narrows too far: `str.{}` is the whole type", path.spelled(), enc)).at(span));
                    }
                    return Ok(Ty::Str(e));
                }
                "char" => {
                    no_args("char")?;
                    let Some(unit) = path.seg(1) else {
                        return err(Diagnostic::error("`char` is never bare: say the unit — `char.point`, `char.grapheme`, `char.byte`").at(span));
                    };
                    let u = match unit {
                        "byte" => Unit::Byte,
                        "point" => Unit::Point,
                        "grapheme" => Unit::Grapheme,
                        other => return err(Diagnostic::error(format!("`char.{}` is not a unit: `byte`, `point` or `grapheme`", other)).at(span)),
                    };
                    return Ok(Ty::Char(u));
                }
                "list" | "array" => {
                    if path.segs.len() > 1 {
                        return err(Diagnostic::error(format!("the element type goes in brackets: `{}[int32]`, not `{}`", head, path.spelled())).at(span));
                    }
                    if args.len() != 1 {
                        return err(Diagnostic::error(format!("`{}` takes exactly one type parameter, the element type: `{}[int32]`", head, head)).at(span));
                    }
                    let elem = self.resolve_type(&args[0], m, blanks, allow_blanks)?;
                    let kind = if head == "list" { SeqKind::List } else { SeqKind::Array };
                    return Ok(Ty::Seq { elem: Box::new(elem), kind });
                }
                "map" => {
                    let order = match path.seg(1) {
                        None => MapOrder::Free(InferId(0)),
                        Some("ordered") => MapOrder::Ordered,
                        Some("unordered") => MapOrder::Unordered,
                        Some(other) => return err(Diagnostic::error(format!("`map.{}` is not a variant: `map` (Precog decides), `map.ordered` (walkable, insertion order) or `map.unordered` (never walked)", other)).at(span)),
                    };
                    if args.len() != 2 {
                        return err(Diagnostic::error("`map` takes two type parameters, the key and the value: `map[str.utf8, int32]`").at(span));
                    }
                    let key = self.resolve_type(&args[0], m, blanks, allow_blanks)?;
                    let val = self.resolve_type(&args[1], m, blanks, allow_blanks)?;
                    self.check_key_type(&key, args[0].span)?;
                    return Ok(Ty::Map { key: Box::new(key), val: Box::new(val), order });
                }
                "simd" => {
                    if path.segs.len() > 1 {
                        return err(Diagnostic::error(format!("`simd` takes its lane type and count in brackets: `simd[uint8, 16]`, not `{}`", path.spelled())).at(span));
                    }
                    if args.len() != 2 {
                        return err(Diagnostic::error("`simd` takes two things in brackets, the lane type and how many lanes: `simd[uint8, 16]`").at(span));
                    }
                    let elem = self.resolve_type(&args[0], m, blanks, allow_blanks)?;
                    let Some(lanes) = lane_count(&args[1]) else {
                        return err(Diagnostic::error("the second thing in a `simd` is how many lanes, written as a number: `simd[uint8, 16]`").at(args[1].span));
                    };
                    if simd_lane_bits(&elem).is_none() {
                        return err(Diagnostic::error(format!("a `simd` holds plain numbers or bools, and `{}` is neither: its lanes live in a register, so each one is a fixed-width number", spell(&elem, self))).at(args[0].span));
                    }
                    if !simd_shape_ok(&elem, lanes) {
                        let has = simd_shapes_for(&elem).unwrap_or(0);
                        return err(Diagnostic::error(format!("this target has no `simd[{}, {}]`: its vector register holds {} lanes of `{}`, not {}", spell(&elem, self), lanes, has, spell(&elem, self), lanes))
                            .at(span)
                            .why(format!("a `simd` is only ever a shape the machine has ({}-bit registers here); portability is the plain `for` loop, which Precog and LLVM vectorise by themselves", TARGET_SIMD_BITS))
                            .fix(None, format!("simd[{}, {}]", spell(&elem, self), has)));
                    }
                    return Ok(Ty::Simd { elem: Box::new(elem), lanes });
                }
                "channel" => {
                    if args.len() != 1 {
                        return err(Diagnostic::error("`channel` takes one type parameter: `channel[str.utf8]`").at(span));
                    }
                    let elem = self.resolve_type(&args[0], m, blanks, allow_blanks)?;
                    return Ok(Ty::Channel(Box::new(elem)));
                }
                "ptr" => {
                    let mutable = match path.seg(1) {
                        None => false,
                        Some("mut") => true,
                        Some(other) => return err(Diagnostic::error(format!("`ptr.{}` does not exist: `ptr[T]` (read-only) or `ptr.mut[T]`", other)).at(span)),
                    };
                    if args.len() != 1 {
                        return err(Diagnostic::error("`ptr` takes one type parameter: `ptr[uint8]`").at(span));
                    }
                    let elem = self.resolve_type(&args[0], m, blanks, allow_blanks)?;
                    if !elem.contains_var() && !elem.is_c_shaped(&self.structs) {
                        let why = match elem.base() {
                            Ty::Struct { id, .. } => format!("`{}` is not `layout.c`: only a `layout.c struct` has C's shape", self.structs[id.0 as usize].name),
                            other => format!("`{}` is not C-shaped", spell(other, self)),
                        };
                        return err(Diagnostic::error(format!("`ptr[{}]` cannot exist: a pointer points at something C-shaped", spell(&elem, self))).at(span).why(why));
                    }
                    return Ok(Ty::Ptr { mutable, elem: Box::new(elem) });
                }
                "duration" => return Ok(Ty::Duration),
                "fn" => unreachable!(),
                "uenr" => return err(Diagnostic::error("`uenr` is the standard Limb's type, `std::uenr`, and `std::` is not available yet").at(span)),
                _ => {}
            }
        }
        // A user type by path.
        match self.lookup_path(path, m)? {
            ItemRef::Struct(id) => {
                let info = &self.structs[id.0 as usize];
                let targs = self.type_args(args, &info.blanks, &info.name, m, blanks, allow_blanks, span)?;
                Ok(Ty::Struct { id, args: targs })
            }
            ItemRef::Enum(id) => {
                let info = &self.enums[id.0 as usize];
                let targs = self.type_args(args, &info.blanks, &info.name, m, blanks, allow_blanks, span)?;
                Ok(Ty::Enum { id, args: targs })
            }
            other => err(Diagnostic::error(format!("`{}` is a {}, not a type", path.spelled(), other.what())).at(span)),
        }
    }

    fn type_args(&self, args: &[ast::Type], declared: &[String], name: &str, m: ModuleId, blanks: &mut Vec<(String, Option<IfaceId>)>, allow_blanks: bool, span: Span) -> Result<Vec<Ty>> {
        if args.is_empty() && !declared.is_empty() {
            // Left to inference where a value fills the blanks (`let 'm' := Maybe.some[42]`).
            return Ok(declared.iter().map(|_| Ty::Infer(InferId(u32::MAX))).collect());
        }
        if args.len() != declared.len() {
            return err(Diagnostic::error(format!("`{}` has {} blank(s) ({}), and {} type(s) were given", name, declared.len(), declared.iter().map(|b| format!("<{}>", b)).collect::<Vec<_>>().join(", "), args.len())).at(span));
        }
        let mut out = Vec::new();
        for a in args {
            out.push(self.resolve_type(a, m, blanks, allow_blanks)?);
        }
        Ok(out)
    }

    pub fn check_key_type(&self, key: &Ty, span: Span) -> Result<()> {
        match key.base() {
            Ty::Fn(_) => err(Diagnostic::error("a function cannot be a map key: keys are plain values, text, chars, structs, enums and lists of keys").at(span)),
            Ty::Ptr { .. } => err(Diagnostic::error("a pointer cannot be a map key: keys are plain values, text, chars, structs, enums and lists of keys").at(span)),
            Ty::Map { .. } => err(Diagnostic::error("a map cannot be a map key: keys are plain values, text, chars, structs, enums and lists of keys").at(span)),
            Ty::Channel(_) => err(Diagnostic::error("a channel cannot be a map key").at(span)),
            Ty::Seq { elem, .. } => self.check_key_type(elem, span),
            _ => Ok(()),
        }
    }
}

pub fn conv_kind(k: ast::ParamKind) -> ParamKind {
    match k {
        ast::ParamKind::Owned => ParamKind::Owned,
        ast::ParamKind::Lent => ParamKind::Lent,
        ast::ParamKind::LentMut => ParamKind::LentMut,
    }
}

fn placeholder_fn(name: &str, m: ModuleId, vis: Vis, span: Span, name_span: Span) -> FnDeclInfo {
    FnDeclInfo { name: name.to_string(), path: name.to_string(), module: m, vis, params: Vec::new(), ret: Ty::Nothing, fails: Vec::new(), blanks: Vec::new(), body: None, op: None, is_literal: false, span, name_span }
}

fn qualify(items: &Items, m: ModuleId, name: &str) -> String {
    if m == ROOT {
        name.to_string()
    } else {
        format!("{}.{}", items.modules[m.0 as usize].path, name)
    }
}

pub fn describe_module(path: &str) -> String {
    if path == "limb" {
        "the root (`limb.cpls`)".to_string()
    } else {
        format!("module `{}`", path)
    }
}

pub fn spell(t: &Ty, items: &Items) -> String {
    TySpelling { ty: t, structs: &items.structs, enums: &items.enums }.to_string()
}

pub fn primitive(head: &str) -> Option<Ty> {
    if head == "bool" {
        return Some(Ty::Bool);
    }
    let (signed, prefix) = if head.starts_with("uint") { (false, 4) } else if head.starts_with("int") { (true, 3) } else if head.starts_with("bin") {
        let bits: u16 = head[3..].parse().ok()?;
        if matches!(bits, 16 | 32 | 64 | 128) {
            return Some(Ty::Bin(BinW::Fixed(bits)));
        }
        return None;
    } else if head.starts_with("deci") {
        let bits: u16 = head[4..].parse().ok()?;
        if matches!(bits, 32 | 64 | 128) {
            return Some(Ty::Deci(bits));
        }
        return None;
    } else {
        return None;
    };
    let bits: u16 = head[prefix..].parse().ok()?;
    if matches!(bits, 8 | 16 | 32 | 64 | 128) {
        Some(Ty::int(signed, bits))
    } else {
        None
    }
}

/// What a whole number of this width holds. Everything here is held in
/// `i128`, so 128 bits is the ceiling and the widths above it are not
/// carried — `resolve_type_path` refuses them before anything asks.
pub fn int_range(signed: bool, bits: u16) -> (i128, i128) {
    if signed {
        if bits >= 128 {
            return (i128::MIN, i128::MAX);
        }
        let max = (1i128 << (bits - 1)) - 1;
        (-max - 1, max)
    } else {
        if bits >= 127 {
            return (0, i128::MAX);
        }
        (0, (1i128 << bits) - 1)
    }
}

pub fn int_literal(e: &ast::Expr) -> Option<i128> {
    match &e.kind {
        ast::ExprKind::Int(t) => t.parse().ok(),
        ast::ExprKind::Unary { op: ast::UnOp::Neg, expr } => int_literal(expr).map(|v| -v),
        ast::ExprKind::Paren(inner) => int_literal(inner),
        _ => None,
    }
}

pub fn bin_literal(e: &ast::Expr) -> Option<String> {
    match &e.kind {
        ast::ExprKind::Bin(t) => Some(t.clone()),
        ast::ExprKind::Unary { op: ast::UnOp::Neg, expr } => bin_literal(expr).map(|v| format!("-{}", v)),
        ast::ExprKind::Paren(inner) => bin_literal(inner),
        _ => None,
    }
}

fn len_bounds(lo: &ast::Expr, hi: &ast::Expr, span: Span) -> Result<(u64, u64)> {
    let l = int_literal(lo).ok_or_else(|| Box::new(Diagnostic::error("a length bound's ends are whole numbers").at(lo.span)))?;
    let h = int_literal(hi).ok_or_else(|| Box::new(Diagnostic::error("a length bound's ends are whole numbers").at(hi.span)))?;
    if l < 0 {
        return err(Diagnostic::error("a length bound's low end is 0 or more: a length is never negative").at(lo.span));
    }
    if l > h {
        return err(Diagnostic::error(format!("the low bound comes first: `[{}, {}]`", h, l)).at(span));
    }
    Ok((l as u64, h as u64))
}

pub fn with_article(s: &str) -> String {
    if s.starts_with(['a', 'e', 'i', 'o', 'u']) {
        format!("an {}", s)
    } else {
        format!("a {}", s)
    }
}

/// The lane count written in a `simd[T, N]`.
fn lane_count(t: &ast::Type) -> Option<u32> {
    match &t.kind {
        ast::TypeKind::Count(n) => Some(*n),
        _ => None,
    }
}
