//! The store: the app's data, kept apart from the UI. The UI is the agent's and a push replaces
//! it; the store is shared by the person and every agent, and nothing but a write changes it.
//! Every entry has a revision, so a write can say which one it was based on and be refused
//! when someone else wrote in between. Text an editor shows is an entry too (a caretline
//! document: the text and its undo history); the carets and scroll stay with each editor.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    /// Goes up by one with every write, from anyone.
    pub rev: u64,
    /// Who wrote last: `seed` (the UI's starting value), `person` (typing), or an agent's name.
    pub by: String,
    #[serde(flatten)]
    pub body: Body,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Body {
    /// Text an editor shows: the document, with its undo history.
    Text { doc: Box<caretline::Document> },
    /// Any JSON value: a list's rows, a chart's numbers, a record.
    Value { value: Value },
}

/// A write to one entry. Text entries take `Text` and `Edit`; any entry takes `Put` of a string
/// for text or a value otherwise; `Merge` merges an object into a value (RFC 7396).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "write", rename_all = "lowercase")]
pub enum Write {
    Put { value: Value },
    Merge { value: Value },
    /// Ranges of the current text replaced, in chars, in order and apart: `[from, to, text]`.
    Edit { changes: Vec<(usize, usize, String)> },
    /// The entry goes: a seed of the same name can start it again, and a source of the same
    /// name is no longer hidden by it.
    Delete {},
}

impl Entry {
    pub fn value(value: Value, by: &str) -> Entry {
        Entry { rev: 1, by: by.into(), body: Body::Value { value } }
    }

    pub fn text(text: &str, by: &str) -> Entry {
        let mut doc = caretline::Document::new(text, None);
        doc.sanitize();
        Entry { rev: 1, by: by.into(), body: Body::Text { doc: Box::new(doc) } }
    }

    /// What an agent reads back: the revision, who wrote last, and the text or the value.
    pub fn read(&self) -> Value {
        let mut out = serde_json::json!({"rev": self.rev, "by": self.by});
        match &self.body {
            Body::Text { doc } => out["text"] = doc.text.to_string().into(),
            Body::Value { value } => out["value"] = value.clone(),
        }
        out
    }
}

/// RFC 7396: `patch`'s keys replace `target`'s, `null` removes one, objects merge deeply.
pub fn merge(target: &mut Value, patch: &Value) {
    let Value::Object(p) = patch else {
        *target = patch.clone();
        return;
    };
    if !target.is_object() {
        *target = Value::Object(Default::default());
    }
    let t = target.as_object_mut().expect("an object");
    for (k, v) in p {
        if v.is_null() {
            t.remove(k);
        } else {
            merge(t.entry(k.clone()).or_insert(Value::Null), v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_follows_rfc_7396() {
        let mut v = json!({"a": 1, "b": {"c": 2, "d": 3}});
        merge(&mut v, &json!({"a": null, "b": {"c": 9}, "e": [1]}));
        assert_eq!(v, json!({"b": {"c": 9, "d": 3}, "e": [1]}));
    }

    #[test]
    fn entries_round_trip_through_json() {
        for e in [Entry::text("a note", "seed"), Entry::value(json!([1, 2]), "agent")] {
            let back: Entry = serde_json::from_value(serde_json::to_value(&e).unwrap()).unwrap();
            assert_eq!(back.read(), e.read());
        }
    }
}
