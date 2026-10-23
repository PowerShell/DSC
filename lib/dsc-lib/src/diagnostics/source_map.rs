// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Locate JSON pointers in the text of a JSON or YAML document.

use std::{collections::HashMap, ops::Range};

/// The spans of one location in a document.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    /// The span of the member name, when the value is a member of an object.
    key: Option<Range<usize>>,
    /// The span of the value.
    value: Range<usize>,
}

/// The text of a document together with the byte span of every location in it, indexed by JSON
/// pointer (RFC 6901).
///
/// A source map lets a diagnostic point at the exact place in a document that a pointer
/// identifies, both as a byte range for rendering a snippet and as a line and column.
#[derive(Clone, Debug)]
pub struct SourceMap {
    text: String,
    entries: HashMap<String, Entry>,
    line_starts: Vec<usize>,
}

impl SourceMap {
    /// Build a source map from document text.
    ///
    /// Text that is valid JSON is mapped with the JSON grammar; anything else is mapped as YAML.
    ///
    /// # Arguments
    ///
    /// * `text` - The document text.
    ///
    /// # Returns
    ///
    /// The source map, or `None` when the text is neither valid JSON nor valid YAML.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let entries = if serde_json::from_str::<serde::de::IgnoredAny>(text).is_ok() {
            json::scan(text)?
        } else {
            yaml::scan(text)?
        };
        Some(Self {
            text: text.to_string(),
            entries,
            line_starts: line_starts(text),
        })
    }

    /// The document text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether the document has a value at the pointer.
    ///
    /// # Arguments
    ///
    /// * `pointer` - The JSON pointer; the empty pointer is the document root.
    #[must_use]
    pub fn contains(&self, pointer: &str) -> bool {
        self.entries.contains_key(pointer)
    }

    /// The span of the value at the pointer.
    ///
    /// # Arguments
    ///
    /// * `pointer` - The JSON pointer; the empty pointer is the document root.
    #[must_use]
    pub fn value_span(&self, pointer: &str) -> Option<Range<usize>> {
        self.entries.get(pointer).map(|entry| entry.value.clone())
    }

    /// The span of the member name of the value at the pointer, when it is a member of an object.
    ///
    /// # Arguments
    ///
    /// * `pointer` - The JSON pointer.
    #[must_use]
    pub fn key_span(&self, pointer: &str) -> Option<Range<usize>> {
        self.entries.get(pointer).and_then(|entry| entry.key.clone())
    }

    /// The span that best identifies the pointer: the member name when the value is a member of
    /// an object, otherwise the value.
    ///
    /// # Arguments
    ///
    /// * `pointer` - The JSON pointer.
    #[must_use]
    pub fn span(&self, pointer: &str) -> Option<Range<usize>> {
        self.entries.get(pointer).map(|entry| entry.key.clone().unwrap_or_else(|| entry.value.clone()))
    }

    /// The span of the nearest location that exists in the document, walking up from the pointer
    /// to its ancestors.
    ///
    /// Useful for a pointer into a value the document doesn't spell out, like a property that is
    /// missing or a character offset inside a string.
    ///
    /// # Arguments
    ///
    /// * `pointer` - The JSON pointer.
    ///
    /// # Returns
    ///
    /// The pointer that was found and its span.
    #[must_use]
    pub fn nearest_span(&self, pointer: &str) -> Option<(String, Range<usize>)> {
        let mut current = pointer;
        loop {
            if let Some(span) = self.span(current) {
                return Some((current.to_string(), span));
            }
            let parent_end = current.rfind('/')?;
            current = &current[..parent_end];
        }
    }

    /// The line and column of a byte offset, both starting at 1. Columns count characters.
    ///
    /// # Arguments
    ///
    /// * `offset` - The byte offset into the text.
    #[must_use]
    pub fn location(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.text.len());
        let line_index = match self.line_starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        let line_start = self.line_starts[line_index];
        let column = self.text[line_start..offset].chars().count() + 1;
        (line_index + 1, column)
    }
}

/// The byte offset where each line starts.
fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

/// Escape a member name as a JSON pointer reference token.
fn escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

mod json {
    use super::{Entry, escape};
    use std::{collections::HashMap, ops::Range};

    /// Record the span of every value in valid JSON text.
    pub(super) fn scan(text: &str) -> Option<HashMap<String, Entry>> {
        let mut scanner = Scanner { text, bytes: text.as_bytes(), pos: 0, entries: HashMap::new() };
        scanner.value("", None)?;
        Some(scanner.entries)
    }

