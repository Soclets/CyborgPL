//! Item collection: the module tree of a Kit, every declared function,
//! struct, enum, interface, operator, const and extern, and how a path
//! written in one module reaches an item in another (`kits.md`).

use std::collections::{HashMap, HashSet};

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

/// What the power ledger needs of one Kit.
#[derive(Debug, Clone, Default)]
pub struct KitPowers {
    pub prefix: String,
    /// The program's own Kit: "this Kit" in a message.
    pub is_program: bool,
    pub lists: Vec<kayo_base::powers::Power>,
    pub cores: Vec<String>,
}

/// Where a bare path leads (owner, 2026-10-06).
#[derive(Debug, Clone)]
pub enum Bareness {
    Own(ItemRef),
    /// Another Kit's, by its prefix.
    Dep(String, ItemRef),
    /// One of std's names.
    Std,
}

/// How a board's output takes `cord::print`'s bytes (owner, 2026-10-09:
/// both).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputForm {
    /// `['b': uint8]`: one a call.
    Byte,
    /// `[lent 'bytes': array[uint8]]`: a chunk a call.
    Chunk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// `kit`, `shapes`, `shapes.circle`.
    pub path: String,
    pub parent: Option<ModuleId>,
    pub vis: Vis,
    pub items: HashMap<String, (ItemRef, Vis, Span)>,
    /// Consts live in the value namespace, keyed by their quoted name.
    pub consts: HashMap<String, (crate::hir::ConstId, Vis, Span)>,
    /// The `case.any` names, folded, and as declared.
    pub folded: HashMap<String, String>,
    pub folded_consts: HashMap<String, String>,
}

impl Items {
    /// A field or case name as written against the one declared: the same,
    /// or the same folded where its struct or enum (by its name's span) is
    /// `case.any` for them (owner, 2026-10-06).
    pub fn same_name(&self, set: &HashSet<Span>, owner: Span, declared: &str, written: &str) -> bool {
        declared == written || (set.contains(&owner) && ast::fold_case(declared) == ast::fold_case(written))
    }
}

impl Module {
    /// An item by its name, or by the folded name of an insensitive one.
    pub fn item(&self, name: &str) -> Option<&(ItemRef, Vis, Span)> {
        self.items.get(name).or_else(|| self.folded.get(&ast::fold_case(name)).and_then(|n| self.items.get(n)))
    }
    pub fn const_named(&self, name: &str) -> Option<&(crate::hir::ConstId, Vis, Span)> {
        self.consts.get(name).or_else(|| self.folded_consts.get(&ast::fold_case(name)).and_then(|n| self.consts.get(n)))
    }
}

/// A parameter as declared, before instantiation.
#[derive(Debug, Clone)]
pub struct ParamDecl {
    pub kind: ParamKind,
    /// `lent@x`: its lifetime group (owner, 2026-09-29).
    pub group: Option<String>,
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
    /// `-> lent@x T`: a borrow handed back, whether it may be changed, and
    /// its group (owner, 2026-09-29).
    pub ret_lent: Option<(bool, Option<String>)>,
    pub fails: Vec<(String, Option<Ty>)>,
    /// Generic blanks by first appearance: name and interface bound.
    pub blanks: Vec<(String, Vec<IfaceId>)>,
    pub body: Option<ast::Block>,
    /// `op` declarations: the operator's spelling.
    pub op: Option<String>,
    /// An anonymous `fn` literal (captures are resolved by the checker).
    pub is_literal: bool,
    /// `test fn` (owner, 2026-09-29).
    pub test: bool,
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
    /// A C type sharing its name with a C function (owner, 2026-10-10:
    /// `struct stat` and `stat`): the name calls the function, and where a
    /// type stands it is the type.
    pub c_fn_types: HashMap<crate::hir::ExternId, ItemRef>,
    /// Where a `must.stop`'s expression is (owner, 2026-09-29): a check
    /// site there must fire, so one failing every time is not refused.
    pub must_stop_spans: Vec<Span>,
    /// Each Kit's dependencies, by the name its `CCA.cplr` gives them
    /// (owner, 2026-09-29): its root module to theirs. `areas::circle`
    /// starts at the root this map gives `areas` from the Kit it is in.
    pub kit_deps: HashMap<ModuleId, HashMap<String, ModuleId>>,
    /// Somewhere a C door that is checked everywhere else is trusted under
    /// `NO_FUCKING_CHECKS`: a range type may then hold what it does not
    /// allow, and no range is a fact the code generator may hand LLVM.
    pub trusted_door: bool,
    /// Files marked `#strict` (owner, 2026-09-27).
    pub strict_files: std::collections::HashSet<crate::source::FileId>,
    /// Each pub thing's `bare` mark, by its name's span (owner,
    /// 2026-10-06).
    pub bare_marks: HashMap<Span, crate::ast::Bare>,
    /// `case.any` (owner, 2026-10-06), worked out where each file
    /// is read: the names that match in any case, by their name's span;
    /// the structs whose fields and the enums whose cases do; the
    /// functions (and `MAIN`, by its span) whose variables do.
    pub case_names: HashSet<Span>,
    pub case_fields: HashSet<Span>,
    pub case_cases: HashSet<Span>,
    pub case_vars: HashSet<Span>,
    /// The roots of Kits whose `CCA.cplr` does not list stdkit: std's names
    /// are not theirs to write bare.
    pub no_std_roots: std::collections::HashSet<ModuleId>,
    /// The Nerve cord's root, when a Neuron of the program lists it (owner,
    /// 2026-10-14): `cord::` looks there first, then at the built-ins.
    pub nerve_cord: Option<ModuleId>,
    /// The roots of Kits whose `CCA.cplr` does not list IEG: the rule
    /// that marks raw memory and unproven code `!check` is IEG's, so
    /// nothing there needs the mark (owner, 2026-10-06: "yes").
    /// Each Kit with a `CCA.cplr`, by its root, for the power ledger
    /// (owner, 2026-10-07); a lone file has none, and every power.
    pub kit_powers: HashMap<ModuleId, KitPowers>,
    /// Each Neuron's folder, by its root module, where an `incl` path
    /// starts (owner, 2026-10-10); a lone file's own folder.
    pub neuron_dirs: HashMap<ModuleId, std::path::PathBuf>,
    /// The program's own `deny`.
    pub program_deny: Vec<kayo_base::powers::Deny>,
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
    pub main: Option<(ast::Block, ModuleId, Span, Vec<ast::Param>)>,
    /// Pending struct/enum/iface bodies to resolve after every name is known.
    pending_structs: Vec<(StructId, ast::StructDecl, ModuleId)>,
    pending_enums: Vec<(EnumId, ast::EnumDecl, ModuleId)>,
    pending_ifaces: Vec<(IfaceId, ast::IfaceDecl, ModuleId)>,
    pending_fns: Vec<(FnId, ast::FnDecl, ModuleId, Option<String>)>,
    pending_externs: Vec<(crate::hir::ExternId, ast::FnDecl, ModuleId)>,
    pending_extern_lets: Vec<(crate::hir::ExternId, ast::Type, ModuleId)>,
    pending_consts: Vec<(crate::hir::ConstId, Option<ast::Type>, ModuleId)>,
    pending_claims: Vec<(ast::Type, ast::Path, ModuleId, Span)>,
    /// `fn Counter.next`: each function's type, as written, where it is.
    pending_type_fns: Vec<(FnId, ast::Ident, ModuleId)>,
    /// The functions belonging to a type (owner, 2026-10-09: "`b`"), by
    /// the type and the name after the dot.
    pub type_fns: HashMap<(ItemRef, String), FnId>,
    /// The Brainstem's `optional` (owner, 2026-10-08), in every program.
    pub base_optional: Option<EnumId>,
    /// With no OS, the board's output and exit the target names (owner,
    /// 2026-10-09).
    pub board_output: Option<(FnId, OutputForm)>,
    pub board_exit: Option<FnId>,
}

pub const ROOT: ModuleId = ModuleId(0);

impl Items {
    pub fn module_path(&self, m: ModuleId) -> &str {
        &self.modules[m.0 as usize].path
    }

