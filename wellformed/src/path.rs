//! JSON Pointer (RFC 6901) with wildcard extension.
//!
//! Supports:
//! - Standard JSON Pointer syntax: `/foo/bar/0`
//! - Wildcard segment `*` for "all array elements": `/items/*/name`
//! - Empty pointer `` for root

use crate::error::{Result, WelError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A segment in a JSON Pointer path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Segment {
    /// Object property key.
    Key(String),
    /// Array index.
    Index(usize),
    /// Wildcard - matches all array elements.
    Wildcard,
}

impl Segment {
    /// Parse a single segment string.
    fn parse(s: &str) -> Self {
        if s == "*" {
            Segment::Wildcard
        } else if let Ok(idx) = s.parse::<usize>() {
            Segment::Index(idx)
        } else {
            // Unescape JSON Pointer escapes: ~1 -> /, ~0 -> ~
            let unescaped = s.replace("~1", "/").replace("~0", "~");
            Segment::Key(unescaped)
        }
    }
}

/// A parsed JSON Pointer with extended wildcard support.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct JsonPointer {
    segments: Vec<Segment>,
}

impl JsonPointer {
    /// Create an empty pointer (references root).
    pub fn root() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Parse a JSON Pointer string.
    ///
    /// # Examples
    /// ```
    /// use wellformed::path::JsonPointer;
    ///
    /// let ptr = JsonPointer::parse("/foo/bar/0").unwrap();
    /// let ptr_wild = JsonPointer::parse("/items/*/name").unwrap();
    /// ```
    pub fn parse(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Ok(Self::root());
        }

        if !s.starts_with('/') {
            return Err(WelError::InvalidPointer(format!(
                "JSON pointer must start with '/' or be empty: {s}"
            )));
        }

        let segments = s[1..].split('/').map(Segment::parse).collect();