    struct Scanner<'a> {
        text: &'a str,
        bytes: &'a [u8],
        pos: usize,
        entries: HashMap<String, Entry>,
    }

    impl Scanner<'_> {
        fn skip_whitespace(&mut self) {
            while self.pos < self.bytes.len() && matches!(self.bytes[self.pos], b' ' | b'\t' | b'\n' | b'\r') {
                self.pos += 1;
            }
        }

        fn expect(&mut self, byte: u8) -> Option<()> {
            self.skip_whitespace();
            if self.bytes.get(self.pos) == Some(&byte) {
                self.pos += 1;
                Some(())
            } else {
                None
            }
        }

        fn value(&mut self, pointer: &str, key: Option<Range<usize>>) -> Option<Range<usize>> {
            self.skip_whitespace();
            let start = self.pos;
            let span = match self.bytes.get(self.pos)? {
                b'{' => self.object(pointer)?,
                b'[' => self.array(pointer)?,
                b'"' => self.string()?,
                _ => {
                    while self.pos < self.bytes.len() && !matches!(self.bytes[self.pos], b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r') {
                        self.pos += 1;
                    }
                    start..self.pos
                },
            };
            self.entries.insert(pointer.to_string(), Entry { key, value: span.clone() });
            Some(span)
        }

        fn object(&mut self, pointer: &str) -> Option<Range<usize>> {
            let start = self.pos;
            self.pos += 1;
            loop {
                self.skip_whitespace();
                match self.bytes.get(self.pos)? {
                    b'}' => {
                        self.pos += 1;
                        return Some(start..self.pos);
                    },
                    b',' => {
                        self.pos += 1;
                    },
                    b'"' => {
                        let key_span = self.string()?;
                        let key: String = serde_json::from_str(&self.text[key_span.clone()]).ok()?;
                        self.expect(b':')?;
                        self.value(&format!("{pointer}/{}", escape(&key)), Some(key_span))?;
                    },
                    _ => return None,
                }
            }
        }

        fn array(&mut self, pointer: &str) -> Option<Range<usize>> {
            let start = self.pos;
            self.pos += 1;
            let mut index = 0;
            loop {
                self.skip_whitespace();
                match self.bytes.get(self.pos)? {
                    b']' => {
                        self.pos += 1;
                        return Some(start..self.pos);
                    },
                    b',' => {
                        self.pos += 1;
                    },
                    _ => {
                        self.value(&format!("{pointer}/{index}"), None)?;
                        index += 1;
                    },
                }
            }
        }

        fn string(&mut self) -> Option<Range<usize>> {
            let start = self.pos;
            self.pos += 1;
            while let Some(byte) = self.bytes.get(self.pos) {
                match byte {
                    b'\\' => self.pos += 2,
                    b'"' => {
                        self.pos += 1;
                        return Some(start..self.pos);
                    },
                    _ => self.pos += 1,
                }
            }
            None
        }
    }
}

mod yaml {
    use super::{Entry, escape, line_starts};
    use saphyr_parser::{Event, Marker, Parser, Span};
    use std::{collections::HashMap, ops::Range};

    /// A container whose members are being read.
    enum Frame {
        Mapping {
            pointer: String,
            key: Option<Range<usize>>,
            /// The member name the next value belongs to: `None` while expecting a key, and
            /// `Some(None)` for a key that isn't a scalar and so can't be addressed by a pointer.
            pending: Option<Option<(String, Range<usize>)>>,
            start: usize,
            end: usize,
        },
        Sequence {
            pointer: String,
            key: Option<Range<usize>>,
            index: usize,
            start: usize,
            end: usize,
        },
    }