    /// Is `m` the same as `d` or nested inside it?
    /// The root of the Kit module `m` is in: each Kit is its own tree.
    pub fn kit_root(&self, m: ModuleId) -> ModuleId {
        let mut cur = m;
        while let Some(p) = self.modules[cur.0 as usize].parent {
            cur = p;
        }
        cur
    }

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
        let path = module_path(self, name, parent);
        let id = ModuleId(self.modules.len() as u32);
        self.modules.push(Module { name: name.to_string(), path, parent, vis, items: HashMap::new(), consts: HashMap::new(), folded: HashMap::new(), folded_consts: HashMap::new() });
        id
    }

    fn declare(&mut self, m: ModuleId, name: &str, r: ItemRef, vis: Vis, span: Span) -> Result<()> {
        // A C type and a C function may share a name (owner, 2026-10-10).
        let c_type = |i: &ItemRef, items: &Items| match i {
            ItemRef::Struct(id) => items.structs[id.0 as usize].layout_c,
            ItemRef::Enum(id) => items.enums[id.0 as usize].numbers.is_some(),
            _ => false,
        };
        if let Some((prev, _, _)) = self.modules[m.0 as usize].items.get(name).cloned() {
            match (prev, r) {
                (ItemRef::ExternFn(f), t) if c_type(&t, self) && !self.c_fn_types.contains_key(&f) => {
                    self.c_fn_types.insert(f, t);
                    return Ok(());
                }
                (t, ItemRef::ExternFn(f)) if c_type(&t, self) => {
                    self.c_fn_types.insert(f, t);
                    self.modules[m.0 as usize].items.insert(name.to_string(), (r, vis, span));
                    return Ok(());
                }
                _ => {}
            }
        }
        let module = &mut self.modules[m.0 as usize];
        if let Some((prev, _, prev_span)) = module.items.get(name) {
            let prev_what = prev.what();
            let what = r.what();
            let msg = if prev_what == what {
                format!("`{}` is defined twice in {}: {} of that name is already declared", name, describe_module(&module.path), with_article(what))
            } else if matches!((prev, r), (ItemRef::Module(_), _) | (_, ItemRef::Module(_))) {
                format!("`{}` is both a module and a {}: a module and a type cannot share a name — rename one", name, if prev_what == "module" { what } else { prev_what })
            } else {
                format!("`{}` is defined twice in {}: {} and {} cannot share a name", name, describe_module(&module.path), with_article(prev_what), with_article(what))
            };
            return err(Diagnostic::error(msg).at(span).why_at(*prev_span, format!("the first `{}` is declared here", name)));
        }
        module.items.insert(name.to_string(), (r, vis, span));
        if self.case_names.contains(&span) {
            module.folded.insert(ast::fold_case(name), name.to_string());
        }
        Ok(())
    }

    /// The Brainstem's own declarations (owner, 2026-10-08): `optional`,
    /// in a module of its own that every bare path falls back to.
    pub fn add_base(&mut self, file: &ast::File) -> Result<()> {
        let m = self.add_module("brainstem", None, Vis::Pub);
        for item in &file.items {
            if let ItemKind::Enum(decl) = &item.kind {
                let id = EnumId(self.enums.len() as u32);
                self.enums.push(EnumInfo { name: decl.name.name.clone(), path: decl.name.name.clone(), cases: Vec::new(), numbers: None, blanks: Vec::new(), bounds: Vec::new(), span: decl.name.span });
                self.declare(m, &decl.name.name, ItemRef::Enum(id), Vis::Pub, decl.name.span)?;
                self.pending_enums.push((id, decl.clone(), m));
                self.base_optional = Some(id);
            }
        }
        Ok(())
    }

    /// Collect one parsed file as module `m`.
    pub fn collect_file(&mut self, file: &ast::File, m: ModuleId, is_root: bool) -> Result<()> {
        if let Some(f) = file.strict {
            self.strict_files.insert(f);
        }
        for item in &file.items {
            self.collect_item(item, m, is_root)?;
        }
        Ok(())
    }

    fn collect_item(&mut self, item: &Item, m: ModuleId, is_root: bool) -> Result<()> {
        let vis = item.vis;
        match &item.kind {
            ItemKind::Main(params, block) => {
                if !is_root {
                    return err(Diagnostic::error("`MAIN` lives at the root of the kit (`kit.cy`), not in a module").at(item.span));
                }
                if self.main.is_some() {
                    return err(Diagnostic::error("a program has one `MAIN`; this is the second").at(item.span));
                }
                self.main = Some((block.clone(), m, item.span, params.clone()));
            }
            ItemKind::Fn(decl) if decl.owner.is_some() => {
                let owner = decl.owner.clone().unwrap();
                let id = FnId(self.fns.len() as u32);
                let name = format!("{}.{}", owner.name, decl.name.name);
                self.fns.push(placeholder_fn(&name, m, vis, item.span, decl.name.span));
                self.pending_type_fns.push((id, owner, m));
                self.pending_fns.push((id, decl.clone(), m, None));
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
                let decl = ast::FnDecl { name: op.op.clone(), announce: op.announce.clone(), params: op.params.clone(), ret: op.ret.clone(), ret_lent: None, fails: Vec::new(), body: Some(op.body.clone()), test: false, owner: None };
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
                if decl.name.name == "optional" {
                    return err(Diagnostic::error("`optional` is the base's: every program has `enum optional <T: any> { value: <T>, none }`; name this struct something else").at(decl.name.span));
                }
                let id = StructId(self.structs.len() as u32);
                self.structs.push(StructInfo { name: decl.name.name.clone(), path: qualify(self, m, &decl.name.name), layout_c: decl.layout_c, opaque: decl.opaque, fields: Vec::new(), blanks: Vec::new(), bounds: Vec::new(), span: decl.name.span });
                self.declare(m, &decl.name.name, ItemRef::Struct(id), vis, decl.name.span)?;
                self.pending_structs.push((id, decl.clone(), m));
            }
            ItemKind::Enum(decl) => {
                if decl.name.name == "optional" {
                    return err(Diagnostic::error("`optional` is the base's: every program has `enum optional <T: any> { value: <T>, none }`; name this enum something else").at(decl.name.span));
                }
                let id = EnumId(self.enums.len() as u32);
                self.enums.push(EnumInfo { name: decl.name.name.clone(), path: qualify(self, m, &decl.name.name), cases: Vec::new(), numbers: decl.layout_c.then(Vec::new), blanks: Vec::new(), bounds: Vec::new(), span: decl.name.span });
                self.declare(m, &decl.name.name, ItemRef::Enum(id), vis, decl.name.span)?;
                self.pending_enums.push((id, decl.clone(), m));
            }
            ItemKind::Interface(decl) => {
                let id = IfaceId(self.ifaces.len() as u32);
                self.ifaces.push(IfaceInfo { name: decl.name.name.clone(), path: qualify(self, m, &decl.name.name), entries: Vec::new(), types: Vec::new(), span: decl.name.span });
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
                if self.case_names.contains(&decl.name.span) {
                    module.folded_consts.insert(ast::fold_case(&decl.name.name), decl.name.name.clone());
                }
                self.pending_consts.push((id, decl.ty.clone(), m));
            }
            ItemKind::Module { name, items } => {
                let child = self.add_module(&name.name, Some(m), vis);
                self.declare(m, &name.name, ItemRef::Module(child), vis, name.span)?;
                for it in items {
                    self.collect_item(it, child, false)?;
                }
            }
            // Flattened as it is read; each line on its own if not.
            ItemKind::ExternBlock(lines) => {
                for (v, k) in lines {
                    let v = if vis == ast::Vis::Private { *v } else { vis };
                    self.collect_item(&Item { vis: v, kind: k.clone(), span: item.span, bare: item.bare, case: item.case }, m, is_root)?;
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
            let mut blanks = self.announced(&decl.announce, *m)?;
            let mut fields = Vec::new();
            for f in &decl.fields {
                let ty = self.resolve_type(&f.ty, *m, &mut blanks, true)?;
                if fields.iter().any(|x: &FieldInfo| x.name == f.name.name) {
                    return err(Diagnostic::error(format!("field `{}` is declared twice in `{}`", f.name.name, decl.name.name)).at(f.name.span));
                }
                // A range is Cyborg's promise, and C sees the plain number
                // (owner, 2026-09-27: a).
                if decl.layout_c && !ty.base().is_c_shaped(&self.structs, &self.enums) {
                    return err(Diagnostic::error(format!("field `{}` of the `layout.C` struct `{}` is not C-shaped: only numbers, `bool`, `ptr[…]` and other `layout.C` structs and enums can cross to C", f.name.name, decl.name.name)).at(f.ty.span));
                }
                fields.push(FieldInfo { name: f.name.name.clone(), immut: f.immut, ty });
            }
            let s = &mut self.structs[id.0 as usize];
            s.fields = fields;
            s.bounds = blanks.iter().map(|(_, b)| b.clone()).collect();
            s.blanks = blanks.into_iter().map(|(n, _)| n).collect();
        }
        let enums = std::mem::take(&mut self.pending_enums);
        for (id, decl, m) in &enums {
            let mut blanks = self.announced(&decl.announce, *m)?;
            let mut cases = Vec::new();
            let mut numbers: Vec<i64> = Vec::new();
            for c in &decl.cases {
                let payload = match &c.payload {
                    Some(t) => Some(self.resolve_type(t, *m, &mut blanks, true)?),
                    None => None,
                };
                if cases.iter().any(|x: &CaseInfo| x.name == c.name.name) {
                    return err(Diagnostic::error(format!("case `{}` is declared twice in `{}`", c.name.name, decl.name.name)).at(c.name.span));
                }
                if decl.layout_c {
                    // C's enum (owner, 2026-09-27: a): a number per case,
                    // each its own, written as the C header does.
                    if c.payload.is_some() {
                        return err(Diagnostic::error(format!("case `{}` of the C enum `{}` carries no data: C's enum is a number per case — `{} = 1`", c.name.name, decl.name.name, c.name.name)).at(c.name.span));
                    }
                    let Some((n, at)) = c.number else {
                        return err(Diagnostic::error(format!("case `{}` of the C enum `{}` says its number, as the C header does — `{} = 65`: guessing the order is how bindings go wrong", c.name.name, decl.name.name, c.name.name)).at(c.name.span));
                    };
                    if n < i32::MIN as i128 || n > i32::MAX as i128 {
                        return err(Diagnostic::error(format!("C's enum is 32 bits, C's `int`: {} does not fit int32", n)).at(at));
                    }
                    if let Some(j) = numbers.iter().position(|x| *x == n as i64) {
                        return err(Diagnostic::error(format!("`{}` and `{}` are both {}: each case has its own number", decl.cases[j].name.name, c.name.name, n)).at(at));
                    }
                    numbers.push(n as i64);
                } else if let Some((_, at)) = c.number {
                    return err(Diagnostic::error(format!("only a `layout.C enum`'s cases have numbers: `{}` is a Cyborg enum, whose cases are names", decl.name.name)).at(at));
                }
                cases.push(CaseInfo { name: c.name.name.clone(), payload });
            }
            let e = &mut self.enums[id.0 as usize];
            if decl.layout_c {
                e.numbers = Some(numbers);
            }
            e.cases = cases;
            e.bounds = blanks.iter().map(|(_, b)| b.clone()).collect();
            e.blanks = blanks.into_iter().map(|(n, _)| n).collect();
        }
        let ifaces = std::mem::take(&mut self.pending_ifaces);
        for (id, decl, m) in &ifaces {
            let mut entries = Vec::new();
            let mut types = Vec::new();
            for t in &decl.types {
                let mut none = Vec::new();
                types.push(self.resolve_type(t, *m, &mut none, false)?);
            }
            self.ifaces[id.0 as usize].types = types;
            for e in &decl.entries {
                let mut blanks: Vec<(String, Vec<IfaceId>)> = decl.own.iter().map(|t| (t.name.clone(), Vec::new())).collect();
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
        // A function belongs to a struct, an enum or a function of its own
        // Neuron (`fn cum.quick`, 2026-10-09, and a collection's members,
        // 2026-10-13), once per name; an enum's function is not named as a
        // case is.
        let type_fns = std::mem::take(&mut self.pending_type_fns);
        for (id, owner, m) in &type_fns {
            let path = ast::Path { kit: None, segs: vec![owner.clone()], span: owner.span };
            let ty = match self.lookup_path(&path, *m)? {
                t @ (ItemRef::Struct(_) | ItemRef::Enum(_) | ItemRef::Fn(_)) => t,
                other => return err(Diagnostic::error(format!("`{}` is a {}: a function belongs to a struct, an enum or a function, `fn {}.name […]` naming one", owner.name, other.what(), owner.name)).at(owner.span)),
            };
            let name = self.fns[id.0 as usize].name.clone();
            let after = name[owner.name.len() + 1..].to_string();
            if let ItemRef::Enum(e) = ty {
                if self.enums[e.0 as usize].cases.iter().any(|c| c.name == after) || self.pending_enums.iter().any(|(pe, d, _)| *pe == e && d.cases.iter().any(|c| c.name.name == after)) {
                    return err(Diagnostic::error(format!("`{}` is a case of `{}`, so `fn {}` cannot take its name", after, owner.name, name)).at(self.fns[id.0 as usize].name_span));
                }
            }
            if let Some(prev) = self.type_fns.get(&(ty, after.clone())) {
                return err(Diagnostic::error(format!("`fn {}` is defined twice", name)).at(self.fns[id.0 as usize].name_span).why_at(self.fns[prev.0 as usize].name_span, format!("the first `fn {}` is declared here", name)));
            }
            self.type_fns.insert((ty, after), *id);
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
                    return err(Diagnostic::error(format!("the `extern.C fn {}` parameter '{}' needs a type: C's signature is written in full", decl.name.name, p.name.name)).at(p.span));
                };
                let ty = self.resolve_type(t, *m, &mut blanks, false)?;
                if !blanks.is_empty() {
                    return err(Diagnostic::error("a blank `<T>` cannot cross to C: an `extern fn` is written in C's own types").at(t.span));
                }
                if ty.bound().is_some() {
                    return err(Diagnostic::error(format!("a range type cannot cross to C: '{}' is written as the plain `{}`", p.name.name, spell(&ty.base().clone(), self))).at(t.span));
                }
                self.check_c_shaped(&ty, t.span, &format!("the `extern.C fn {}` parameter '{}'", decl.name.name, p.name.name))?;
                // C calls a callback, so a number it sends that no case has
                // would have nowhere to fail (owner, 2026-09-27).
                if let Ty::Fn(f) = ty.base() {
                    if let Some((_, bad)) = f.params.iter().find(|(_, t)| self.checked_at_door(t)) {
                        let spelled = spell(bad, self);
                        return err(Diagnostic::error(format!("a callback's parameter cannot be `{}`: C calls it, so a number it sends that `{}` does not allow would have nowhere to fail", spelled, spelled)).at(t.span).fix(None, format!("take the plain number and convert inside: `to.{}[…] otherwise …`", spelled)));
                    }
                }
                params.push((p.name.name.clone(), ty));
            }
            let (ret, ranged) = match &decl.ret {
                Some(t) => {
                    let ty = self.resolve_type(t, *m, &mut blanks, false)?;
                    if !blanks.is_empty() {
                        return err(Diagnostic::error("a blank `<T>` cannot cross to C: an `extern fn` is written in C's own types").at(t.span));
                    }
                    let ranged = self.checked_at_door(&ty);
                    self.check_c_shaped(ty.base(), t.span, &format!("the `extern.C fn {}` return", decl.name.name))?;
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
            self.check_c_shaped(&t, ty.span, &format!("C's variable '{}'", self.externs[id.0 as usize].name))?;
            // Reading a variable never fails, so it holds C's plain number
            // (owner, 2026-09-28: b).
            if self.checked_at_door(&t) {
                let spelled = spell(&t, self);
                return err(Diagnostic::error(format!("a C variable cannot be `{}`: reading a variable never fails, so it holds C's plain number", spelled)).at(ty.span).fix(None, format!("declare it `int32` and convert: `to.{}[…] otherwise …`", spelled)));
            }
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

    /// What C hands back that Cyborg must check before holding it: a range
    /// (`ffi.md`, a ranged return), a C enum's number, which may match no
    /// case, and a `layout.C` struct with either among its fields (owner,
    /// 2026-09-27: a). Outside, the call fails `out_of_range`.
    pub fn checked_at_door(&self, ty: &Ty) -> bool {
        ty.checked_at_door(&self.structs, &self.enums)
    }

    fn check_c_shaped(&self, ty: &Ty, span: Span, what: &str) -> Result<()> {
        if ty.is_c_shaped(&self.structs, &self.enums) {
            return Ok(());
        }
        let why = match ty.base() {
            Ty::Struct { id, .. } => format!("`{}` is not `layout.C`: only a `layout.C struct` has C's shape", self.structs[id.0 as usize].name),
            Ty::Enum { id, .. } => format!("`{}` is not `layout.C`: only a `layout.C enum` crosses to C, a number per case", self.enums[id.0 as usize].name),
            Ty::Str(_) => "a `str` is not C-shaped: C sees bytes, so hand it a `ptr[uint8]` (a lend of the text)".to_string(),
            Ty::Seq { .. } => "a list is not C-shaped: C sees a pointer to the elements — `ptr[T]` and a length".to_string(),
            Ty::Fn(_) => "the callback is not C-shaped: parameters by value (no `lent`), C-shaped types, no `fail`".to_string(),
            _ => format!("`{}` is not C-shaped: only numbers, `bool`, `ptr[…]` and `layout.C` structs cross to C", spell(ty, self)),
        };
        err(Diagnostic::error(format!("{} is not C-shaped", what)).at(span).why(why))
    }

    fn resolve_fn_decl(&mut self, decl: &ast::FnDecl, m: ModuleId, op: Option<String>, vis: Vis, span: Span) -> Result<FnDeclInfo> {
        // A `test fn` (owner, 2026-09-29): `cca test` calls it with nothing
        // and looks for nothing back but whether its `must`s hold.
        if decl.test {
            if let Some(p) = decl.params.first() {
                return err(Diagnostic::error(format!("the `test fn {}` takes no parameters: `kayo test` calls it with nothing — make '{}' a value inside it", decl.name.name, p.name.name)).at(p.span));
            }
            if decl.ret.is_some() || decl.ret_lent.is_some() || !decl.fails.is_empty() {
                return err(Diagnostic::error(format!("the `test fn {}` returns nothing and cannot fail: what it checks, it says with `must`", decl.name.name)).at(decl.name.span));
            }
        }
        let mut blanks = self.announced(&decl.announce, m)?;
        // A borrow's group is one the function announced (owner, 2026-10-02).
        let groups = decl.params.iter().filter_map(|p| p.group.as_ref()).chain(decl.ret_lent.iter().filter_map(|r| r.group.as_ref()));
        for g in groups {
            if !decl.announce.groups.iter().any(|a| a.name == g.name) {
                return err(Diagnostic::error(format!("`@{}` was never announced: a lifetime group is announced right after the name — `fn {} @{} […]`, or after the generics and a `|`", g.name, decl.name.name, g.name)).at(g.span));
            }
        }
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
            params.push(ParamDecl { kind: conv_kind(p.kind), group: p.group.as_ref().map(|g| g.name.clone()), amp: p.amp, mutability: p.mutability, variadic: p.variadic, name: p.name.name.clone(), ty, from_blank, span: p.span });
        }
        let ret = match &decl.ret {
            Some(t) => self.resolve_type(t, m, &mut blanks, true)?,
            None => Ty::Nothing,
        };
        let fails = self.resolve_fails(&decl.fails, m, &mut blanks)?;
        // A returned borrow's group is one its parameters are in.
        if let Some(rl) = &decl.ret_lent {
            if let Some(g) = &rl.group {
                if !decl.params.iter().any(|p| p.group.as_ref().map(|x| &x.name) == Some(&g.name)) {
                    return err(Diagnostic::error(format!("no parameter is `lent@{}`, so `-> lent@{}` would borrow from nothing: mark the parameters it may come from", g.name, g.name)).at(g.span));
                }
            }
            if !fails.is_empty() {
                return err(Diagnostic::error("not yet: a function that hands back a borrow and can also fail has no shape in kayo yet").at(rl.span));
            }
        }
        if let Some(o) = &op {
            if params.len() != 2 {
                return err(Diagnostic::error(format!("an operator takes exactly two operands; `{}` is declared with {}", o, params.len())).at(decl.name.span));
            }
            let language_op = matches!(o.as_str(), "plus" | "minus" | "times" | "pow" | "equal_to" | "not_equal" | "<" | ">" | "at_most" | "at_least" | "++" | "and" | "or" | "not" | "xor" | "shl" | "shr" | "plus.wrap" | "minus.wrap" | "times.wrap" | "pow.wrap") || o.starts_with("div") || o.starts_with("mod");
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
        Ok(FnDeclInfo { name: decl.name.name.clone(), path, module: m, vis, params, ret, ret_lent: decl.ret_lent.as_ref().map(|r| (r.mutable, r.group.as_ref().map(|g| g.name.clone()))), fails, blanks, body: decl.body.clone(), op, is_literal: false, test: decl.test, span, name_span: decl.name.span })
    }

    pub fn resolve_fails(&mut self, fails: &[ast::FailSpec], m: ModuleId, blanks: &mut Vec<(String, Vec<IfaceId>)>) -> Result<Vec<(String, Option<Ty>)>> {
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
        match self.resolve_bare(path, m, false)? {
            Bareness::Own(i) | Bareness::Dep(_, i) => Ok(i),
            Bareness::Std => unreachable!("std is asked for only by calls"),
        }
    }

    /// Where a path written in module `m` leads (owner, 2026-10-06): a
    /// bare one may be this Kit's own, std's (when `std` says the name is
    /// one of std's and this Kit lists std), or another Kit's marked
    /// `bare.optional` or `bare.only`; more than one is refused where it is
    /// written, and this Kit's own does not win (owner: "no"). One with
    /// another Kit's prefix is refused when that Kit forbids the path.
    pub fn resolve_bare(&self, path: &ast::Path, m: ModuleId, std: bool) -> Result<Bareness> {
        if let Some(kit) = path.kit.as_ref().filter(|l| l.name != "std") {
            let item = self.lookup_plain(path, m)?;
            let span = self.dependency(kit, m).ok().and_then(|root| self.lookup_pub_from(path, root)).map(|(_, s)| s);
            if span.and_then(|s| self.bare_marks.get(&s)) == Some(&ast::Bare::Only) {
                let bare = path.segs.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(".");
                let call = if matches!(item, ItemRef::Fn(_) | ItemRef::ExternFn(_)) { "[…]" } else { "" };
                return err(Diagnostic::error(format!("`{}` is `bare.only`: `{}` forbids its path, so write `{}{}`", path.spelled(), kit.name, bare, call)).at(path.span));
            }
            return Ok(Bareness::Own(item));
        }
        let own = self.lookup_plain(path, m);
        if path.kit.is_some() || path.head() == "my" {
            return own.map(Bareness::Own);
        }
        // A bare name means this module's, and a module sees the same
        // clash as the root (design claude, 2026-10-06); its own is named
        // from the root, `my.m.exit`.
        let here = self.kit_root(m);
        let written = path.spelled();
        let mut found: Vec<(String, Bareness)> = Vec::new();
        if let Ok(i) = &own {
            if !matches!(i, ItemRef::Module(_)) {
                let module = &self.modules[m.0 as usize].path;
                let inner = module.rsplit("::").next().unwrap_or(module);
                let at = if m == here { String::new() } else { format!("{}.", inner) };
                found.push((format!("`my.{}{}` (this kit's)", at, written), Bareness::Own(*i)));
            }
        }
        // Inside the Nerve cord, its own names are its own, not `cord::`'s.
        if std && !self.no_std_roots.contains(&here) && self.nerve_cord != Some(here) {
            found.push((format!("`std::{}`", written), Bareness::Std));
        }
        let mut never: Option<String> = None;
        if let Some(deps) = self.kit_deps.get(&here) {
            let mut deps: Vec<(&String, &ModuleId)> = deps.iter().collect();
            deps.sort_by(|a, b| a.0.cmp(b.0));
            for (prefix, root) in deps {
                let Some((item, span)) = self.lookup_pub_from(path, *root) else { continue };
                if matches!(item, ItemRef::Module(_)) {
                    continue;
                }
                match self.bare_marks.get(&span) {
                    Some(ast::Bare::Optional | ast::Bare::Only) => found.push((format!("`{}::{}`", prefix, written), Bareness::Dep(prefix.clone(), item))),
                    Some(ast::Bare::Never) if matches!(item, ItemRef::Fn(_) | ItemRef::ExternFn(_)) => {
                        never.get_or_insert(prefix.clone());
                    }
                    _ => {}
                }
            }
        }
        if found.len() > 1 {
            return err(Diagnostic::error(format!("`{}` could be any of {}: write the path", written, found.iter().map(|(l, _)| l.as_str()).collect::<Vec<_>>().join(", "))).at(path.span));
        }
        if let Some((_, b)) = found.pop() {
            return Ok(b);
        }
        if let Some(prefix) = never {
            return err(Diagnostic::error(format!("`{w}` is `{p}`'s and `bare.never`: `{p}` requires its path, so write `{p}::{w}[…]`", w = written, p = prefix)).at(path.span));
        }
        own.map(Bareness::Own)
    }

    /// `Counter.next`: a function belonging to a type (owner, 2026-10-09:
    /// "`b`"), when the path's head is a struct or an enum. `None` when it
    /// is not a type's, or names an enum's case. One without `pub` is seen
    /// across its Neuron (the oracle's pick; not asked).
    pub fn lookup_type_fn(&self, path: &ast::Path, m: ModuleId) -> Result<Option<FnId>> {
        if path.segs.len() < 2 {
            return Ok(None);
        }
        let prefix = ast::Path { kit: path.kit.clone(), segs: path.segs[..path.segs.len() - 1].to_vec(), span: path.span };
        let ty = match self.lookup_path(&prefix, m) {
            Ok(t @ (ItemRef::Struct(_) | ItemRef::Enum(_) | ItemRef::Fn(_))) => t,
            _ => return Ok(None),
        };
        let last = path.last();
        let Some(&id) = self.type_fns.get(&(ty, last.name.clone())) else {
            if let ItemRef::Struct(s) = ty {
                let name = &self.structs[s.0 as usize].name;
                return err(Diagnostic::error(format!("`{}` has no function `{}`: there is no `fn {}.{}`", name, last.name, name, last.name)).at(last.span));
            }
            if let ItemRef::Fn(f) = ty {
                let name = &self.fns[f.0 as usize].name;
                return err(Diagnostic::error(format!("`{}` has no `{}`: there is no `fn {}.{}`", name, last.name, name, last.name)).at(last.span));
            }
            return Ok(None);
        };
        let f = &self.fns[id.0 as usize];
        if self.kit_root(f.module) != self.kit_root(m) && f.vis != Vis::Pub {
            return err(Diagnostic::error(format!("`fn {}` is not `pub`, and another kit reaches only `pub` items", f.name)).at(last.span).fix(None, "its own kit opens it with `pub`"));
        }
        Ok(Some(id))
    }

    /// A `pub` item of another Kit, from its root, and its name's span.
    /// What the Nerve cord has at this path, `pub` (owner, 2026-10-14), and
    /// whether code may write it bare.
    pub fn nerve_cord_item(&self, path: &ast::Path) -> Option<(ItemRef, bool)> {
        let (item, span) = self.lookup_pub_from(path, self.nerve_cord?)?;
        Some((item, matches!(self.bare_marks.get(&span), Some(ast::Bare::Optional | ast::Bare::Only))))
    }

    fn lookup_pub_from(&self, path: &ast::Path, root: ModuleId) -> Option<(ItemRef, Span)> {
        let mut cur = root;
        let last = path.segs.len() - 1;
        for (i, seg) in path.segs.iter().enumerate() {
            let (item, vis, span) = self.modules[cur.0 as usize].item(&seg.name)?;
            if *vis != Vis::Pub {
                return None;
            }
            match item {
                _ if i == last => return Some((*item, *span)),
                ItemRef::Module(child) => cur = *child,
                _ => return None,
            }
        }
        None
    }

    /// The path walked as written, from module `m`.
    fn lookup_plain(&self, path: &ast::Path, m: ModuleId) -> Result<ItemRef> {
        let mut cur = m;
        if let Some(kit) = &path.kit {
            match (kit.name.as_str(), self.nerve_cord) {
                ("std", Some(root)) => cur = root,
                ("std", None) => return err(Diagnostic::error(format!("`{}` is not available yet: there is no `std` kit for now — the base language comes first", path.spelled())).at(path.span)),
                _ => cur = self.dependency(kit, m)?,
            }
            if path.segs.first().map_or(false, |s| s.name == "my") {
                return err(Diagnostic::error(format!("`{}::` already starts at that kit's root: write `{}::…` without `my.`", kit.name, kit.name)).at(path.span));
            }
        }
        let mut segs = path.segs.iter().peekable();
        let first = segs.peek().unwrap();
        if first.name == "my" {
            segs.next();
            cur = self.kit_root(m);
            if segs.peek().is_none() {
                return err(Diagnostic::error("`my` is followed by `.` and the root item it reaches: `my.helper[…]`").at(path.span));
            }
        }
        let mut result = None;
        let mut first_seg = path.kit.is_none();
        while let Some(seg) = segs.next() {
            let module = &self.modules[cur.0 as usize];
            // The base's `optional`, which no Neuron may declare.
            if std::mem::take(&mut first_seg) && seg.name == "optional" && module.item(&seg.name).is_none() {
                if let Some(id) = self.base_optional {
                    if segs.peek().is_some() {
                        return err(Diagnostic::error("`optional` is an enum, not a module: nothing comes after it in a path but a case").at(seg.span));
                    }
                    result = Some(ItemRef::Enum(id));
                    continue;
                }
            }
            let Some((item, vis, _)) = module.item(&seg.name) else {
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

    /// A const reached by path: `a.'k'`, `kit.'scale'`, `areas::'pi'`.
    pub fn lookup_const_path(&self, path: &ast::Path, name: &ast::QName, m: ModuleId) -> Result<crate::hir::ConstId> {
        if let Some(kit) = &path.kit {
            if kit.name == "std" && self.nerve_cord.is_none() {
                return err(Diagnostic::error("`std::` is not available yet: there is no `std` kit for now").at(path.span));
            }
            if kit.name == "std" && path.segs.is_empty() {
                return self.lookup_const_in(self.nerve_cord.unwrap(), name, m);
            }
            if path.segs.is_empty() {
                let root = self.dependency(kit, m)?;
                return self.lookup_const_in(root, name, m);
            }
        }
        let target = if path.is(&["my"]) {
            self.kit_root(m)
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
        let Some((id, vis, _)) = module.const_named(&name.name) else {
            return err(Diagnostic::error(format!("there is no const '{}' in {}", name.name, describe_module(&module.path))).at(name.span));
        };
        if !self.visible(*vis, target, from) && self.kit_root(target) != self.kit_root(from) {
            return err(Diagnostic::error(format!("the const '{}' is not `pub`, and another kit reaches only `pub` items", name.name)).at(name.span));
        }
        if !self.visible(*vis, target, from) {
            return err(Diagnostic::error(format!("'{}' is private to {}: add `pub` (other kits too) or `pub.kit` (this kit) where it is declared", name.name, describe_module(&module.path))).at(name.span));
        }
        Ok(*id)
    }

    /// The root of the Kit `kit::` names from module `m`: one its
    /// `CCA.cplr` lists (owner, 2026-09-29).
    fn dependency(&self, kit: &ast::Ident, m: ModuleId) -> Result<ModuleId> {
        let here = self.kit_root(m);
        match self.kit_deps.get(&here).and_then(|d| d.get(&kit.name)) {
            Some(r) => Ok(*r),
            None => err(Diagnostic::error(format!("no kit with the prefix `{}`: a kit this one uses is listed in its `CyJet.cyr` (`dependencies = [cyjet::dep[path = …],]`), and its own `CyJet.cyr` says `prefix = \"{}\"`", kit.name, kit.name)).at(kit.span)),
        }
    }

    fn visible(&self, vis: Vis, declared_in: ModuleId, from: ModuleId) -> bool {
        // Only `pub` crosses a Kit's edge (`kits.md`).
        if self.kit_root(declared_in) != self.kit_root(from) {
            return vis == Vis::Pub;
        }
        match vis {
            Vis::Pub | Vis::Kit => true,
            Vis::Private => self.inside(from, declared_in),
        }
    }

    fn check_visible(&self, item: ItemRef, vis: Vis, declared_in: ModuleId, from: ModuleId, seg: &ast::Ident) -> Result<()> {
        if self.visible(vis, declared_in, from) {
            return Ok(());
        }
        let where_ = describe_module(&self.modules[declared_in.0 as usize].path);
        let what = item.what();
        if self.kit_root(declared_in) != self.kit_root(from) {
            return err(Diagnostic::error(format!("{} `{}` is not `pub`, and another kit reaches only `pub` items", what, seg.name)).at(seg.span).fix(None, "its own kit opens it with `pub`; `pub.kit` keeps it inside that kit"));
        }
        let full = match item {
            ItemRef::Module(c) => format!("module `{}`", self.modules[c.0 as usize].path),
            _ => format!("{} `{}`", what, seg.name),
        };
        err(Diagnostic::error(format!("{} is private to {}: it is visible there and in the modules nested inside it", full, where_))
            .at(seg.span)
            .fix(None, format!("write `pub.kit` before it to open it to the whole kit, or `pub` for other kits too")))
    }

    fn hint_for_missing(&self, name: &str, m: ModuleId) -> String {
        // Is it somewhere else? Say where.
        for (i, module) in self.modules.iter().enumerate() {
            if module.items.contains_key(name) && ModuleId(i as u32) != m {
                let from_here = if i == 0 { format!("kit.{}", name) } else { format!("kit.{}.{}", module.path, name) };
                return format!("`{}` is declared in {} — from here that is `{}`; a bare name is this module's, a dotted path is relative to it", name, describe_module(&module.path), from_here);
            }
        }
        "a bare name is this module's own; another module's item is reached by its path, `kit.` starts at the root".to_string()
    }

    /// Resolve a written type in module `m`. `blanks` collects `<T>` by first
    /// appearance; `allow_blanks` is false where a blank cannot appear.
    /// The generics a declaration announced, in order, each with the
    /// interfaces it must satisfy (none for `any`).
    pub fn announced(&self, a: &ast::Announce, m: ModuleId) -> Result<Vec<(String, Vec<IfaceId>)>> {
        let mut out: Vec<(String, Vec<IfaceId>)> = Vec::new();
        for g in &a.generics {
            if out.iter().any(|(n, _)| *n == g.name.name) {
                return err(Diagnostic::error(format!("`<{}>` is announced twice", g.name.name)).at(g.name.span));
            }
            let mut bounds = Vec::new();
            for p in &g.bounds {
                match self.lookup_path(p, m) {
                    Ok(ItemRef::Iface(i)) => bounds.push(i),
                    Ok(other) => return err(Diagnostic::error(format!("`{}` is a {}, not an interface: what a generic accepts is an interface — `<{}>: Ordered` — or `any`", p.spelled(), other.what(), g.name.name)).at(p.span)),
                    Err(_) => return err(Diagnostic::error(format!("there is no interface named `{}`", p.spelled())).at(p.span)),
                }
            }
            out.push((g.name.name.clone(), bounds));
        }
        Ok(out)
    }

    /// A type where an opaque C struct may be named, for the caller to say
    /// what of it is refused (`size.of[FILE]`).
    pub fn resolve_type_opaque_ok(&self, t: &ast::Type, m: ModuleId, blanks: &mut Vec<(String, Vec<IfaceId>)>, allow_blanks: bool) -> Result<Ty> {
        POINTEE.with(|p| p.set(true));
        self.resolve_type(t, m, blanks, allow_blanks)
    }

    pub fn resolve_type(&self, t: &ast::Type, m: ModuleId, blanks: &mut Vec<(String, Vec<IfaceId>)>, allow_blanks: bool) -> Result<Ty> {
        // Whether this type is what a `ptr` points at, the one place an
        // opaque C struct may stand.
        let pointee = POINTEE.with(|p| p.replace(false));
        match &t.kind {
            ast::TypeKind::Var { name, bound } => {
                if !allow_blanks {
                    return err(Diagnostic::error(format!("a blank `<{}>` cannot appear here", name.name)).at(t.span));
                }
                let _ = bound;
                // Every generic is announced right after its declaration's
                // name (owner, 2026-10-02), so a mention names one of those.
                match blanks.iter().position(|(n, _)| *n == name.name) {
                    Some(pos) => Ok(Ty::Var(pos as u32)),
                    None => err(Diagnostic::error(format!("`<{}>` was never announced: a generic is announced right after the name, with what it accepts — `fn f <{}>: any […]`", name.name, name.name)).at(t.span)),
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
                Ok(Ty::Fn(Box::new(FnTy { params: ps, ret: r, fails: fs, owning: false })))
            }
            ast::TypeKind::Bound { inner, lo, hi, unit, span, old } => {
                let base = self.resolve_type(inner, m, blanks, allow_blanks)?;
                if *old {
                    // Whether it takes a bound comes before how one is
                    // written (owner, 2026-10-12, labelled bounds).
                    return match &base {
                        Ty::Seq { kind: SeqKind::Array, .. } => err(Diagnostic::error("a length bound belongs on a `list`: an `array` has one length, already known").at(*span)),
                        Ty::Int(_) | Ty::Bin(_) | Ty::Deci(_) | Ty::Str(_) | Ty::Seq { .. } | Ty::Map { .. } | Ty::Channel(_) => err(Diagnostic::error("a bound says which it is: `int32[min 0, max 100]` (`[min 1]` alone is \"at least 1\") (owner, 2026-10-12)").at(*span)),
                        other => err(Diagnostic::error(format!("`{}` takes no bound: bounds go on numbers (`int32[min 0, max 100]`), text, lists, maps and channels", spell(other, self))).at(*span)),
                    };
                }
                self.resolve_bound(base, lo, hi, unit.as_ref(), *span)
            }
            ast::TypeKind::Count(n) => err(Diagnostic::error(format!("`{}` is a number, not a type: only a `simd[T, N]` takes one, as its lane count", n)).at(t.span)),
            ast::TypeKind::Path { path, args } => {
                POINTEE.with(|p| p.set(pointee));
                self.resolve_type_path(path, args, m, blanks, allow_blanks, t.span)
            }
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
                // `min` and `max` are the base type's own limits
                // (`input.md`, one-ended ranges): `int8[1, max]` is 1 to 127.
                let limits = match &base {
                    Ty::Int(IntW::Fixed { signed, bits }) => Some(int_range(*signed, *bits)),
                    _ => None,
                };
                let end = |e: &ast::Expr| -> Result<i128> {
                    if let Some(v) = int_literal(e) {
                        return Ok(v);
                    }
                    if let ast::ExprKind::Path(p) = &e.kind {
                        if p.is(&["min"]) || p.is(&["max"]) {
                            let Some((min, max)) = limits else {
                                return err(Diagnostic::error(format!("`{}` is the type's own limit, and this integer's width is not known yet", p.head())).at(e.span));
                            };
                            return Ok(if p.is(&["min"]) { min } else { max });
                        }
                    }
                    err(Diagnostic::error("an integer's bounds are integers: `int32[min 0, max 100]`").at(e.span))
                };
                let (l, h) = (end(lo)?, end(hi)?);
                if l > h {
                    return err(Diagnostic::error(format!("the low bound comes first: `[min {}, max {}]`", h, l)).at(span));
                }
                let said = Said { lo: !left_out(lo, "min"), hi: !left_out(hi, "max") };
                if let Ty::Int(IntW::Fixed { signed, bits }) = &base {
                    let (min, max) = int_range(*signed, *bits);
                    // A bound is a value of its type (owner, 2026-09-23):
                    // no `uint8` is 300, and none is -1.
                    for (v, e) in [(l, lo), (h, hi)] {
                        if v < min || v > max {
                            return err(Diagnostic::error(format!("{} does not fit {} ({} to {}): a range's bounds are values of its type", v, spell(&base, self), min, max)).at(e.span));
                        }
                    }
                }
                Bound::Int { lo: l, hi: h, said }
            }
            Ty::Bin(_) => {
                if let Some(u) = unit {
                    return err(Diagnostic::error(format!("a number's bound has no unit; `{}` belongs on a text's length bound", u.name)).at(u.span));
                }
                // The largest finite value of each width; a bound past it
                // is no value of the type (owner, 2026-09-23).
                let top = match &base {
                    Ty::Bin(BinW::Fixed(16)) => Some(65504.0),
                    Ty::Bin(BinW::Fixed(32)) => Some(f32::MAX as f64),
                    Ty::Bin(BinW::Fixed(64)) => Some(f64::MAX),
                    _ => None,
                };
                // A side left out is the type's own limit (owner,
                // 2026-10-12): `bin64[min 0.0]` is 0.0 to the largest bin64.
                let side = |e: &ast::Expr| -> Result<Option<String>> {
                    match &e.kind {
                        ast::ExprKind::Path(p) if p.is(&["min"]) || p.is(&["max"]) => match top {
                            Some(t) => Ok(Some(format!("{:?}", if p.is(&["min"]) { -t } else { t }))),
                            None => err(Diagnostic::error(format!("`{}` is the type's own limit, and this bin's width is not known yet", p.head())).at(e.span)),
                        },
                        _ => Ok(None),
                    }
                };
                let l = match side(lo)? {
                    Some(v) => v,
                    None => bin_literal(lo).ok_or_else(|| Box::new(Diagnostic::error("a bin's bounds are bins: `bin64[min 0.0, max 1.0]` — an integer's bounds are integers, and the two do not mix").at(lo.span)))?,
                };
                let h = match side(hi)? {
                    Some(v) => v,
                    None => bin_literal(hi).ok_or_else(|| Box::new(Diagnostic::error("a bin's bounds are bins: `bin64[min 0.0, max 1.0]`").at(hi.span)))?,
                };
                if let Some(top) = top {
                    for (t, e) in [(&l, lo), (&h, hi)] {
                        let v = t.parse::<f64>().unwrap_or(f64::INFINITY);
                        if !(v.abs() <= top) {
                            return err(Diagnostic::error(format!("{} does not fit {}: a range's bounds are values of its type", t, spell(&base, self))).at(e.span));
                        }
                    }
                }
                if l.parse::<f64>().unwrap_or(0.0) > h.parse::<f64>().unwrap_or(0.0) {
                    return err(Diagnostic::error(format!("the low bound comes first: `[min {}, max {}]`", h, l)).at(span));
                }
                Bound::Bin { lo: l, hi: h, said: Said { lo: !left_out(lo, "min"), hi: !left_out(hi, "max") } }
            }
            // With `bin`'s rules (owner, 2026-09-29): the bounds are
            // written with a point, and each is a value of the width.
            Ty::Deci(bits) => {
                if let Some(u) = unit {
                    return err(Diagnostic::error(format!("a number's bound has no unit; `{}` belongs on a text's length bound", u.name)).at(u.span));
                }
                for e in [lo, hi] {
                    if let ast::ExprKind::Path(p) = &e.kind {
                        if p.is(&["min"]) || p.is(&["max"]) {
                            return err(Diagnostic::error(format!("not yet: a deci's bound with a side left out — kayo writes both sides of a deci's range so far: `deci{}[min 0.0, max 1.0]`", bits)).at(span));
                        }
                    }
                }
                let written = |e: &ast::Expr| bin_literal(e).ok_or_else(|| Box::new(Diagnostic::error(format!("a deci's bounds are written with a point, as a bin's are: `deci{}[min 0.0, max 1.0]`", bits)).at(e.span)));
                let (l, h) = (written(lo)?, written(hi)?);
                for (t, e) in [(&l, lo), (&h, hi)] {
                    match crate::deci::literal(t, *bits) {
                        Ok(_) => {}
                        Err(crate::deci::LitErr::Digits(n)) => return err(Diagnostic::error(format!("`{}` has {} digits; deci{} holds {}: a range's bounds are values of its type", t, n, bits, crate::deci::digits(*bits))).at(e.span)),
                        Err(crate::deci::LitErr::Range) => return err(Diagnostic::error(format!("`{}` is past what deci{} holds: a range's bounds are values of its type", t, bits)).at(e.span)),
                    }
                }
                if l.replace('_', "").parse::<f64>().unwrap_or(0.0) > h.replace('_', "").parse::<f64>().unwrap_or(0.0) {
                    return err(Diagnostic::error(format!("the low bound comes first: `[min {}, max {}]`", h, l)).at(span));
                }
                Bound::Bin { lo: l, hi: h, said: Said::default() }
            }
            Ty::Str(_) => {
                let u = match unit {
                    Some(u) => match u.name.as_str() {
                        "bytes" | "byte" => Unit::Byte,
                        "points" | "point" => Unit::Point,
                        "graphemes" | "grapheme" => Unit::Grapheme,
                        other => return err(Diagnostic::error(format!("`{}` is not a unit of text length: `bytes`, `points` or `graphemes`", other)).at(u.span)),
                    },
                    None => return err(Diagnostic::error("a text's length bound must say which unit it counts — `bytes`, `points` or `graphemes` — after the last number: `str.utf8[min 1, max 40 graphemes]`; a text has three lengths and no single one").at(span)),
                };
                let (l, h) = len_bounds(lo, hi, span)?;
                Bound::Len { lo: l, hi: h, unit: Some(u) }
            }
            Ty::Seq { kind: SeqKind::Array, .. } => {
                return err(Diagnostic::error("a length bound belongs on a `list`: an `array` has one length, already known").at(span));
            }
            Ty::Seq { .. } | Ty::Map { .. } | Ty::Channel(_) => {
                if matches!(base, Ty::Channel(_)) && matches!(&hi.kind, ast::ExprKind::Path(p) if p.is(&["max"])) {
                    return err(Diagnostic::error("a bounded channel says how much it holds: `channel[T][max 100]`; with no bound it is a hand-off").at(span));
                }
                if let Some(u) = unit {
                    return err(Diagnostic::error(format!("a count bound has no unit; `{}` belongs on a text's length bound", u.name)).at(u.span));
                }
                let (l, h) = len_bounds(lo, hi, span)?;
                Bound::Len { lo: l, hi: h, unit: None }
            }
            other => return err(Diagnostic::error(format!("`{}` takes no bound: bounds go on numbers (`int32[min 0, max 100]`), text, lists, maps and channels", spell(other, self))).at(span)),
        };
        Ok(Ty::Bounded { inner: Box::new(base), bound })
    }

    fn resolve_type_path(&self, path: &ast::Path, args: &[ast::Type], m: ModuleId, blanks: &mut Vec<(String, Vec<IfaceId>)>, allow_blanks: bool, span: Span) -> Result<Ty> {
        let pointee = POINTEE.with(|p| p.replace(false));
        let head = path.head();
        let no_args = |what: &str| -> Result<()> {
            if !args.is_empty() {
                return err(Diagnostic::error(format!("`{}` takes no type parameters", what)).at(span));
            }
            Ok(())
        };
        if path.kit.is_none() {
            // Rust's names that have no CyborgPL type of their own (owner,
            // 2026-10-02), with what to write instead.
            let refused = match head {
                "usize" | "isize" => Some(format!("`{}` is as big as the machine, and a CyborgPL type's size never depends on it: write the width — `{}64` (or `{}32`)", head, if head == "usize" { "uint" } else { "int" }, if head == "usize" { "uint" } else { "int" })),
                "String" => Some("text is `str.utf8` (or `str.ascii`), and there is one kind of it: owned by default, lent with `lend` — Rust's `String` and `&str` are both `str.utf8`".to_string()),
                "f128" => Some("there is no `f128`, as there is no `bin128`: quad precision is software arithmetic on every machine — `f64`, or `deci128` for more digits".to_string()),
                _ => None,
            };
            if let Some(why) = refused {
                return err(Diagnostic::error(why).at(span));
            }
        }
        // The brain float comes in one width (owner, 2026-10-02).
        if path.kit.is_none() && head != "brain16" && head != "bfloat16" && head != "bf16" {
            let brainy = ["brain", "bfloat", "bf"].iter().find(|p| head.strip_prefix(*p).is_some_and(|r| !r.is_empty() && r.bytes().all(|c| c.is_ascii_digit())));
            if let Some(p) = brainy {
                return err(Diagnostic::error(format!("the brain float comes in `brain16` only (also written `bfloat16` and `bf16`): there is no `{}`, and `{}` is 16 bits by name", head, p)).at(span));
            }
        }
        if path.kit.is_none() && head == "bin128" {
            return err(Diagnostic::error("there is no `bin128`: quad precision is software arithmetic on every machine — `bin64`, or `deci128` for more digits (owner, 2026-09-23)").at(span));
        }
        // An integer of any width (owner, 2026-10-03).
        if let Some(rest) = head.strip_prefix("!std_") {
            return std_int(rest, span);
        }
        if path.kit.is_none() {
            // A 128-bit whole number is carried: what Precog's sets cannot
            // hold, it claims nothing about (`Domain::of_ty`).
            if let Some(t) = primitive(head) {
                if path.segs.len() > 1 {
                    return err(Diagnostic::error(format!("`{}` takes no narrowing: `{}` is already a complete type", path.spelled(), head)).at(span));
                }
                no_args(head)?;
                return Ok(t);
            }
            match head {
                // C's machine-sized types (owner, 2026-10-10), CNC's: each
                // read as the Nerve type it is on the target.
                "C" if path.segs.len() == 2 => {
                    no_args(&path.spelled())?;
                    let name = path.seg(1).unwrap_or("");
                    return match crate::target::c_type(name) {
                        Some(crate::target::CType::Int(signed, bits)) => Ok(Ty::int(signed, bits)),
                        Some(crate::target::CType::Bin(bits)) => Ok(Ty::Bin(crate::types::BinW::Fixed(bits as u16))),
                        Some(crate::target::CType::NotYet(why)) => err(Diagnostic::error(format!("not yet: {}", why)).at(span)),
                        None => err(Diagnostic::error(format!("`C.{}` is not one of C's machine-sized types: {}", name, crate::target::C_TYPES.iter().map(|t| format!("`C.{}`", t)).collect::<Vec<_>>().join(", "))).at(span)),
                    };
                }
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
                        Some(other) => return err(Diagnostic::error(format!("`map.{}` is not a variant: `map` (the prover decides), `map.ordered` (walkable, insertion order) or `map.unordered` (never walked)", other)).at(span)),
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
                    // A mask (owner, 2026-09-27: a): one answer per lane of
                    // the vectors it compares, so as many lanes as they have.
                    if elem.base() == &Ty::Bool {
                        let widths: Vec<u32> = [64u32, 32, 16, 8].iter().map(|b| TARGET_SIMD_BITS / b).collect();
                        if !widths.contains(&lanes) {
                            let listed = widths.iter().map(|w| w.to_string()).collect::<Vec<_>>();
                            return err(Diagnostic::error(format!("a mask has as many lanes as the vectors it compares: {} or {} here, not {}", listed[..listed.len() - 1].join(", "), listed[listed.len() - 1], lanes)).at(args[1].span));
                        }
                        return Ok(Ty::Simd { elem: Box::new(elem), lanes });
                    }
                    // A lane is a standard width (owner, 2026-10-03: a).
                    if let Ty::Int(IntW::Fixed { bits, .. }) = elem.base() {
                        if !matches!(bits, 8 | 16 | 32 | 64 | 128) {
                            return err(Diagnostic::error(format!("a vector's lanes are a standard width, and `{}` is not one: `int8`, `int16`, `int32` or `int64` (unsigned too)", spell(&elem, self))).at(args[0].span));
                        }
                    }
                    if simd_lane_bits(&elem).is_none() {
                        return err(Diagnostic::error(format!("a `simd` holds plain numbers or bools, and `{}` is neither: its lanes live in a register, so each one is a fixed-width number", spell(&elem, self))).at(args[0].span));
                    }
                    if !simd_shape_ok(&elem, lanes) {
                        let has = simd_shapes_for(&elem).unwrap_or(0);
                        return err(Diagnostic::error(format!("this target has no `simd[{}, {}]`: its vector register holds {} lanes of `{}`, not {}", spell(&elem, self), lanes, has, spell(&elem, self), lanes))
                            .at(span)
                            .why(format!("a `simd` is only ever a shape the machine has ({}-bit registers here); portability is the plain `for` loop, which the prover and LLVM vectorise by themselves", TARGET_SIMD_BITS))
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
                    POINTEE.with(|p| p.set(true));
                    let elem = self.resolve_type(&args[0], m, blanks, allow_blanks)?;
                    if !elem.contains_var() && !elem.is_c_shaped(&self.structs, &self.enums) {
                        let why = match elem.base() {
                            Ty::Struct { id, .. } => format!("`{}` is not `layout.C`: only a `layout.C struct` has C's shape", self.structs[id.0 as usize].name),
                            other => format!("`{}` is not C-shaped", spell(other, self)),
                        };
                        return err(Diagnostic::error(format!("`ptr[{}]` cannot exist: a pointer points at something C-shaped", spell(&elem, self))).at(span).why(why));
                    }
                    return Ok(Ty::Ptr { mutable, elem: Box::new(elem) });
                }
                "duration" => return Ok(Ty::Duration),
                "fn" => unreachable!(),
                "uenr" => return err(Diagnostic::error("`uenr` is the standard kit's type, `std::uenr`, and `std::` is not available yet").at(span)),
                _ => {}
            }
        }
        // A user type by path; a C function's name, the C type sharing it.
        let item = match self.lookup_path(path, m)? {
            ItemRef::ExternFn(f) if self.c_fn_types.contains_key(&f) => self.c_fn_types[&f],
            other => other,
        };
        match item {
            ItemRef::Struct(id) => {
                let info = &self.structs[id.0 as usize];
                if info.opaque && !pointee {
                    return err(Diagnostic::error(format!("`{n}` is C's and opaque (owner, 2026-10-10): it is only pointed at, `ptr.mut[{n}]`, never held", n = info.name)).at(span));
                }
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

    fn type_args(&self, args: &[ast::Type], declared: &[String], name: &str, m: ModuleId, blanks: &mut Vec<(String, Vec<IfaceId>)>, allow_blanks: bool, span: Span) -> Result<Vec<Ty>> {
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
    FnDeclInfo { name: name.to_string(), path: name.to_string(), module: m, vis, params: Vec::new(), ret: Ty::Nothing, ret_lent: None, fails: Vec::new(), blanks: Vec::new(), body: None, op: None, is_literal: false, test: false, span, name_span }
}

fn qualify(items: &Items, m: ModuleId, name: &str) -> String {
    if m == ROOT {
        name.to_string()
    } else {
        format!("{}.{}", items.modules[m.0 as usize].path, name)
    }
}

/// A module's path for messages: `shapes.circle` in the Kit being built,
/// `areas::shapes` in a dependency, whose root is `areas::kit`.
pub fn module_path(items: &Items, name: &str, parent: Option<ModuleId>) -> String {
    match parent {
        None => "kit".to_string(),
        Some(p) => {
            let pp = &items.modules[p.0 as usize];
            if pp.parent.is_none() {
                match pp.path.strip_suffix("::kit") {
                    Some(kit) => format!("{}::{}", kit, name),
                    None => name.to_string(),
                }
            } else {
                format!("{}.{}", pp.path, name)
            }
        }
    }
}

pub fn describe_module(path: &str) -> String {
    if let Some(kit) = path.strip_suffix("::kit") {
        format!("the root of kit `{}`", kit)
    } else if path == "kit" {
        "the root (`kit.cy`)".to_string()
    } else {
        format!("module `{}`", path)
    }
}

pub fn spell(t: &Ty, items: &Items) -> String {
    TySpelling { ty: t, structs: &items.structs, enums: &items.enums }.to_string()
}

/// Rust's number type names, read as CyborgPL's (owner, 2026-10-02).
pub fn rust_alias(w: &str) -> Option<&'static str> {
    Some(match w {
        "i8" => "int8",
        "i16" => "int16",
        "i32" => "int32",
        "i64" => "int64",
        "i128" => "int128",
        "u8" => "uint8",
        "u16" => "uint16",
        "u32" => "uint32",
        "u64" => "uint64",
        "u128" => "uint128",
        "f16" => "bin16",
        "f32" => "bin32",
        "f64" => "bin64",
        _ => return None,
    })
}

/// `!std.int24`, `!std.uint4`, `!std.i24`, `!std.u4` (owner, 2026-10-03; `.`
/// since 2026-10-06), held inside as `!std_…`:
/// any width from 1 to LLVM's 2^23 bits that is not a standard one.
fn std_int(rest: &str, span: Span) -> Result<Ty> {
    let (signed, digits, ours) = if let Some(d) = rest.strip_prefix("uint") {
        (false, d, true)
    } else if let Some(d) = rest.strip_prefix("int") {
        (true, d, true)
    } else if let Some(d) = rest.strip_prefix('u') {
        (false, d, false)
    } else if let Some(d) = rest.strip_prefix('i') {
        (true, d, false)
    } else {
        (true, "", false)
    };
    let not_a_type = || err(Diagnostic::error(format!("`not_std.{}` is not a type: a non-standard integer is `not_std.intN` or `not_std.uintN` (MTM's `not_std.iN` or `not_std.uN`), N its width in bits", rest)).at(span));
    if digits.is_empty() || !digits.bytes().all(|c| c.is_ascii_digit()) {
        return not_a_type();
    }
    let Ok(bits) = digits.parse::<u64>() else {
        return err(Diagnostic::error(format!("`not_std.{}` is past the widest integer there is: LLVM's {} bits", rest, crate::types::WIDEST_INT)).at(span));
    };
    if bits == 0 {
        return err(Diagnostic::error(format!("`not_std.{}` holds nothing: a width is at least 1 bit", rest)).at(span));
    }
    if bits > crate::types::WIDEST_INT as u64 {
        return err(Diagnostic::error(format!("`not_std.{}` is past the widest integer there is: LLVM's {} bits", rest, crate::types::WIDEST_INT)).at(span));
    }
    if matches!(bits, 8 | 16 | 32 | 64 | 128) {
        let std = if ours { format!("{}int{}", if signed { "" } else { "u" }, bits) } else { format!("{}{}", if signed { "i" } else { "u" }, bits) };
        return err(Diagnostic::error(format!("`{}` is standard: write `{}` — the mark `not_std.` says a width is not", std, std)).at(span));
    }
    Ok(Ty::int(signed, bits as u32))
}

pub fn primitive(head: &str) -> Option<Ty> {
    let head = rust_alias(head).unwrap_or(head);
    if matches!(head, "brain16" | "bfloat16" | "bf16") {
        return Some(Ty::Brain);
    }
    if head == "bool" {
        return Some(Ty::Bool);
    }
    let (signed, prefix) = if head.starts_with("uint") { (false, 4) } else if head.starts_with("int") { (true, 3) } else if head.starts_with("bin") {
        let bits: u16 = head[3..].parse().ok()?;
        // No `bin128` (owner, 2026-09-23): refused by name in
        // `resolve_type_path`.
        if matches!(bits, 16 | 32 | 64) {
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
    let bits: u32 = head[prefix..].parse().ok()?;
    if matches!(bits, 8 | 16 | 32 | 64 | 128) {
        Some(Ty::int(signed, bits))
    } else {
        None
    }
}

/// What a whole number of this width holds, as far as `i128` holds it: a
/// range past it is cut at `i128`'s ends. Where that would be wrong — a
/// proof, a store's narrowing — [`int_range_exact`] says so instead.
pub fn int_range(signed: bool, bits: u32) -> (i128, i128) {
    if let Some(r) = int_range_exact(signed, bits) {
        return r;
    }
    let (s, _) = storage(signed, bits);
    if s { (i128::MIN, i128::MAX) } else { (0, i128::MAX) }
}

/// The same, or `None` when `i128` does not hold the whole range: an
/// unsigned 128 bits or wider, a signed wider than 128, an exact answer
/// whose limit in between is past 126 bits.
pub fn int_range_exact(signed: bool, bits: u32) -> Option<(i128, i128)> {
    // The exact type: what some `limit`-bit type holds (`Ty::exact_at`).
    if signed && bits & crate::types::EXACT_FLAG != 0 {
        let limit = bits & !crate::types::EXACT_FLAG;
        if limit == 64 {
            return Some((i64::MIN as i128, u64::MAX as i128));
        }
        if limit >= 127 {
            return None;
        }
        return Some((-(1i128 << (limit - 1)), (1i128 << limit) - 1));
    }
    if (signed && bits > 128) || (!signed && bits >= 128) {
        return None;
    }
    Some(int_range_fits(signed, bits))
}

/// The width, and sign, that holds a type's every value: an exact answer
/// whose limit is L needs L + 1 signed bits.
pub fn storage(signed: bool, bits: u32) -> (bool, u32) {
    if signed && bits & crate::types::EXACT_FLAG != 0 {
        return (true, (bits & !crate::types::EXACT_FLAG) + 1);
    }
    (signed, bits)
}

/// Does every value of the first integer type fit the second? By widths,
/// so it holds past `i128` too.
pub fn int_fits(s1: bool, b1: u32, s2: bool, b2: u32) -> bool {
    let ((s1, b1), (s2, b2)) = (storage(s1, b1), storage(s2, b2));
    match (s1, s2) {
        (false, false) | (true, true) => b1 <= b2,
        (false, true) => b1 < b2,
        (true, false) => false,
    }
}

fn int_range_fits(signed: bool, bits: u32) -> (i128, i128) {
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

/// Whether a bound's side was left out: the parser writes the type's own
/// limit, `min` or `max`, for it.
fn left_out(e: &ast::Expr, w: &str) -> bool {
    matches!(&e.kind, ast::ExprKind::Path(p) if p.is(&[w]))
}

fn len_bounds(lo: &ast::Expr, hi: &ast::Expr, span: Span) -> Result<(u64, u64)> {
    // A side left out (owner, 2026-10-12): the low one is 0, the high one
    // unlimited.
    let left_out = |e: &ast::Expr, w: &str| matches!(&e.kind, ast::ExprKind::Path(p) if p.is(&[w]));
    let l = if left_out(lo, "min") { 0 } else { int_literal(lo).ok_or_else(|| Box::new(Diagnostic::error("a length bound's ends are whole numbers").at(lo.span)))? };
    let h = if left_out(hi, "max") { u64::MAX as i128 } else { int_literal(hi).ok_or_else(|| Box::new(Diagnostic::error("a length bound's ends are whole numbers").at(hi.span)))? };
    if l < 0 {
        return err(Diagnostic::error("a length bound's low end is 0 or more: a length is never negative").at(lo.span));
    }
    if l > h {
        return err(Diagnostic::error("the low bound is at most the high one: `list[int32][min 0, max 1000]`").at(span));
    }
    Ok((l as u64, h as u64))
}

thread_local! { static POINTEE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }

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