        Ok(Self { segments })
    }

    /// Get the segments of this pointer.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Check if this pointer is empty (references root).
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Check if this pointer contains any wildcards.
    pub fn has_wildcards(&self) -> bool {
        self.segments.iter().any(|s| matches!(s, Segment::Wildcard))
    }

    /// Join this pointer with another (relative) pointer.
    pub fn join(&self, other: &JsonPointer) -> JsonPointer {
        let mut segments = self.segments.clone();
        segments.extend(other.segments.iter().cloned());
        JsonPointer { segments }
    }

    /// Push a segment onto this pointer.
    pub fn push(&mut self, segment: Segment) {
        self.segments.push(segment);
    }

    /// Push a key segment.
    pub fn push_key(&mut self, key: impl Into<String>) {
        self.segments.push(Segment::Key(key.into()));
    }

    /// Push an index segment.
    pub fn push_index(&mut self, index: usize) {
        self.segments.push(Segment::Index(index));
    }

    /// Resolve this pointer against a JSON value.
    ///
    /// Returns all matching values (multiple if wildcards are used).
    pub fn resolve<'a>(&self, value: &'a Value) -> Vec<&'a Value> {
        self.resolve_recursive(value, 0)
    }

    fn resolve_recursive<'a>(&self, value: &'a Value, depth: usize) -> Vec<&'a Value> {
        if depth >= self.segments.len() {
            return vec![value];
        }

        match &self.segments[depth] {
            Segment::Key(key) => {
                if let Some(v) = value.get(key) {
                    self.resolve_recursive(v, depth + 1)
                } else {
                    vec![]
                }
            }
            Segment::Index(idx) => {
                if let Some(v) = value.get(idx) {
                    self.resolve_recursive(v, depth + 1)
                } else {
                    vec![]
                }
            }
            Segment::Wildcard => {
                if let Some(arr) = value.as_array() {
                    arr.iter()
                        .flat_map(|v| self.resolve_recursive(v, depth + 1))
                        .collect()
                } else {
                    vec![]
                }
            }
        }
    }

    /// Resolve this pointer and return mutable references.
    ///
    /// Note: Wildcards are not supported for mutable resolution.
    pub fn resolve_mut<'a>(&self, value: &'a mut Value) -> Option<&'a mut Value> {
        if self.has_wildcards() {
            return None;
        }

        let mut current = value;
        for segment in &self.segments {
            current = match segment {
                Segment::Key(key) => current.get_mut(key)?,
                Segment::Index(idx) => current.get_mut(idx)?,
                Segment::Wildcard => return None,
            };
        }
        Some(current)
    }

    /// Resolve this pointer with path tracking.
    ///
    /// Returns pairs of (concrete_path, value) where concrete_path has
    /// wildcards replaced with actual indices.
    pub fn resolve_with_paths<'a>(&self, value: &'a Value) -> Vec<(JsonPointer, &'a Value)> {
        self.resolve_with_paths_recursive(value, 0, JsonPointer::root())
    }

    fn resolve_with_paths_recursive<'a>(
        &self,
        value: &'a Value,
        depth: usize,
        current_path: JsonPointer,
    ) -> Vec<(JsonPointer, &'a Value)> {
        if depth >= self.segments.len() {
            return vec![(current_path, value)];
        }

        match &self.segments[depth] {
            Segment::Key(key) => {
                if let Some(v) = value.get(key) {
                    let mut path = current_path;
                    path.push(Segment::Key(key.clone()));
                    self.resolve_with_paths_recursive(v, depth + 1, path)
                } else {
                    vec![]
                }
            }
            Segment::Index(idx) => {
                if let Some(v) = value.get(idx) {
                    let mut path = current_path;
                    path.push(Segment::Index(*idx));
                    self.resolve_with_paths_recursive(v, depth + 1, path)
                } else {
                    vec![]
                }
            }
            Segment::Wildcard => {
                if let Some(arr) = value.as_array() {
                    arr.iter()
                        .enumerate()
                        .flat_map(|(i, v)| {
                            let mut path = current_path.clone();
                            path.push(Segment::Index(i));
                            self.resolve_with_paths_recursive(v, depth + 1, path)
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
        }
    }
}

impl std::fmt::Display for JsonPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.segments.is_empty() {
            Ok(())
        } else {
            write!(f, "/")?;
            for (i, segment) in self.segments.iter().enumerate() {
                if i > 0 {
                    write!(f, "/")?;
                }
                match segment {
                    Segment::Key(k) => {
                        // Escape: ~ -> ~0, / -> ~1
                        write!(f, "{}", k.replace('~', "~0").replace('/', "~1"))?;
                    }
                    Segment::Index(idx) => write!(f, "{}", idx)?,
                    Segment::Wildcard => write!(f, "*")?,
                }
            }
            Ok(())
        }
    }
}

impl Serialize for JsonPointer {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, "{}", self).expect("formatting to string failed");
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for JsonPointer {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        JsonPointer::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_empty() {
        let ptr = JsonPointer::parse("").unwrap();
        assert!(ptr.is_empty());
        assert_eq!(ptr.to_string(), "");
    }

    #[test]
    fn test_parse_simple() {
        let ptr = JsonPointer::parse("/foo/bar").unwrap();
        assert_eq!(ptr.segments.len(), 2);
        assert_eq!(ptr.segments[0], Segment::Key("foo".to_string()));
        assert_eq!(ptr.segments[1], Segment::Key("bar".to_string()));
        assert_eq!(ptr.to_string(), "/foo/bar");
    }

    #[test]
    fn test_parse_with_index() {
        let ptr = JsonPointer::parse("/items/0/name").unwrap();
        assert_eq!(ptr.segments.len(), 3);
        assert_eq!(ptr.segments[0], Segment::Key("items".to_string()));
        assert_eq!(ptr.segments[1], Segment::Index(0));
        assert_eq!(ptr.segments[2], Segment::Key("name".to_string()));
    }

    #[test]
    fn test_parse_with_wildcard() {
        let ptr = JsonPointer::parse("/items/*/name").unwrap();
        assert!(ptr.has_wildcards());
        assert_eq!(ptr.segments[1], Segment::Wildcard);
    }

    #[test]
    fn test_parse_escaped() {
        let ptr = JsonPointer::parse("/foo~1bar/baz~0qux").unwrap();
        assert_eq!(ptr.segments[0], Segment::Key("foo/bar".to_string()));
        assert_eq!(ptr.segments[1], Segment::Key("baz~qux".to_string()));
    }

    #[test]
    fn test_parse_invalid() {
        assert!(JsonPointer::parse("foo/bar").is_err());
    }

    #[test]
    fn test_resolve_simple() {
        let value = json!({
            "foo": {
                "bar": 42
            }
        });
        let ptr = JsonPointer::parse("/foo/bar").unwrap();
        let results = ptr.resolve(&value);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], &json!(42));
    }

    #[test]
    fn test_resolve_array_index() {
        let value = json!({
            "items": ["a", "b", "c"]
        });
        let ptr = JsonPointer::parse("/items/1").unwrap();
        let results = ptr.resolve(&value);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], &json!("b"));
    }

    #[test]
    fn test_resolve_wildcard() {
        let value = json!({
            "items": [
                {"name": "alice"},
                {"name": "bob"},
                {"name": "charlie"}
            ]
        });
        let ptr = JsonPointer::parse("/items/*/name").unwrap();
        let results = ptr.resolve(&value);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], &json!("alice"));
        assert_eq!(results[1], &json!("bob"));
        assert_eq!(results[2], &json!("charlie"));
    }

    #[test]
    fn test_resolve_with_paths() {
        let value = json!({
            "items": [
                {"name": "alice"},
                {"name": "bob"}
            ]
        });
        let ptr = JsonPointer::parse("/items/*/name").unwrap();
        let results = ptr.resolve_with_paths(&value);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0.to_string(), "/items/0/name");
        assert_eq!(results[0].1, &json!("alice"));
        assert_eq!(results[1].0.to_string(), "/items/1/name");
        assert_eq!(results[1].1, &json!("bob"));
    }

    #[test]
    fn test_join() {
        let base = JsonPointer::parse("/foo").unwrap();
        let rel = JsonPointer::parse("/bar/baz").unwrap();
        let joined = base.join(&rel);
        assert_eq!(joined.to_string(), "/foo/bar/baz");
    }

    #[test]
    fn test_serde_roundtrip() {
        let ptr = JsonPointer::parse("/foo/*/bar").unwrap();
        let json = serde_json::to_string(&ptr).unwrap();
        assert_eq!(json, "\"/foo/*/bar\"");
        let parsed: JsonPointer = serde_json::from_str(&json).unwrap();
        assert_eq!(ptr, parsed);
    }
}
