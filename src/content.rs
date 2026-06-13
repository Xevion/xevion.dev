//! Detail-page body model: an ordered list of addressable blocks.
//!
//! A block is `{ id, type, data }` where `data` is opaque to the backend — all
//! type-specific meaning (rendering, validation, editing) lives in the frontend
//! registry. The backend only enforces structural invariants (unique ids,
//! non-empty types, resolvable anchors) and applies one mutation at a time, so
//! clients never resend the whole document.

use serde::{Deserialize, Serialize};

/// One content block. `data` carries the type-specific payload (e.g. a prose
/// block's markdown) and is never interpreted here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub data: serde_json::Value,
}

/// A new block's content for `insert`. The store owns the `id`, so it isn't
/// part of the input; `type` is required.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockInput {
    #[serde(rename = "type")]
    pub r#type: String,
    pub data: serde_json::Value,
}

/// A `set` payload. `data` always replaces the block's data; `type` is optional
/// — omit it to change data while keeping the block's existing type, or supply
/// it to retype the block in place.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockPatch {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    pub data: serde_json::Value,
}

/// Where an insert or move lands, relative to the existing block list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "at", rename_all = "snake_case")]
pub enum Anchor {
    Start,
    End,
    After { id: String },
    Before { id: String },
}

/// The body document: an ordered list of blocks. Stored as the project's
/// `detail_content` JSONB.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentDoc {
    pub blocks: Vec<Block>,
}

/// A single mutating operation, applied server-side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ContentOp {
    Set { id: String, block: BlockPatch },
    Insert { anchor: Anchor, block: BlockInput },
    Delete { id: String },
    Move { id: String, anchor: Anchor },
}

/// Structural failures applying an op. Type-semantic validation is the
/// frontend's job; this is only about the block list's integrity.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentError {
    /// No block with this id exists (target of set/delete/move, or an anchor).
    NotFound(String),
    /// A new block's id collides with an existing one.
    DuplicateId(String),
    /// A move that references itself as its own anchor.
    SelfAnchor,
    /// A block's `type` is empty or whitespace.
    EmptyType,
}

fn validate_type(r#type: &str) -> Result<(), ContentError> {
    if r#type.trim().is_empty() {
        return Err(ContentError::EmptyType);
    }
    Ok(())
}

/// A short, URL- and CLI-friendly block id. Lowercase alphanumerics keep it easy
/// to read aloud and pass on a command line; 8 chars over a 36-symbol alphabet is
/// ample for the handful of blocks in one document.
pub fn generate_block_id() -> String {
    const ALPHABET: [char; 36] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    ];
    nanoid::nanoid!(8, &ALPHABET)
}

impl ContentDoc {
    /// An empty document.
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    /// Parse a document out of a stored `detail_content` JSONB value. Missing
    /// (`None`) or anything that isn't a valid block document (e.g. a legacy
    /// TipTap doc) degrades to an empty document rather than failing — the page
    /// renders its shell instead of erroring.
    pub fn from_stored(value: Option<&serde_json::Value>) -> Self {
        value
            .and_then(|v| serde_json::from_value::<Self>(v.clone()).ok())
            .unwrap_or_default()
    }

    /// Serialize for storage in the project's `detail_content` JSONB. An empty
    /// document persists as `None` (SQL `NULL`) rather than `{"blocks":[]}`, so a
    /// project with no body reads as having no detail content.
    pub fn to_stored(&self) -> Option<serde_json::Value> {
        if self.blocks.is_empty() {
            None
        } else {
            Some(serde_json::to_value(self).expect("a ContentDoc always serializes"))
        }
    }

    /// Borrow a block by id.
    pub fn block(&self, id: &str) -> Option<&Block> {
        self.blocks.iter().find(|b| b.id == id)
    }

    fn index_of(&self, id: &str) -> Option<usize> {
        self.blocks.iter().position(|b| b.id == id)
    }

