//! `CyJet.toml`, Tankun's Obvious Minimal Language (owner, 2026-10-22; `design/cyjet-toml.md`): the owner's own
//! format, not TOML. A section is a phrase ending in `:`; under it come `key = value` lines and blocks, a name then
//! `{`, its `key = value` lines, then `}`. No indentation matters, no quotes. Comments are CyborgPL's family: `#` to
//! the end of the line, `#wl` the whole line, `#Nl` N lines counting this one, `#Nld` this line and N more.

pub struct Block { pub name: String, pub keys: Vec<(String, String, usize)>, pub line: usize }
pub struct Section { pub name: String, pub keys: Vec<(String, String, usize)>, pub blocks: Vec<Block>, pub line: usize }
pub struct Doc { pub sections: Vec<Section> }

/// The text of each line with its comments taken out, as CyborgPL's comments are
fn uncomment(src: &str) -> Vec<String> {
    let lines: Vec<&str> = src.split('\n').map(|l| l.trim_end_matches('\r')).collect();
    let mut out = Vec::with_capacity(lines.len());
    let mut skip = 0usize;
    for line in &lines {
        if skip > 0 { skip -= 1; out.push(String::new()); continue; }
        let Some(at) = line.find('#') else { out.push(line.to_string()); continue };
        let rest = &line[at + 1..];
        if rest.starts_with("wl") { out.push(String::new()); continue; }
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            let n: usize = digits.parse().unwrap_or(1);
            let after = &rest[digits.len()..];
            if after.starts_with("ld") { skip = n; } else if after.starts_with('l') { skip = n.saturating_sub(1); }
        }
        out.push(line[..at].to_string());
    }
    out
}

pub fn read(src: &str) -> Result<Doc, (usize, String)> {
    let mut doc = Doc { sections: Vec::new() };
    let mut open: Option<Block> = None;
    for (i, text) in uncomment(src).iter().enumerate() {
        let n = i + 1;
        let t = text.trim();
        if t.is_empty() { continue; }
        if t == "}" {
            let b = open.take().ok_or((n, "a `}` with no block open".to_string()))?;
            doc.sections.last_mut().expect("a block is always in a section").blocks.push(b);
            continue;
        }
        if let Some(name) = t.strip_suffix('{') {
            let name = name.trim();
            if open.is_some() { return Err((n, "a block inside a block: close the one before with `}`".into())); }
            if doc.sections.is_empty() { return Err((n, format!("the block `{name}` is in no section: write its section first, as `my kits:`"))); }
            if name.is_empty() || name.contains(char::is_whitespace) || name.contains('=') { return Err((n, "a block is a name, then `{`: `std {`".into())); }
            let sec = doc.sections.last().unwrap();
            if sec.blocks.iter().any(|b| b.name == name) { return Err((n, format!("`{name}` is given twice in `{}:`", sec.name))); }
            open = Some(Block { name: name.to_string(), keys: Vec::new(), line: n });
            continue;
        }
        if let Some((k, v)) = t.split_once('=') {
            let (k, v) = (k.trim(), v.trim());
            if k.is_empty() || k.contains(char::is_whitespace) { return Err((n, format!("a key is one word: `{k} = …`"))); }
            if v.is_empty() { return Err((n, format!("`{k}` has no value: `{k} = …`"))); }
            if v.starts_with('"') { return Err((n, format!("a value has no quotes: `{k} = {}`", v.trim_matches('"')))); }
            let keys = match &mut open {
                Some(b) => &mut b.keys,
                None => match doc.sections.last_mut() { Some(s) => &mut s.keys, None => return Err((n, format!("`{k}` is in no section: write its section first, as `my details:`"))) },
            };
            if keys.iter().any(|(x, _, _)| x == k) { return Err((n, format!("`{k}` is given twice"))); }
            keys.push((k.to_string(), v.to_string(), n));
            continue;
        }
        if let Some(name) = t.strip_suffix(':') {
            if open.is_some() { return Err((n, "a section inside a block: close the block with `}` first".into())); }
            let name = name.trim();
            if doc.sections.iter().any(|s| s.name == name) { return Err((n, format!("`{name}:` is written twice"))); }
            doc.sections.push(Section { name: name.to_string(), keys: Vec::new(), blocks: Vec::new(), line: n });
            continue;
        }
        return Err((n, format!("`{t}` is not a section (`my kits:`), a block (`std {{`), a `key = value` line or a `}}`")));
    }
    if let Some(b) = open { return Err((b.line, format!("the block `{}` is never closed with `}}`", b.name))); }
    Ok(doc)
}
