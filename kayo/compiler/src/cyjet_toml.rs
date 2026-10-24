//! `CyJet.toml` (owner, 2026-10-22): Tankun's Obvious Minimal Language, the
//! owner's own format with TOML's initials. A section is a phrase ending in
//! `:`; under it come `key = value` lines, with no quotes and no indentation
//! that matters, and blocks, `name {` alone on a line, its keys, then `}`
//! alone on a line. Comments are CyborgPL's: `#`, `#wl`, `#Nl`, `#Nld`.
//! This reads the shape; `load::read_manifest` says what each part means.

use crate::diag::{err, Diagnostic, Edit, Result};
use crate::source::{FileId, Span};

/// The sections there are (owner, 2026-10-22: "`my targets`", "(a)" twice).
pub const SECTIONS: [&str; 7] = ["my details", "my kits", "my limbs", "my C libs", "my powers", "my denied powers", "my targets"];

pub struct Doc {
    pub sections: Vec<Section>,
}

pub struct Section {
    pub name: String,
    pub span: Span,
    pub pairs: Vec<Pair>,
    pub blocks: Vec<Block>,
}

pub struct Pair {
    pub key: String,
    pub key_span: Span,
    pub value: String,
    pub value_span: Span,
}

pub struct Block {
    pub name: String,
    pub span: Span,
    pub pairs: Vec<Pair>,
}

impl Doc {
    pub fn section(&self, name: &str) -> Option<&Section> {
        self.sections.iter().find(|s| s.name == name)
    }
}