    /// Resolve an anchor to an insertion index in the current list.
    fn resolve_anchor(&self, anchor: &Anchor) -> Result<usize, ContentError> {
        match anchor {
            Anchor::Start => Ok(0),
            Anchor::End => Ok(self.blocks.len()),
            Anchor::After { id } => self
                .index_of(id)
                .map(|i| i + 1)
                .ok_or_else(|| ContentError::NotFound(id.clone())),
            Anchor::Before { id } => self
                .index_of(id)
                .ok_or_else(|| ContentError::NotFound(id.clone())),
        }
    }

    /// Apply a patch to a block in place, keeping its id and position. `data` is
    /// always replaced; `type` is replaced only if the patch supplies one (a
    /// `None` type leaves the block's existing type untouched).
    pub fn set(&mut self, id: &str, patch: BlockPatch) -> Result<(), ContentError> {
        if let Some(r#type) = &patch.r#type {
            validate_type(r#type)?;
        }
        let i = self
            .index_of(id)
            .ok_or_else(|| ContentError::NotFound(id.to_string()))?;
        if let Some(r#type) = patch.r#type {
            self.blocks[i].r#type = r#type;
        }
        self.blocks[i].data = patch.data;
        Ok(())
    }

    /// Insert a new block at `anchor`, assigning it `new_id`. Returns the
    /// created block.
    pub fn insert(
        &mut self,
        anchor: &Anchor,
        input: BlockInput,
        new_id: String,
    ) -> Result<Block, ContentError> {
        validate_type(&input.r#type)?;
        if self.index_of(&new_id).is_some() {
            return Err(ContentError::DuplicateId(new_id));
        }
        let at = self.resolve_anchor(anchor)?;
        let block = Block {
            id: new_id,
            r#type: input.r#type,
            data: input.data,
        };
        self.blocks.insert(at, block.clone());
        Ok(block)
    }

    /// Remove a block, returning it.
    pub fn delete(&mut self, id: &str) -> Result<Block, ContentError> {
        let i = self
            .index_of(id)
            .ok_or_else(|| ContentError::NotFound(id.to_string()))?;
        Ok(self.blocks.remove(i))
    }

    /// Move an existing block to `anchor`.
    pub fn move_block(&mut self, id: &str, anchor: &Anchor) -> Result<(), ContentError> {
        if let Anchor::After { id: target } | Anchor::Before { id: target } = anchor {
            if target == id {
                return Err(ContentError::SelfAnchor);
            }
        }
        let from = self
            .index_of(id)
            .ok_or_else(|| ContentError::NotFound(id.to_string()))?;
        // Resolve against the current list, then account for the gap the removal
        // leaves behind anything that sat after the moving block.
        let to = self.resolve_anchor(anchor)?;
        let block = self.blocks.remove(from);
        let adjusted = if to > from { to - 1 } else { to };
        self.blocks.insert(adjusted, block);
        Ok(())
    }

    /// Apply one operation, sourcing a fresh id from `gen_id` for inserts.
    /// Returns the affected block (created, updated, removed, or moved).
    pub fn apply(
        &mut self,
        op: ContentOp,
        gen_id: impl FnOnce() -> String,
    ) -> Result<Block, ContentError> {
        match op {
            ContentOp::Set { id, block } => {
                self.set(&id, block)?;
                Ok(self
                    .block(&id)
                    .expect("block exists immediately after a successful set")
                    .clone())
            }
            ContentOp::Insert { anchor, block } => self.insert(&anchor, block, gen_id()),
            ContentOp::Delete { id } => self.delete(&id),
            ContentOp::Move { id, anchor } => {
                self.move_block(&id, &anchor)?;
                Ok(self
                    .block(&id)
                    .expect("block exists immediately after a successful move")
                    .clone())
            }
        }
    }

    /// Apply a batch of operations atomically: every op must succeed or the
    /// document is left untouched. Ops apply in order against a working copy —
    /// so a later op may reference a block an earlier op in the same batch
    /// inserted — and the result is committed only once the whole batch lands.
    pub fn apply_all(
        &mut self,
        ops: Vec<ContentOp>,
        mut gen_id: impl FnMut() -> String,
    ) -> Result<(), ContentError> {
        let mut working = self.clone();
        for op in ops {
            working.apply(op, &mut gen_id)?;
        }
        *self = working;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn prose(md: &str) -> BlockInput {
        BlockInput {
            r#type: "prose".into(),
            data: json!({ "md": md }),
        }
    }

    fn ids(doc: &ContentDoc) -> Vec<&str> {
        doc.blocks.iter().map(|b| b.id.as_str()).collect()
    }

    fn seed() -> ContentDoc {
        let mut doc = ContentDoc::new();
        doc.insert(&Anchor::End, prose("one"), "a".into()).unwrap();
        doc.insert(&Anchor::End, prose("two"), "b".into()).unwrap();
        doc.insert(&Anchor::End, prose("three"), "c".into()).unwrap();
        doc
    }

    #[test]
    fn new_doc_is_empty() {
        assert!(ContentDoc::new().blocks.is_empty());
    }

    #[test]
    fn insert_into_empty_doc_adds_one_block() {
        let mut doc = ContentDoc::new();
        let created = doc.insert(&Anchor::End, prose("hello"), "x1".into()).unwrap();
        assert_eq!(created.id, "x1");
        assert_eq!(created.r#type, "prose");
        assert_eq!(created.data, json!({ "md": "hello" }));
        assert_eq!(ids(&doc), vec!["x1"]);
    }

    #[test]
    fn insert_after_places_immediately_after_target() {
        let mut doc = seed();
        doc.insert(&Anchor::After { id: "a".into() }, prose("mid"), "z".into())
            .unwrap();
        assert_eq!(ids(&doc), vec!["a", "z", "b", "c"]);
    }

    #[test]
    fn insert_before_places_immediately_before_target() {
        let mut doc = seed();
        doc.insert(&Anchor::Before { id: "c".into() }, prose("mid"), "z".into())
            .unwrap();
        assert_eq!(ids(&doc), vec!["a", "b", "z", "c"]);
    }

    #[test]
    fn insert_start_prepends() {
        let mut doc = seed();
        doc.insert(&Anchor::Start, prose("first"), "z".into()).unwrap();
        assert_eq!(ids(&doc), vec!["z", "a", "b", "c"]);
    }

    #[test]
    fn insert_with_missing_anchor_is_not_found() {
        let mut doc = seed();
        let err = doc
            .insert(&Anchor::After { id: "nope".into() }, prose("x"), "z".into())
            .unwrap_err();
        assert_eq!(err, ContentError::NotFound("nope".into()));
    }

    #[test]
    fn insert_with_empty_type_is_rejected() {
        let mut doc = ContentDoc::new();
        let input = BlockInput {
            r#type: "  ".into(),
            data: json!({}),
        };
        assert_eq!(
            doc.insert(&Anchor::End, input, "z".into()).unwrap_err(),
            ContentError::EmptyType
        );
    }

    #[test]
    fn insert_with_duplicate_id_is_rejected() {
        let mut doc = seed();
        let err = doc
            .insert(&Anchor::End, prose("x"), "b".into())
            .unwrap_err();
        assert_eq!(err, ContentError::DuplicateId("b".into()));
    }

    #[test]
    fn set_with_type_replaces_type_and_data_keeping_position() {
        let mut doc = seed();
        let patch = BlockPatch {
            r#type: Some("code".into()),
            data: json!({ "lang": "rust", "code": "fn main(){}" }),
        };
        doc.set("b", patch).unwrap();
        assert_eq!(ids(&doc), vec!["a", "b", "c"]);
        let b = doc.block("b").unwrap();
        assert_eq!(b.r#type, "code");
        assert_eq!(b.data, json!({ "lang": "rust", "code": "fn main(){}" }));
    }

    #[test]
    fn set_without_type_keeps_existing_type_and_replaces_data() {
        let mut doc = seed(); // "b" is a prose block
        let patch = BlockPatch {
            r#type: None,
            data: json!({ "md": "edited" }),
        };
        doc.set("b", patch).unwrap();
        let b = doc.block("b").unwrap();
        assert_eq!(b.r#type, "prose", "type should be retained when omitted");
        assert_eq!(b.data, json!({ "md": "edited" }));
    }

    #[test]
    fn set_missing_block_is_not_found() {
        let mut doc = seed();
        let patch = BlockPatch {
            r#type: None,
            data: json!({ "md": "x" }),
        };
        assert_eq!(
            doc.set("nope", patch).unwrap_err(),
            ContentError::NotFound("nope".into())
        );
    }

    #[test]
    fn set_with_empty_type_is_rejected() {
        let mut doc = seed();
        let patch = BlockPatch {
            r#type: Some("  ".into()),
            data: json!({}),
        };
        assert_eq!(doc.set("b", patch).unwrap_err(), ContentError::EmptyType);
    }

    #[test]
    fn delete_removes_block_and_returns_it() {
        let mut doc = seed();
        let removed = doc.delete("b").unwrap();
        assert_eq!(removed.id, "b");
        assert_eq!(ids(&doc), vec!["a", "c"]);
    }

    #[test]
    fn delete_missing_block_is_not_found() {
        let mut doc = seed();
        assert_eq!(
            doc.delete("nope").unwrap_err(),
            ContentError::NotFound("nope".into())
        );
    }

    #[test]
    fn move_after_reorders() {
        let mut doc = seed();
        doc.move_block("a", &Anchor::After { id: "b".into() }).unwrap();
        assert_eq!(ids(&doc), vec!["b", "a", "c"]);
    }

    #[test]
    fn move_before_reorders() {
        let mut doc = seed();
        doc.move_block("c", &Anchor::Before { id: "a".into() }).unwrap();
        assert_eq!(ids(&doc), vec!["c", "a", "b"]);
    }

    #[test]
    fn move_to_end_moves_block_last() {
        let mut doc = seed();
        doc.move_block("a", &Anchor::End).unwrap();
        assert_eq!(ids(&doc), vec!["b", "c", "a"]);
    }

    #[test]
    fn move_to_start_moves_block_first() {
        let mut doc = seed();
        doc.move_block("c", &Anchor::Start).unwrap();
        assert_eq!(ids(&doc), vec!["c", "a", "b"]);
    }

    #[test]
    fn move_relative_to_self_is_rejected() {
        let mut doc = seed();
        assert_eq!(
            doc.move_block("b", &Anchor::After { id: "b".into() })
                .unwrap_err(),
            ContentError::SelfAnchor
        );
    }

    #[test]
    fn move_missing_block_is_not_found() {
        let mut doc = seed();
        assert_eq!(
            doc.move_block("nope", &Anchor::End).unwrap_err(),
            ContentError::NotFound("nope".into())
        );
    }

    #[test]
    fn move_with_missing_anchor_is_not_found() {
        let mut doc = seed();
        assert_eq!(
            doc.move_block("a", &Anchor::After { id: "nope".into() })
                .unwrap_err(),
            ContentError::NotFound("nope".into())
        );
    }

    #[test]
    fn apply_insert_uses_generated_id_and_returns_block() {
        let mut doc = ContentDoc::new();
        let op = ContentOp::Insert {
            anchor: Anchor::End,
            block: prose("hi"),
        };
        let created = doc.apply(op, || "gen1".into()).unwrap();
        assert_eq!(created.id, "gen1");
        assert_eq!(ids(&doc), vec!["gen1"]);
    }

    #[test]
    fn apply_set_returns_updated_block() {
        let mut doc = seed();
        let op = ContentOp::Set {
            id: "b".into(),
            block: BlockPatch {
                r#type: None,
                data: json!({ "md": "changed" }),
            },
        };
        let updated = doc.apply(op, || unreachable!("set must not generate an id")).unwrap();
        assert_eq!(updated.id, "b");
        assert_eq!(updated.r#type, "prose");
        assert_eq!(updated.data, json!({ "md": "changed" }));
    }

    #[test]
    fn apply_delete_returns_removed_block() {
        let mut doc = seed();
        let op = ContentOp::Delete { id: "a".into() };
        let removed = doc.apply(op, || unreachable!()).unwrap();
        assert_eq!(removed.id, "a");
        assert_eq!(ids(&doc), vec!["b", "c"]);
    }

    #[test]
    fn apply_move_returns_moved_block() {
        let mut doc = seed();
        let op = ContentOp::Move {
            id: "a".into(),
            anchor: Anchor::End,
        };
        let moved = doc.apply(op, || unreachable!()).unwrap();
        assert_eq!(moved.id, "a");
        assert_eq!(ids(&doc), vec!["b", "c", "a"]);
    }

    #[test]
    fn apply_all_applies_ops_in_order() {
        let mut doc = seed(); // a, b, c
        let mut n = 0;
        let next_id = || {
            n += 1;
            format!("g{n}")
        };
        let ops = vec![
            ContentOp::Insert {
                anchor: Anchor::After { id: "a".into() },
                block: prose("x"),
            },
            ContentOp::Delete { id: "c".into() },
            ContentOp::Set {
                id: "b".into(),
                block: BlockPatch {
                    r#type: None,
                    data: json!({ "md": "B!" }),
                },
            },
        ];
        doc.apply_all(ops, next_id).unwrap();
        assert_eq!(ids(&doc), vec!["a", "g1", "b"]);
        assert_eq!(doc.block("b").unwrap().data, json!({ "md": "B!" }));
    }

    #[test]
    fn apply_all_is_atomic_when_an_op_fails() {
        let mut doc = seed(); // a, b, c
        let ops = vec![
            ContentOp::Delete { id: "a".into() },
            ContentOp::Delete { id: "missing".into() }, // fails here
        ];
        let err = doc
            .apply_all(ops, || unreachable!("no inserts in this batch"))
            .unwrap_err();
        assert_eq!(err, ContentError::NotFound("missing".into()));
        // The earlier delete must NOT have taken effect.
        assert_eq!(ids(&doc), vec!["a", "b", "c"]);
    }

    #[test]
    fn apply_all_empty_batch_is_a_noop() {
        let mut doc = seed();
        doc.apply_all(vec![], || unreachable!("empty batch generates no ids"))
            .unwrap();
        assert_eq!(ids(&doc), vec!["a", "b", "c"]);
    }

    #[test]
    fn apply_all_can_reference_a_block_inserted_earlier_in_the_same_batch() {
        let mut doc = ContentDoc::new();
        let mut n = 0;
        let next_id = || {
            n += 1;
            format!("g{n}")
        };
        let ops = vec![
            ContentOp::Insert {
                anchor: Anchor::End,
                block: prose("first"),
            },
            ContentOp::Set {
                id: "g1".into(),
                block: BlockPatch {
                    r#type: Some("code".into()),
                    data: json!({ "code": "x" }),
                },
            },
        ];
        doc.apply_all(ops, next_id).unwrap();
        assert_eq!(ids(&doc), vec!["g1"]);
        assert_eq!(doc.block("g1").unwrap().r#type, "code");
    }

    #[test]
    fn apply_all_assigns_distinct_ids_to_multiple_inserts() {
        let mut doc = ContentDoc::new();
        let mut n = 0;
        let next_id = || {
            n += 1;
            format!("g{n}")
        };
        let ops = vec![
            ContentOp::Insert {
                anchor: Anchor::End,
                block: prose("one"),
            },
            ContentOp::Insert {
                anchor: Anchor::End,
                block: prose("two"),
            },
        ];
        doc.apply_all(ops, next_id).unwrap();
        assert_eq!(ids(&doc), vec!["g1", "g2"]);
    }

    #[test]
    fn op_insert_serializes_with_tagged_wire_format() {
        let op = ContentOp::Insert {
            anchor: Anchor::After { id: "b".into() },
            block: prose("hi"),
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(
            v,
            json!({
                "op": "insert",
                "anchor": { "at": "after", "id": "b" },
                "block": { "type": "prose", "data": { "md": "hi" } }
            })
        );
    }

    #[test]
    fn set_op_with_type_serializes_with_type_key() {
        let op = ContentOp::Set {
            id: "a".into(),
            block: BlockPatch {
                r#type: Some("code".into()),
                data: json!({ "code": "x" }),
            },
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(
            v,
            json!({
                "op": "set",
                "id": "a",
                "block": { "type": "code", "data": { "code": "x" } }
            })
        );
    }

    #[test]
    fn set_op_without_type_omits_the_type_key() {
        let op = ContentOp::Set {
            id: "a".into(),
            block: BlockPatch {
                r#type: None,
                data: json!({ "md": "hi" }),
            },
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(
            v,
            json!({
                "op": "set",
                "id": "a",
                "block": { "data": { "md": "hi" } }
            })
        );
    }

    #[test]
    fn set_op_deserializes_data_only_patch_as_no_type() {
        let v = json!({
            "op": "set",
            "id": "a",
            "block": { "data": { "md": "hi" } }
        });
        let op: ContentOp = serde_json::from_value(v).unwrap();
        assert_eq!(
            op,
            ContentOp::Set {
                id: "a".into(),
                block: BlockPatch {
                    r#type: None,
                    data: json!({ "md": "hi" }),
                },
            }
        );
    }

    #[test]
    fn op_round_trips_through_json() {
        let op = ContentOp::Move {
            id: "x".into(),
            anchor: Anchor::Start,
        };
        let v = serde_json::to_value(&op).unwrap();
        let back: ContentOp = serde_json::from_value(v).unwrap();
        assert_eq!(op, back);
    }

    #[test]
    fn doc_round_trips_through_json() {
        let doc = seed();
        let v = serde_json::to_value(&doc).unwrap();
        let back: ContentDoc = serde_json::from_value(v).unwrap();
        assert_eq!(doc, back);
    }

    #[test]
    fn generated_id_is_eight_lowercase_alphanumerics() {
        let id = generate_block_id();
        assert_eq!(id.len(), 8, "id was {id:?}");
        assert!(
            id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "id {id:?} has out-of-alphabet chars"
        );
    }

    #[test]
    fn generated_ids_do_not_collide() {
        let set: std::collections::HashSet<String> =
            (0..1000).map(|_| generate_block_id()).collect();
        assert_eq!(set.len(), 1000);
    }

    #[test]
    fn from_stored_none_is_empty() {
        assert_eq!(ContentDoc::from_stored(None), ContentDoc::new());
    }

    #[test]
    fn from_stored_valid_doc_parses() {
        let doc = seed();
        let v = serde_json::to_value(&doc).unwrap();
        assert_eq!(ContentDoc::from_stored(Some(&v)), doc);
    }

    #[test]
    fn from_stored_legacy_tiptap_doc_is_empty() {
        let legacy = json!({
            "type": "doc",
            "content": [{ "type": "paragraph", "content": [] }]
        });
        assert_eq!(ContentDoc::from_stored(Some(&legacy)), ContentDoc::new());
    }

    #[test]
    fn from_stored_garbage_is_empty() {
        assert_eq!(
            ContentDoc::from_stored(Some(&json!("nonsense"))),
            ContentDoc::new()
        );
    }

    #[test]
    fn to_stored_empty_doc_is_none() {
        // An empty document persists as SQL NULL, not `{"blocks":[]}`, so a
        // project with no body still reads as having no detail content.
        assert_eq!(ContentDoc::new().to_stored(), None);
    }

    #[test]
    fn to_stored_nonempty_doc_is_some_blocks() {
        let doc = seed();
        let stored = doc.to_stored().expect("non-empty doc should persist a value");
        assert_eq!(stored, json!({ "blocks": serde_json::to_value(&doc.blocks).unwrap() }));
    }

    #[test]
    fn to_stored_then_from_stored_round_trips() {
        let doc = seed();
        let stored = doc.to_stored();
        assert_eq!(ContentDoc::from_stored(stored.as_ref()), doc);
    }
}