    /// Record the span of every node in the first document of YAML text.
    pub(super) fn scan(text: &str) -> Option<HashMap<String, Entry>> {
        let offsets = Offsets::new(text);
        let mut entries = HashMap::new();
        let mut frames: Vec<Frame> = Vec::new();
        // Nesting depth inside a key that is a collection, whose events are skipped.
        let mut skipping = 0usize;
        let mut root_done = false;

        for item in Parser::new_from_str(text) {
            let (event, span) = item.ok()?;
            let range = offsets.range(&span);
            match event {
                Event::StreamStart | Event::StreamEnd | Event::Nothing => {},
                Event::DocumentStart(_) => {
                    if root_done {
                        break;
                    }
                },
                Event::DocumentEnd => root_done = true,
                Event::Scalar(..) | Event::Alias(..) => {
                    if skipping > 0 {
                        continue;
                    }
                    let key_text = match &event {
                        Event::Scalar(value, ..) => Some(value.to_string()),
                        _ => None,
                    };
                    match frames.last_mut() {
                        Some(Frame::Mapping { pending: pending @ None, .. }) => {
                            *pending = Some(key_text.map(|value| (value, range)));
                        },
                        Some(frame) => {
                            let (pointer, key) = child(frame, range.end);
                            if let Some(pointer) = pointer {
                                entries.insert(pointer, Entry { key, value: range });
                            }
                        },
                        None => {
                            entries.insert(String::new(), Entry { key: None, value: range });
                        },
                    }
                },
                Event::SequenceStart(..) | Event::MappingStart(..) => {
                    if skipping > 0 {
                        skipping += 1;
                        continue;
                    }
                    if let Some(Frame::Mapping { pending: pending @ None, .. }) = frames.last_mut() {
                        *pending = Some(None);
                        skipping = 1;
                        continue;
                    }
                    let (pointer, key) = match frames.last_mut() {
                        Some(frame) => child(frame, range.end),
                        None => (Some(String::new()), None),
                    };
                    let Some(pointer) = pointer else {
                        skipping = 1;
                        continue;
                    };
                    frames.push(if matches!(event, Event::SequenceStart(..)) {
                        Frame::Sequence { pointer, key, index: 0, start: range.start, end: range.end }
                    } else {
                        Frame::Mapping { pointer, key, pending: None, start: range.start, end: range.end }
                    });
                },
                Event::SequenceEnd | Event::MappingEnd => {
                    if skipping > 0 {
                        skipping -= 1;
                        continue;
                    }
                    let frame = frames.pop()?;
                    let (pointer, key, start, end) = match frame {
                        Frame::Mapping { pointer, key, start, end, .. } | Frame::Sequence { pointer, key, start, end, .. } => (pointer, key, start, end),
                    };
                    // A flow collection's closing bracket has a span of its own; a block
                    // collection ends with its last member.
                    let end = if range.end > range.start { range.end.max(end) } else { end.max(start) };
                    if let Some(parent) = frames.last_mut() {
                        extend(parent, end);
                    }
                    entries.insert(pointer, Entry { key, value: start..end });
                },
            }
        }
        Some(entries)
    }

    /// Resolve the pointer and member-name span of the next value in a container, and advance
    /// the container past it.
    fn child(frame: &mut Frame, end: usize) -> (Option<String>, Option<Range<usize>>) {
        match frame {
            Frame::Mapping { pointer, pending, end: frame_end, .. } => {
                *frame_end = (*frame_end).max(end);
                match pending.take().flatten() {
                    Some((key, span)) => (Some(format!("{pointer}/{}", escape(&key))), Some(span)),
                    None => (None, None),
                }
            },
            Frame::Sequence { pointer, index, end: frame_end, .. } => {
                *frame_end = (*frame_end).max(end);
                let child = format!("{pointer}/{index}");
                *index += 1;
                (Some(child), None)
            },
        }
    }

    fn extend(frame: &mut Frame, end: usize) {
        match frame {
            Frame::Mapping { end: frame_end, .. } | Frame::Sequence { end: frame_end, .. } => {
                *frame_end = (*frame_end).max(end);
            },
        }
    }

