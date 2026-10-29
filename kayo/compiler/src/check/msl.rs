//! `inline.MSL` checked (owner, 2026-10-01, 2026-10-22; `design/gpu.md`): a
//! `no_check` site holding one `kernel`; what it binds, each by the verb its
//! kind takes (a list or a `layout.C` struct lent, a number copied to a
//! Metal `constant`); its `#include`s inside the kit; and its code compiled
//! by Apple's compiler now, so a mistake in it is a build's error.

use crate::ast::{self, Verb};
use crate::diag::{err, Diagnostic, Edit, Result};
use crate::hir;
use crate::source::Span;
use crate::types::{SeqKind, Ty};

use super::{Checker, FnCtx, LocalKind};

/// What a bound value is, for the verb it takes.
enum Bound {
    /// A list or an array of numbers or `layout.C` structs, or one struct:
    /// lent.
    Lent,
    /// A number: copied.
    Value,
}

impl Checker {
    pub(super) fn inline_msl(&mut self, ctx: &mut FnCtx, text: &str, text_span: Span, threads: &ast::Expr, bindings: &[ast::MslBinding], (precise, wrap): (bool, bool), span: Span) -> Result<hir::StmtKind> {
        if self.needs_mark(ctx) {
            self.power_first()?;
            return err(Diagnostic::error("an `inline.%MSL` block is a no_check site: the compiler cannot prove anything inside it — write `no_check inline.%MSL threads … { … }`").named("no_check_missing").at(span).fix_edit(span, Edit::Before, "no_check "));
        }
        // One kernel, the one the block runs; helpers are fine.
        let kernels = crate::metal::kernels(text);
        let kernel = match kernels.as_slice() {
            [] => return err(Diagnostic::error("an `inline.%MSL` block holds one `kernel` function, the one it runs, and this one has none").named("metal::kernel_missing").at(text_span)),
            [one] => one.clone(),
            many => return err(Diagnostic::error(format!("an `inline.%MSL` block holds one kernel, and this one has {}: {}", many.len(), many.iter().map(|k| format!("`{}`", k)).collect::<Vec<_>>().join(", "))).named("metal::two_kernels").at(text_span)),
        };
        // Shared code is the kit's, from its root.
        let root = self.items.kit_root(ctx.module);
        let dir = self.items.neuron_dirs.get(&root).cloned().unwrap_or_else(|| std::path::PathBuf::from("."));
        for (path, _) in crate::metal::includes(text) {
            match crate::metal::inside_kit(&dir, &path) {
                None => return err(Diagnostic::error(format!("`#include \"{}\"` reaches outside the kit: a shared Metal file lives in the kit's folder, its path from the kit's root", path)).named("metal::include_outside_kit").at(text_span)),
                Some(full) if !full.is_file() => return err(Diagnostic::error(format!("`#include \"{}\"`: there is no such file in the kit (the path is from the kit's root, {})", path, full.display())).named("metal::include_missing").at(text_span)),
                Some(_) => {}
            }
        }
        // How many copies of the kernel run: a whole number.
        let threads_e = self.expr(ctx, threads)?;
        let threads_e = self.expect_store(ctx, threads_e, &Ty::u64(), "the thread count")?;
        // The bindings.
        let mut ins = Vec::new();
        let mut lends = Vec::new();
        let mut slots: Vec<(u32, String)> = Vec::new();
        let mut structs: Vec<usize> = Vec::new();
        for b in bindings {
            let name = &b.name.name;
            let Some(id) = ctx.lookup(name) else {
                return Err(ctx.not_declared(Diagnostic::error(format!("`${}` is not declared", name)).named("not_declared").at(b.name.span), name, b.name.span));
            };
            if let Some((_, first)) = slots.iter().find(|(s, _)| *s == b.slot) {
                return err(Diagnostic::error(format!("`buffer({})` is bound twice, to ${} and ${}: each slot takes one value", b.slot, first, name)).named("metal::slot_bound_twice").at(b.span));
            }
            slots.push((b.slot, name.clone()));
            let ty = self.resolve(&ctx.locals[id.0 as usize].ty);
            let structs_of = &self.items.structs;
            let elem_struct = |t: &Ty| match t.base() {
                Ty::Struct { id, .. } => Some(id.0 as usize),
                _ => None,
            };
            let (kind, what) = match ty.base() {
                Ty::Seq { elem, kind: SeqKind::List | SeqKind::Array } if crate::metal::metal_type(elem, structs_of).is_some() => {
                    if let Some(s) = elem_struct(elem) {
                        structs.push(s);
                    }
                    (Bound::Lent, if elem_struct(elem).is_some() { "a list of `%layout.C` structs" } else { "a list of numbers" })
                }
                Ty::Struct { .. } if crate::metal::metal_type(&ty, structs_of).is_some() => {
                    structs.push(elem_struct(&ty).unwrap());
                    (Bound::Lent, "a `%layout.C` struct")
                }
                t if crate::metal::metal_type(&ty, structs_of).is_some() && !matches!(t, Ty::Struct { .. }) => (Bound::Value, "a single value"),
                _ => return err(Diagnostic::error(format!("'{}' is {}, and what binds to the GPU is a number, a list of numbers, a `%layout.C` struct or a list of them, each in a form Metal has", name, self.spell(&ty))).named("metal::cannot_bind").at(b.name.span)),
            };
            match (kind, b.verb) {
                (Bound::Lent, Verb::Lend | Verb::LendMut) => {
                    if b.verb == Verb::LendMut {
                        let local = &ctx.locals[id.0 as usize];
                        let why = if local.mutability == Some(ast::Mutability::Immut) {
                            Some("is `immut`")
                        } else if local.kind == LocalKind::Lent {
                            Some("is lent (shared)")
                        } else if matches!(local.kind, LocalKind::LoopVar | LocalKind::LoopLent) {
                            Some("is the loop's element")
                        } else {
                            None
                        };
                        if let Some(why) = why {
                            return err(Diagnostic::error(format!("'{}' {}, so `lend.mut ${}` cannot hand the kernel its memory to write", name, why, name)).named(if why.starts_with("is lent") { "changed_through_lend" } else { "cannot_change_immut" }).at(b.name.span));
                        }
                        ctx.changed.insert(id);
                    }
                    lends.push((id, b.verb == Verb::LendMut, format!("buffer({})", b.slot)));
                }
                (Bound::Lent, _) => {
                    return err(Diagnostic::error(format!("'{}' is {}: lend it, `lend ${} -> buffer({})`, or `lend.mut ${} -> buffer({})` if the kernel writes it", name, what, name, b.slot, name, b.slot)).named("metal::binding_verb_wrong").at(b.span).fix_edit(b.span, Edit::Verb, "lend").fix_edit(b.span, Edit::Verb, "lend.mut"));
                }
                (Bound::Value, Verb::Copy) => ins.push((id, format!("buffer({})", b.slot))),
                (Bound::Value, _) => {
                    return err(Diagnostic::error(format!("'{}' is a single value: `copy ${} -> buffer({})`, a Metal `constant`", name, name, b.slot)).named("metal::binding_verb_wrong").at(b.span).fix_edit(b.span, Edit::Verb, "copy"));
                }
            }
        }
        // A value lent to read is read only in the kernel too (owner,
        // 2026-10-25: "(a)"): a list may be shared with no copy, and a write
        // would change what the program holds as unchanged.
        let params = crate::metal::buffer_params(text, &kernel);
        for b in bindings.iter().filter(|b| b.verb == Verb::Lend) {
            if let Some((_, at, false)) = params.iter().find(|(slot, _, _)| *slot == b.slot) {
                let start = text_span.start as usize + 1 + at;
                let word = text[*at..].split_whitespace().next().unwrap_or("");
                let param = Span::new(text_span.file, start, start + word.len());
                return err(Diagnostic::error(format!("${} is lent to read, and the kernel's `buffer({})` parameter can write it: make it `const device …` (or `constant …`), or lend it to change, `lend.mut ${} -> buffer({})`", b.name.name, b.slot, b.name.name, b.slot)).named("metal::lent_but_written").at(param).fix_edit(param, Edit::Insert, "const ").fix_edit(b.span, Edit::Verb, "lend.mut"));
            }
        }
        // The structs' Metal declarations, then the block as written.
        let decls = crate::metal::declarations(&structs, &self.items.structs).map_err(|what| Box::new(Diagnostic::error(format!("{} has no form in Metal: a struct the GPU reads holds numbers and `%layout.C` structs Metal has", what)).named("metal::cannot_bind").at(span)))?;
        let source = format!("#include <metal_stdlib>\nusing namespace metal;\n{}{}", decls, text);
        let before = source.lines().count() - text.lines().count();
        let library = crate::metal::compile(&source, &dir, precise, wrap).map_err(|(line, said)| {
            // The error's line in the file: the block's text starts just
            // after its `{`, and the line Apple's compiler names is in it.
            let at = line.map(|l| l.saturating_sub(before)).filter(|l| *l > 0).and_then(|l| {
                let offset: usize = text.split_inclusive('\n').take(l - 1).map(str::len).sum();
                let rest = &text[offset..];
                let lead = rest.len() - rest.trim_start_matches([' ', '\t']).len();
                let len = rest[lead..].find('\n').unwrap_or(rest.len() - lead);
                let start = text_span.start as usize + 1 + offset + lead;
                Some(Span::new(text_span.file, start, start + len))
            });
            Box::new(Diagnostic::error(format!("Apple's Metal compiler refuses the block's code: {}", said)).named("metal::compile_failed").at(at.unwrap_or(text_span)).outside_saying())
        })?;
        Ok(hir::StmtKind::InlineAsm { text: text.to_string(), ins, outs: Vec::new(), lends, clobbers: Vec::new(), msl: Some(Box::new(hir::Msl { library, kernel, threads: threads_e })) })
    }
}
