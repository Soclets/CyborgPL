//! Just enough JSON to read `cca precog --json`: one object per line; numbers
//! that matter come as decimal strings.

#[derive(Clone, Debug, PartialEq)]
pub enum J {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

impl J {
    pub fn get(&self, k: &str) -> Option<&J> {
        match self {
            J::Obj(kv) => kv.iter().find(|(a, _)| a == k).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn str(&self) -> Option<&str> {
        match self {
            J::Str(s) => Some(s),
            _ => None,
        }
    }
    /// A number, whether written as a number or as a decimal string.
    pub fn int(&self) -> Option<i128> {
        match self {
            J::Num(s) | J::Str(s) => s.parse().ok(),
            _ => None,
        }
    }
    pub fn arr(&self) -> Option<&[J]> {
        match self {
            J::Arr(a) => Some(a),
            _ => None,
        }
    }
}

pub fn parse(s: &str) -> Option<J> {
    let b = s.as_bytes();
    let mut i = 0;
    let v = value(b, &mut i)?;
    ws(b, &mut i);
    (i == b.len()).then_some(v)
}

fn ws(b: &[u8], i: &mut usize) {
    while *i < b.len() && (b[*i] as char).is_whitespace() {
        *i += 1;
    }
}

fn value(b: &[u8], i: &mut usize) -> Option<J> {
    ws(b, i);
    match *b.get(*i)? {
        b'n' => lit(b, i, "null", J::Null),
        b't' => lit(b, i, "true", J::Bool(true)),
        b'f' => lit(b, i, "false", J::Bool(false)),
        b'"' => string(b, i).map(J::Str),
        b'[' => {
            *i += 1;
            let mut v = vec![];
            ws(b, i);
            if b.get(*i) == Some(&b']') {
                *i += 1;
                return Some(J::Arr(v));
            }
            loop {
                v.push(value(b, i)?);
                ws(b, i);
                match b.get(*i)? {
                    b',' => *i += 1,
                    b']' => {
                        *i += 1;
                        return Some(J::Arr(v));
                    }
                    _ => return None,
                }
            }
        }
        b'{' => {
            *i += 1;
            let mut kv = vec![];
            ws(b, i);
            if b.get(*i) == Some(&b'}') {
                *i += 1;
                return Some(J::Obj(kv));
            }
            loop {
                ws(b, i);
                let k = string(b, i)?;
                ws(b, i);
                if b.get(*i) != Some(&b':') {
                    return None;
                }
                *i += 1;
                kv.push((k, value(b, i)?));
                ws(b, i);
                match b.get(*i)? {
                    b',' => *i += 1,
                    b'}' => {
                        *i += 1;
                        return Some(J::Obj(kv));
                    }
                    _ => return None,
                }
            }
        }
        _ => {
            let start = *i;
            while *i < b.len() && (b[*i] == b'-' || b[*i] == b'+' || b[*i] == b'.' || b[*i] == b'e' || b[*i] == b'E' || b[*i].is_ascii_digit()) {
                *i += 1;
            }
            (*i > start).then(|| J::Num(String::from_utf8_lossy(&b[start..*i]).into_owned()))
        }
    }
}

fn lit(b: &[u8], i: &mut usize, word: &str, v: J) -> Option<J> {
    if b[*i..].starts_with(word.as_bytes()) {
        *i += word.len();
        Some(v)
    } else {
        None
    }
}

fn string(b: &[u8], i: &mut usize) -> Option<String> {
    if b.get(*i) != Some(&b'"') {
        return None;
    }
    *i += 1;
    let mut out = String::new();
    while let Some(&c) = b.get(*i) {
        *i += 1;
        match c {
            b'"' => return Some(out),
            b'\\' => {
                let e = *b.get(*i)?;
                *i += 1;
                match e {
                    b'n' => out.push('\n'),
                    b't' => out.push('\t'),
                    b'u' => {
                        let hex = std::str::from_utf8(b.get(*i..*i + 4)?).ok()?;
                        *i += 4;
                        out.push(char::from_u32(u32::from_str_radix(hex, 16).ok()?).unwrap_or('?'));
                    }
                    other => out.push(other as char),
                }
            }
            _ => {
                // multi-byte UTF-8: copy the raw bytes of this character
                let start = *i - 1;
                let len = match c {
                    0xC0..=0xDF => 2,
                    0xE0..=0xEF => 3,
                    0xF0..=0xF7 => 4,
                    _ => 1,
                };
                *i = start + len;
                out.push_str(&String::from_utf8_lossy(b.get(start..start + len)?));
            }
        }
    }
    None
}