    /// Converts the parser's line and column markers into byte offsets.
    struct Offsets<'a> {
        text: &'a str,
        line_starts: Vec<usize>,
    }

    impl<'a> Offsets<'a> {
        fn new(text: &'a str) -> Self {
            Self { text, line_starts: line_starts(text) }
        }

        fn range(&self, span: &Span) -> Range<usize> {
            let start = self.offset(&span.start);
            let end = self.offset(&span.end).max(start);
            start..end
        }

        /// The byte offset of a marker, whose line starts at 1 and whose column starts at 0 and
        /// counts characters.
        fn offset(&self, marker: &Marker) -> usize {
            let Some(&line_start) = self.line_starts.get(marker.line().saturating_sub(1)) else {
                return self.text.len();
            };
            let line = &self.text[line_start..];
            let line = &line[..line.find('\n').unwrap_or(line.len())];
            let column_offset = line.char_indices().nth(marker.col()).map_or(line.len(), |(offset, _)| offset);
            line_start + column_offset
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_at(map: &SourceMap, span: Range<usize>) -> &str {
        &map.text()[span]
    }

    #[test]
    fn json_locates_keys_values_and_nested_items() {
        let map = SourceMap::parse(r#"{
  "a": 1,
  "b": [true, {"c~/d": "x"}],
  "e": {"f": null}
}"#).unwrap();
        assert_eq!(text_at(&map, map.key_span("/a").unwrap()), "\"a\"");
        assert_eq!(text_at(&map, map.value_span("/a").unwrap()), "1");
        assert_eq!(text_at(&map, map.value_span("/b/0").unwrap()), "true");
        assert_eq!(text_at(&map, map.key_span("/b/1/c~0~1d").unwrap()), "\"c~/d\"");
        assert_eq!(text_at(&map, map.value_span("/b/1/c~0~1d").unwrap()), "\"x\"");
        assert_eq!(text_at(&map, map.value_span("/e").unwrap()), "{\"f\": null}");
        assert_eq!(text_at(&map, map.value_span("/b").unwrap()), "[true, {\"c~/d\": \"x\"}]");
        assert_eq!(map.value_span("").unwrap().start, 0);
        assert!(map.value_span("/missing").is_none());
        assert_eq!(map.location(map.key_span("/b").unwrap().start), (3, 3));
    }

    #[test]
    fn json_handles_escaped_strings() {
        let map = SourceMap::parse(r#"{"k\"ey": "va\\lue", "n": -1.5e3}"#).unwrap();
        assert_eq!(text_at(&map, map.value_span("/k\"ey").unwrap()), r#""va\\lue""#);
        assert_eq!(text_at(&map, map.value_span("/n").unwrap()), "-1.5e3");
    }

    #[test]
    fn yaml_locates_block_mappings_and_sequences() {
        let text = "$schema: https://example\nresources:\n- name: first\n  type: Test/Echo\n  properties:\n    output: hi\n- name: second\n  type: Test/Echo\n";
        let map = SourceMap::parse(text).unwrap();
        assert_eq!(text_at(&map, map.key_span("/resources/0/properties/output").unwrap()), "output");
        assert_eq!(text_at(&map, map.value_span("/resources/0/properties/output").unwrap()), "hi");
        assert_eq!(text_at(&map, map.value_span("/resources/1/name").unwrap()), "second");
        assert_eq!(text_at(&map, map.key_span("/resources").unwrap()), "resources");
        let item = map.value_span("/resources/0").unwrap();
        assert!(text_at(&map, item.clone()).starts_with("name: first"));
        assert!(text_at(&map, item).ends_with("output: hi"));
        assert_eq!(map.location(map.key_span("/resources/1/type").unwrap().start), (8, 3));
    }

    #[test]
    fn yaml_locates_flow_collections_and_quoted_scalars() {
        let text = "a: {b: [1, 2], 'c d': \"q\"}\n";
        let map = SourceMap::parse(text).unwrap();
        assert_eq!(text_at(&map, map.value_span("/a/b/1").unwrap()), "2");
        assert_eq!(text_at(&map, map.key_span("/a/c d").unwrap()), "'c d'");
        assert_eq!(text_at(&map, map.value_span("/a/c d").unwrap()), "\"q\"");
        assert_eq!(text_at(&map, map.value_span("/a").unwrap()), "{b: [1, 2], 'c d': \"q\"}");
    }

    #[test]
    fn yaml_counts_columns_in_characters() {
        let text = "héllo: wörld\nnext: 1\n";
        let map = SourceMap::parse(text).unwrap();
        assert_eq!(text_at(&map, map.value_span("/héllo").unwrap()), "wörld");
        assert_eq!(text_at(&map, map.value_span("/next").unwrap()), "1");
    }

    #[test]
    fn yaml_skips_complex_keys() {
        let text = "? [a, b]\n: value\nplain: 1\n";
        let map = SourceMap::parse(text).unwrap();
        assert_eq!(text_at(&map, map.value_span("/plain").unwrap()), "1");
    }

    #[test]
    fn nearest_span_walks_up_to_an_ancestor() {
        let map = SourceMap::parse("resources:\n- name: a\n").unwrap();
        let (pointer, span) = map.nearest_span("/resources/0/properties/output").unwrap();
        assert_eq!(pointer, "/resources/0");
        assert_eq!(text_at(&map, span), "name: a");
        assert!(map.nearest_span("/resources/5").is_some());
    }

    #[test]
    fn invalid_text_has_no_map() {
        assert!(SourceMap::parse("a: [b").is_none());
    }
}
