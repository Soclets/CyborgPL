//! Source files and spans. A span is a byte range in one file; line and
//! column are computed from it when a message is printed, never stored on
//! the tree (so a comment shifts spans and nothing else — `precog.md`,
//! Reuse: spans live in a side table and are not hashed).

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(file: FileId, start: usize, end: usize) -> Span {
        Span { file, start: start as u32, end: end as u32 }
    }
    /// The span from the start of `self` to the end of `other`.
    pub fn to(self, other: Span) -> Span {
        Span { file: self.file, start: self.start.min(other.start), end: self.end.max(other.end) }
    }
    pub fn len(self) -> usize {
        (self.end - self.start) as usize
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub id: FileId,
    pub path: PathBuf,
    pub text: String,
    /// Byte offset of the start of every line.
    line_starts: Vec<u32>,
}

impl SourceFile {
    pub fn new(id: FileId, path: PathBuf, text: String) -> SourceFile {
        let mut line_starts = vec![0u32];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i as u32 + 1);
            }
        }
        SourceFile { id, path, text, line_starts }
    }

    /// 1-based line and column (column in graphemes) of a byte offset.
    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        let line = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let start = self.line_starts[line] as usize;
        // Graphemes, as the reader sees them (owner, 2026-10-06): a Thai
        // vowel mark or an emoji's parts are one column with their base.
        let col = unicode_segmentation::UnicodeSegmentation::graphemes(&self.text[start..(offset as usize).min(self.text.len())], true).count();
        (line as u32 + 1, col as u32 + 1)
    }

    /// The same, the column in UTF-16 code units: the JSON form's, as the
    /// Language Server Protocol counts (owner, 2026-10-06: "yea").
    pub fn line_col_utf16(&self, offset: u32) -> (u32, u32) {
        let (line, _) = self.line_col(offset);
        let start = self.line_starts[line as usize - 1] as usize;
        let col = self.text[start..(offset as usize).min(self.text.len())].encode_utf16().count();
        (line, col as u32 + 1)
    }

    pub fn line_text(&self, line: u32) -> &str {
        let i = (line - 1) as usize;
        let start = self.line_starts[i] as usize;
        let end = self.line_starts.get(i + 1).map(|&e| e as usize).unwrap_or(self.text.len());
        self.text[start..end].trim_end_matches(['\n', '\r'])
    }

    pub fn slice(&self, span: Span) -> &str {
        &self.text[span.start as usize..span.end as usize]
    }

    /// The name a message shows: the path as given.
    pub fn display_name(&self) -> String {
        self.path.display().to_string()
    }

    /// The file a second place in a message names (owner, 2026-10-08:
    /// every place a message names gives its file, "`a`", always): its
    /// name, the `where:` line holding the whole path.
    pub fn short_name(&self) -> String {
        self.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| self.display_name())
    }

    /// "line 6 of shop.cpls".
    pub fn line_of(&self, offset: u32) -> String {
        format!("line {} of {}", self.line_col(offset).0, self.short_name())
    }
}

/// Every source file of the build, indexed by `FileId`.
#[derive(Debug, Default)]
pub struct SourceMap {
    pub files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn add(&mut self, path: &Path, text: String) -> FileId {
        let id = FileId(self.files.len() as u32);
        self.files.push(SourceFile::new(id, path.to_path_buf(), text));
        id
    }
    pub fn get(&self, id: FileId) -> &SourceFile {
        &self.files[id.0 as usize]
    }
    pub fn slice(&self, span: Span) -> &str {
        self.get(span.file).slice(span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A message counts graphemes; the JSON form UTF-16 code units (owner,
    /// 2026-10-06). Before ` :=` stand `let '` (5), `น้ำ` (three code
    /// points, one grapheme, three units), `🍕` (one grapheme, two units)
    /// and `'`.
    #[test]
    fn columns_count_graphemes_and_utf16() {
        let text = "let 'น้ำ🍕' := x;\nok".to_string();
        let f = SourceFile::new(FileId(0), PathBuf::from("t.cpls"), text.clone());
        let at = text.find(" :=").unwrap() as u32;
        assert_eq!(f.line_col(at), (1, 9));
        assert_eq!(f.line_col_utf16(at), (1, 12));
        assert_eq!(f.line_col(text.find("ok").unwrap() as u32), (2, 1));
    }
}