/// A value's items: `write, read` is two.
pub fn list(value: &str) -> Vec<String> {
    value.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// Each line with its comments taken out, and where it starts.
fn uncommented(file: FileId, text: &str) -> Result<Vec<(usize, String)>> {
    let mut out = Vec::new();
    let mut skip = 0usize;
    let mut at = 0usize;
    for raw in text.split_inclusive('\n') {
        let start = at;
        at += raw.len();
        let line = raw.trim_end_matches(['\n', '\r']);
        if skip > 0 {
            skip -= 1;
            continue;
        }
        let Some(hash) = line.find('#') else {
            out.push((start, line.to_string()));
            continue;
        };
        let after = &line[hash + 1..];
        let form: String = after.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
        let bad = || err(Diagnostic::error(format!("`#{}` is not a comment form: `#` to the end of the line, `#wl` the whole line, `#Nl` N lines, `#Nld` this line and N down", form)).named("comment_form").at(Span::new(file, start + hash, start + hash + 1 + form.len())).fix_edit(Span::new(file, start + hash + 1, start + hash + 1), crate::diag::Edit::Insert, " "));
        if !form.is_empty() && !after[form.len()..].chars().next().is_none_or(char::is_whitespace) {
            return bad();
        }
        let before = line[..hash].to_string();
        if form.is_empty() {
            out.push((start, before));
            continue;
        }
        if form == "wl" {
            continue;
        }
        let digits: String = form.chars().take_while(char::is_ascii_digit).collect();
        let Ok(n) = digits.parse::<usize>() else { return bad() };
        match &form[digits.len()..] {
            // N lines counting this one.
            "l" => skip = n.saturating_sub(1),
            // This line and N more down.
            "ld" => skip = n,
            _ => return bad(),
        }
        out.push((start, before));
    }
    Ok(out)
}

/// The sections, pairs and blocks of a `CyJet.toml`.
pub fn read(file: FileId, text: &str) -> Result<Doc> {
    let mut doc = Doc { sections: Vec::new() };
    let mut open: Option<Block> = None;
    for (start, line) in uncommented(file, text)? {
        let lead = line.len() - line.trim_start().len();
        let content = line.trim();
        if content.is_empty() {
            continue;
        }
        let span = Span::new(file, start + lead, start + lead + content.len());
        if let Some(b) = &mut open {
            if content == "}" {
                let b = open.take().unwrap();
                doc.sections.last_mut().expect("a block is in a section").blocks.push(b);
                continue;
            }
            if content.ends_with('{') || content.ends_with(':') && !content.contains('=') {
                return err(Diagnostic::error(format!("the block `{}` is never closed with `}}`", b.name)).named("manifest_block_unclosed").at(b.span).fix_edit(span, Edit::InsertLine, "}"));
            }
            b.pairs.push(pair(file, start + lead, content)?);
            continue;
        }
        if content == "}" {
            return err(Diagnostic::error("a `}` with no block open: a block starts `name {` on a line of its own").named("manifest_line_form").at(span));
        }
        if content.ends_with(':') && !content.contains('=') {
            let name = content[..content.len() - 1].trim().to_string();
            if !SECTIONS.contains(&name.as_str()) {
                return err(Diagnostic::error(format!("`{}:` is not a section of `CyJet.toml`: its sections are {}", name, SECTIONS.iter().map(|s| format!("`{}:`", s)).collect::<Vec<_>>().join(", "))).named("manifest_unknown_section").at(span).fix_close(Span::new(file, span.start as usize, span.end as usize - 1), &name, SECTIONS));
            }
            if doc.section(&name).is_some() {
                return err(Diagnostic::error(format!("`{}:` is written twice: each section is written once", name)).named("manifest_given_twice").at(span));
            }
            doc.sections.push(Section { name, span, pairs: Vec::new(), blocks: Vec::new() });
            continue;
        }
        let Some(section) = doc.sections.last_mut() else {
            return err(Diagnostic::error("a `CyJet.toml` starts with a section, a phrase ending in `:`: `my details:`").named("manifest_line_form").at(span));
        };
        if content.contains('{') {
            let name = content[..content.find('{').unwrap()].trim();
            if !content.ends_with('{') || name.is_empty() {
                let n = if name.is_empty() { "name" } else { name };
                return err(Diagnostic::error(format!("a block opens with `{n} {{` on a line of its own, then its `key = value` lines, then `}}` alone on a line")).named("manifest_line_form").at(span));
            }
            open = Some(Block { name: name.to_string(), span, pairs: Vec::new() });
            continue;
        }
        if content.contains('=') {
            section.pairs.push(pair(file, start + lead, content)?);
            continue;
        }
        return err(Diagnostic::error(format!("`{}` is not a section (`my details:`), a block (`name {{`) or a `key = value` line", content)).named("manifest_line_form").at(span));
    }
    if let Some(b) = open {
        return err(Diagnostic::error(format!("the block `{}` is never closed with `}}`", b.name)).named("manifest_block_unclosed").at(b.span).fix_edit(b.span, Edit::Append, "}"));
    }
    Ok(doc)
}

fn pair(file: FileId, start: usize, content: &str) -> Result<Pair> {
    let eq = content.find('=').unwrap_or(content.len());
    let key = content[..eq].trim();
    let span = Span::new(file, start, start + content.len());
    if eq == content.len() || key.is_empty() {
        return err(Diagnostic::error("a line in a block is `key = value`").named("manifest_line_form").at(span));
    }
    let rest = &content[eq + 1..];
    let value = rest.trim();
    let vstart = start + eq + 1 + (rest.len() - rest.trim_start().len());
    let key_span = Span::new(file, start, start + key.len());
    let value_span = Span::new(file, vstart, vstart + value.len());
    // The words of the format (owner, 2026-10-22: "(a)").
    if key == "version" {
        return err(Diagnostic::error("it is `ver`, never `version`, in every section (owner, 2026-10-22)").named("manifest_version_word").at(key_span).fix_edit(key_span, Edit::Replace, "ver"));
    }
    if value.starts_with('"') || value.starts_with('\'') {
        let bare = value.trim_matches(|c| c == '"' || c == '\'');
        return err(Diagnostic::error(format!("a value has no quotes: `{} = {}`", key, bare)).named("manifest_value_quoted").at(value_span).fix_edit(value_span, Edit::Replace, bare.to_string()));
    }
    if value.is_empty() {
        return err(Diagnostic::error(format!("`{}` has no value: `{} = …`", key, key)).named("manifest_line_form").at(span));
    }
    Ok(Pair { key: key.to_string(), key_span, value: value.to_string(), value_span })
}
