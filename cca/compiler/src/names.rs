//! Spelling things back the way the writer wrote them, for messages.

use crate::ast::{Type, TypeKind};

pub fn type_spelling(t: &Type) -> String {
    match &t.kind {
        TypeKind::Path { path, args } => {
            let mut s = path.spelled();
            if !args.is_empty() {
                s.push('[');
                s.push_str(&args.iter().map(type_spelling).collect::<Vec<_>>().join(", "));
                s.push(']');
            }
            s
        }
        TypeKind::Bound { inner, unit, .. } => {
            let mut s = type_spelling(inner);
            s.push_str("[…, …");
            if let Some(u) = unit {
                s.push(' ');
                s.push_str(&u.name);
            }
            s.push(']');
            s
        }
        TypeKind::Count(n) => n.to_string(),
        TypeKind::Var { name, bound } => match bound {
            Some(b) => format!("<{}: {}>", name.name, b.spelled()),
            None => format!("<{}>", name.name),
        },
        TypeKind::Fn { params, ret, fails } => {
            let mut s = String::from("fn[");
            s.push_str(&params.iter().map(|p| {
                let k = match p.kind {
                    crate::ast::ParamKind::Owned => "",
                    crate::ast::ParamKind::Lent => "lent ",
                    crate::ast::ParamKind::LentMut => "lent.mut ",
                };
                format!("{}{}", k, type_spelling(&p.ty))
            }).collect::<Vec<_>>().join(", "));
            s.push(']');
            if let Some(r) = ret {
                s.push_str(" -> ");
                s.push_str(&type_spelling(r));
            }
            if !fails.is_empty() {
                s.push_str(if ret.is_some() { " or fail[" } else { " -> fail[" });
                s.push_str(&fails.iter().map(|f| f.name.name.clone()).collect::<Vec<_>>().join(", "));
                s.push(']');
            }
            s
        }
    }
}
