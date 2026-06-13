//! `ProseMirror`/`TipTap` document model and schema validation.
//!
//! The detail-page body is a single `TipTap` document. [`Node`] is a faithful
//! 1:1 mirror of its JSON — any node type round-trips untouched — and
//! [`Node::validate_document`] bounds what may be stored against an allow-list
//! schema. The [`Doc`] newtype hosts the top-level block ops (insert, replace,
//! delete, move by id). The editor already guarantees well-formed structure;
//! validation here is the write-path safety net (defense-in-depth, the same
//! posture as the SSR sanitizer).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// An inline formatting mark (bold, link, …) carried by a text node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mark {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub attrs: Map<String, Value>,
}

/// One `ProseMirror` node. Text nodes carry `text` (+ optional `marks`); every
/// other node carries `content`. Empty fields are omitted on the wire to match
/// `ProseMirror`'s own JSON output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub attrs: Map<String, Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<Self>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<Mark>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Why a document failed validation. Each maps to a 4xx on the write path.
#[derive(Debug, Clone, PartialEq)]
pub enum PmError {
    /// The value isn't a well-formed `ProseMirror` node at all (it failed to
    /// deserialize). Only the strict write path ([`Doc::parse`]) raises this;
    /// reads ([`Doc::from_stored`]) degrade to an empty document instead.
    Malformed(String),
    /// The root node is not a `doc`.
    NotADoc(String),
    /// A node type outside the schema.
    UnknownNode(String),
    /// A mark type outside the schema.
    UnknownMark(String),
    /// A node appears somewhere its parent's content rule forbids.
    DisallowedChild { parent: String, child: String },
    /// Marks on a non-text node.
    MarksOnNonText(String),
    /// A marked text node inside a code block (code carries no formatting).
    MarksInCodeBlock,
    /// A text node with an empty string (`ProseMirror` never stores these).
    EmptyText,
    /// A text node that also has children.
    TextWithContent,
    /// A non-text node carrying a `text` field.
    NonTextWithText(String),
    /// A `link` mark whose href uses a disallowed scheme.
    BadLinkScheme(String),
}

impl std::fmt::Display for PmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed(detail) => {
                write!(f, "detail content is not a valid document: {detail}")
            }
            Self::NotADoc(found) => {
                write!(f, "document root must be a \"doc\" node, found \"{found}\"")
            }
            Self::UnknownNode(name) => write!(f, "unknown node type \"{name}\""),
            Self::UnknownMark(name) => write!(f, "unknown mark type \"{name}\""),
            Self::DisallowedChild { parent, child } => {
                write!(f, "node \"{child}\" is not allowed inside \"{parent}\"")
            }
            Self::MarksOnNonText(node) => {
                write!(
                    f,
                    "marks are only allowed on text nodes, found on \"{node}\""
                )
            }
            Self::MarksInCodeBlock => write!(f, "code blocks cannot contain formatting marks"),
            Self::EmptyText => write!(f, "text nodes cannot be empty"),
            Self::TextWithContent => write!(f, "a text node cannot have child content"),
            Self::NonTextWithText(node) => {
                write!(f, "non-text node \"{node}\" cannot carry a text field")
            }
            Self::BadLinkScheme(href) => {
                write!(f, "link uses a disallowed URL scheme: \"{href}\"")
            }
        }
    }
}

impl std::error::Error for PmError {}

/// Coarse content category. We only need the doc/block/inline split plus the
/// per-node child rules below — not `ProseMirror`'s full content expressions.
#[derive(Clone, Copy, PartialEq)]
enum Group {
    Doc,
    Block,
    Inline,
}

/// What a node may contain.
#[derive(Clone, Copy)]
enum Content {
    /// A leaf — no children (`horizontalRule`, `hardBreak`).
    Empty,
    /// Any block-group node.
    Block,
    /// Any inline-group node (`text`, `hardBreak`).
    Inline,
    /// Text nodes only, carrying no marks (`codeBlock`).
    Code,
    /// Exactly these node types (lists → `listItem`).
    Only(&'static [&'static str]),
}

struct NodeSpec {
    name: &'static str,
    group: Group,
    content: Content,
}

/// The detail-document schema — mirrors the `TipTap` editor's `StarterKit` (headings
/// 2–4, lists, blockquote, code blocks, rule, hard break) plus the `text` leaf.
/// Adding a widget is one row here (and the matching editor extension).
const NODES: &[NodeSpec] = &[
    NodeSpec {
        name: "doc",
        group: Group::Doc,
        content: Content::Block,
    },
    NodeSpec {
        name: "paragraph",
        group: Group::Block,
        content: Content::Inline,
    },
    NodeSpec {
        name: "heading",
        group: Group::Block,
        content: Content::Inline,
    },
    NodeSpec {
        name: "blockquote",
        group: Group::Block,
        content: Content::Block,
    },
    NodeSpec {
        name: "bulletList",
        group: Group::Block,
        content: Content::Only(&["listItem"]),
    },
    NodeSpec {
        name: "orderedList",
        group: Group::Block,
        content: Content::Only(&["listItem"]),
    },
    NodeSpec {
        name: "listItem",
        group: Group::Block,
        content: Content::Block,
    },
    NodeSpec {
        name: "codeBlock",
        group: Group::Block,
        content: Content::Code,
    },
    NodeSpec {
        name: "horizontalRule",
        group: Group::Block,
        content: Content::Empty,
    },
    NodeSpec {
        name: "hardBreak",
        group: Group::Inline,
        content: Content::Empty,
    },
    NodeSpec {
        name: "text",
        group: Group::Inline,
        content: Content::Empty,
    },
];

/// Marks the editor may emit. `code` here is the inline-code mark, distinct from
/// the `codeBlock` node.
const MARKS: &[&str] = &["bold", "italic", "strike", "code", "link", "underline"];

/// Schemes a `link` href may use — mirrors the editor's own input guard and the
/// SSR sanitizer's `allowedSchemes`.
const LINK_SCHEMES: &[&str] = &["http://", "https://", "mailto:"];

impl NodeSpec {
    /// The schema entry for a node type, if it's in the allow-list.
    fn lookup(name: &str) -> Option<&'static Self> {
        NODES.iter().find(|spec| spec.name == name)
    }
}

impl Content {
    /// Whether a parent with this content rule may contain `child`. `child` is
    /// assumed already validated, so its spec exists.
    fn permits(self, child: &Node) -> bool {
        let group = NodeSpec::lookup(&child.r#type).map(|spec| spec.group);
        match self {
            Self::Empty => false,
            Self::Block => group == Some(Group::Block),
            Self::Inline => group == Some(Group::Inline),
            Self::Code => child.r#type == "text",
            Self::Only(types) => types.contains(&child.r#type.as_str()),
        }
    }
}

impl Mark {
    /// Validate this mark: its type must be in the allow-list, and a `link`'s
    /// href must use an allowed scheme.
    fn validate(&self) -> Result<(), PmError> {
        if !MARKS.contains(&self.r#type.as_str()) {
            return Err(PmError::UnknownMark(self.r#type.clone()));
        }
        if self.r#type == "link" {
            self.validate_link()?;
        }
        Ok(())
    }

    fn validate_link(&self) -> Result<(), PmError> {
        let href = self
            .attrs
            .get("href")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let lower = href.to_ascii_lowercase();
        if LINK_SCHEMES.iter().any(|scheme| lower.starts_with(scheme)) {
            Ok(())
        } else {
            Err(PmError::BadLinkScheme(href.to_string()))
        }
    }
}

impl Node {
    /// Validate this node as a document root: it must be a `doc`, and its whole
    /// subtree must satisfy the schema.
    pub fn validate_document(&self) -> Result<(), PmError> {
        if self.r#type != "doc" {
            return Err(PmError::NotADoc(self.r#type.clone()));
        }
        self.validate_node()
    }

    /// Validate this node and, recursively, its subtree.
    fn validate_node(&self) -> Result<(), PmError> {
        let spec = NodeSpec::lookup(&self.r#type)
            .ok_or_else(|| PmError::UnknownNode(self.r#type.clone()))?;

        if self.r#type == "text" {
            match self.text.as_deref() {
                None | Some("") => return Err(PmError::EmptyText),
                Some(_) => {}
            }
            if !self.content.is_empty() {
                return Err(PmError::TextWithContent);
            }
        } else if self.text.is_some() {
            return Err(PmError::NonTextWithText(self.r#type.clone()));
        }

        if !self.marks.is_empty() && self.r#type != "text" {
            return Err(PmError::MarksOnNonText(self.r#type.clone()));
        }
        for mark in &self.marks {
            mark.validate()?;
        }

        for child in &self.content {
            child.validate_node()?;
            if matches!(spec.content, Content::Code) && !child.marks.is_empty() {
                return Err(PmError::MarksInCodeBlock);
            }
            if !spec.content.permits(child) {
                return Err(PmError::DisallowedChild {
                    parent: self.r#type.clone(),
                    child: child.r#type.clone(),
                });
            }
        }
        Ok(())
    }
}

/// The `attrs` key holding a top-level block's stable id. Matches
/// `@tiptap/extension-unique-id`'s default `attributeName`, so ids minted in the
/// editor and on the server live under the same key.
pub const ID_ATTR: &str = "id";

/// A short, URL- and CLI-friendly block id: 8 lowercase alphanumerics. The
/// editor's unique-id extension is configured with a matching `generateID`
/// (Slice 5), so ids minted on either side share one style — and stay short
/// enough to pass on a command line without nesting awkwardly.
pub fn generate_block_id() -> String {
    const ALPHABET: [char; 36] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    ];
    nanoid::nanoid!(8, &ALPHABET)
}

impl Node {
    /// This block's stable id, if it carries one.
    pub fn block_id(&self) -> Option<&str> {
        self.attrs.get(ID_ATTR).and_then(Value::as_str)
    }

    /// Stamp `id` into this node's `attrs`, overwriting any id it arrived with.
    fn set_block_id(&mut self, id: &str) {
        self.attrs
            .insert(ID_ATTR.to_string(), Value::String(id.to_string()));
    }

    /// All descendant text, concatenated in document order — a node's plain-text
    /// projection, used for previews.
    pub fn text_content(&self) -> String {
        let mut out = self.text.clone().unwrap_or_default();
        for child in &self.content {
            out.push_str(&child.text_content());
        }
        out
    }
}

/// Where an insert or move lands, relative to the existing top-level blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "at", rename_all = "snake_case")]
pub enum Anchor {
    Start,
    End,
    After { id: String },
    Before { id: String },
}

impl Anchor {
    /// Parse the CLI anchor syntax: `start`, `end`, `after:<id>`, `before:<id>`.
    pub fn parse(s: &str) -> Result<Self, String> {
        if s == "start" {
            return Ok(Self::Start);
        }
        if s == "end" {
            return Ok(Self::End);
        }
        if let Some(id) = s.strip_prefix("after:").filter(|id| !id.is_empty()) {
            return Ok(Self::After { id: id.to_string() });
        }
        if let Some(id) = s.strip_prefix("before:").filter(|id| !id.is_empty()) {
            return Ok(Self::Before { id: id.to_string() });
        }
        Err(format!(
            "invalid position \"{s}\"; use start, end, after:<id>, or before:<id>"
        ))
    }
}

/// A single mutating operation over the document's top-level blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum DocOp {
    /// Insert `node` as a new block at `anchor`; the server assigns its id.
    Insert { anchor: Anchor, node: Node },
    /// Replace the block `id` with `node`, keeping the block's id and position.
    Replace { id: String, node: Node },
    /// Remove the block `id`.
    Delete { id: String },
    /// Move the block `id` to `anchor`.
    Move { id: String, anchor: Anchor },
}

/// Structural failures applying an op, plus a schema failure on the result.
#[derive(Debug, Clone, PartialEq)]
pub enum OpError {
    /// No block with this id exists (target of replace/delete/move, or an anchor).
    NotFound(String),
    /// A freshly generated block id collided with an existing one.
    DuplicateId(String),
    /// A move that references itself as its own anchor.
    SelfAnchor,
    /// The document the batch produced failed schema validation.
    Invalid(PmError),
}

/// The detail-page body: a `doc` node whose direct children are the blocks.
/// Wraps a [`Node`] so the ops have an unambiguous "this is the document" home;
/// the inner node is always a faithful `ProseMirror` `doc`.
#[derive(Debug, Clone, PartialEq)]
pub struct Doc(Node);

impl Default for Doc {
    fn default() -> Self {
        Self(Node {
            r#type: "doc".to_string(),
            attrs: Map::new(),
            content: Vec::new(),
            marks: Vec::new(),
            text: None,
        })
    }
}

impl Doc {
    /// Parse a stored `detail_content` value. Anything that isn't a `doc` node
    /// (missing, garbage, a non-doc root) degrades to an empty document rather
    /// than failing — a read never rejects what a write already accepted.
    pub fn from_stored(value: Option<&Value>) -> Self {
        value
            .and_then(|v| serde_json::from_value::<Node>(v.clone()).ok())
            .filter(|node| node.r#type == "doc")
            .map_or_else(Self::default, Self)
    }

    /// Strictly parse client-supplied detail content. Unlike [`Self::from_stored`]
    /// — which tolerantly degrades anything invalid to an empty document for
    /// trusted reads — this rejects malformed or schema-violating input, so the
    /// write path surfaces a 4xx rather than silently dropping the body.
    pub fn parse(value: &Value) -> Result<Self, PmError> {
        let node: Node =
            serde_json::from_value(value.clone()).map_err(|e| PmError::Malformed(e.to_string()))?;
        node.validate_document()?;
        Ok(Self(node))
    }

    /// Serialize for storage; an empty document is `None` (SQL `NULL`), so a
    /// project with no body reads as having no content.
    pub fn to_stored(&self) -> Option<Value> {
        if self.0.content.is_empty() {
            None
        } else {
            Some(serde_json::to_value(&self.0).expect("a doc always serializes"))
        }
    }

    /// The underlying `doc` node.
    pub const fn node(&self) -> &Node {
        &self.0
    }

    /// Consume the document, yielding its `doc` node.
    pub fn into_inner(self) -> Node {
        self.0
    }

    /// The document's top-level blocks.
    pub fn blocks(&self) -> &[Node] {
        &self.0.content
    }

    /// Validate the document against the schema.
    pub fn validate(&self) -> Result<(), PmError> {
        self.0.validate_document()
    }

    /// Borrow a top-level block by id.
    pub fn block(&self, id: &str) -> Option<&Node> {
        self.0.content.iter().find(|n| n.block_id() == Some(id))
    }

    fn index_of(&self, id: &str) -> Option<usize> {
        self.0.content.iter().position(|n| n.block_id() == Some(id))
    }

    /// Resolve an anchor to an insertion index in the current block list.
    fn resolve_anchor(&self, anchor: &Anchor) -> Result<usize, OpError> {
        match anchor {
            Anchor::Start => Ok(0),
            Anchor::End => Ok(self.0.content.len()),
            Anchor::After { id } => self
                .index_of(id)
                .map(|i| i + 1)
                .ok_or_else(|| OpError::NotFound(id.clone())),
            Anchor::Before { id } => self
                .index_of(id)
                .ok_or_else(|| OpError::NotFound(id.clone())),
        }
    }

    /// Insert `node` as a new block at `anchor`, stamping it with `new_id`
    /// (overwriting any id the node arrived with). Returns the inserted block.
    pub fn insert(
        &mut self,
        anchor: &Anchor,
        mut node: Node,
        new_id: String,
    ) -> Result<Node, OpError> {
        if self.index_of(&new_id).is_some() {
            return Err(OpError::DuplicateId(new_id));
        }
        let at = self.resolve_anchor(anchor)?;
        node.set_block_id(&new_id);
        self.0.content.insert(at, node.clone());
        Ok(node)
    }

    /// Replace the block `id` with `node`, preserving the block's id and slot.
    pub fn replace(&mut self, id: &str, mut node: Node) -> Result<Node, OpError> {
        let i = self
            .index_of(id)
            .ok_or_else(|| OpError::NotFound(id.to_string()))?;
        node.set_block_id(id);
        self.0.content[i] = node.clone();
        Ok(node)
    }

    /// Remove a block, returning it.
    pub fn delete(&mut self, id: &str) -> Result<Node, OpError> {
        let i = self
            .index_of(id)
            .ok_or_else(|| OpError::NotFound(id.to_string()))?;
        Ok(self.0.content.remove(i))
    }

    /// Move an existing block to `anchor`.
    pub fn move_block(&mut self, id: &str, anchor: &Anchor) -> Result<(), OpError> {
        if let Anchor::After { id: target } | Anchor::Before { id: target } = anchor
            && target == id
        {
            return Err(OpError::SelfAnchor);
        }
        let from = self
            .index_of(id)
            .ok_or_else(|| OpError::NotFound(id.to_string()))?;
        // Resolve against the current list, then account for the gap the removal
        // leaves behind anything that sat after the moving block.
        let to = self.resolve_anchor(anchor)?;
        let node = self.0.content.remove(from);
        let adjusted = if to > from { to - 1 } else { to };
        self.0.content.insert(adjusted, node);
        Ok(())
    }

    /// Apply one operation, sourcing a fresh id from `gen_id` for inserts.
    /// Returns the affected block. A building block for [`Self::apply_all`]; it
    /// does not validate the resulting document.
    pub fn apply(&mut self, op: DocOp, gen_id: impl FnOnce() -> String) -> Result<Node, OpError> {
        match op {
            DocOp::Insert { anchor, node } => self.insert(&anchor, node, gen_id()),
            DocOp::Replace { id, node } => self.replace(&id, node),
            DocOp::Delete { id } => self.delete(&id),
            DocOp::Move { id, anchor } => {
                self.move_block(&id, &anchor)?;
                Ok(self
                    .block(&id)
                    .expect("block exists immediately after a successful move")
                    .clone())
            }
        }
    }

    /// Apply a batch atomically: every op must succeed and the resulting
    /// document must validate, or the document is left untouched. Ops apply in
    /// order against a working copy (so a later op may reference a block an
    /// earlier op inserted); the result is committed only once the whole batch
    /// lands and validates.
    pub fn apply_all(
        &mut self,
        ops: Vec<DocOp>,
        mut gen_id: impl FnMut() -> String,
    ) -> Result<(), OpError> {
        let mut working = self.clone();
        for op in ops {
            working.apply(op, &mut gen_id)?;
        }
        working.validate().map_err(OpError::Invalid)?;
        *self = working;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A representative document exercising every node and mark kind.
    fn sample() -> Value {
        json!({
            "type": "doc",
            "content": [
                { "type": "heading", "attrs": { "level": 2 },
                  "content": [{ "type": "text", "text": "Title" }] },
                { "type": "paragraph", "content": [
                    { "type": "text", "text": "plain " },
                    { "type": "text", "text": "bold", "marks": [{ "type": "bold" }] },
                    { "type": "text", "text": " and ", },
                    { "type": "text", "text": "link",
                      "marks": [{ "type": "link", "attrs": { "href": "https://example.com" } }] },
                    { "type": "hardBreak" }
                ]},
                { "type": "bulletList", "content": [
                    { "type": "listItem", "content": [
                        { "type": "paragraph", "content": [{ "type": "text", "text": "item" }] }
                    ]}
                ]},
                { "type": "blockquote", "content": [
                    { "type": "paragraph", "content": [{ "type": "text", "text": "quote" }] }
                ]},
                { "type": "codeBlock", "attrs": { "language": "rust" },
                  "content": [{ "type": "text", "text": "fn main() {}" }] },
                { "type": "horizontalRule" }
            ]
        })
    }

    fn node(v: Value) -> Node {
        serde_json::from_value(v).expect("valid node json")
    }

    #[test]
    fn model_round_trips_through_json() {
        let original = sample();
        let parsed: Node = node(original.clone());
        let back = serde_json::to_value(&parsed).unwrap();
        assert_eq!(back, original, "round-trip changed the document");
    }

    #[test]
    fn validate_accepts_a_representative_document() {
        assert_eq!(node(sample()).validate_document(), Ok(()));
    }

    #[test]
    fn validate_rejects_non_doc_root() {
        let v = node(json!({ "type": "paragraph", "content": [] }));
        assert_eq!(
            v.validate_document(),
            Err(PmError::NotADoc("paragraph".into()))
        );
    }

    #[test]
    fn validate_rejects_unknown_node() {
        let v = node(json!({ "type": "doc", "content": [{ "type": "script" }] }));
        assert_eq!(
            v.validate_document(),
            Err(PmError::UnknownNode("script".into()))
        );
    }

    #[test]
    fn validate_rejects_unknown_mark() {
        let v = node(json!({ "type": "doc", "content": [
            { "type": "paragraph", "content": [
                { "type": "text", "text": "x", "marks": [{ "type": "blink" }] }
            ]}
        ]}));
        assert_eq!(
            v.validate_document(),
            Err(PmError::UnknownMark("blink".into()))
        );
    }

    #[test]
    fn validate_rejects_marks_on_block() {
        let v = node(json!({ "type": "doc", "content": [
            { "type": "paragraph", "marks": [{ "type": "bold" }], "content": [] }
        ]}));
        assert_eq!(
            v.validate_document(),
            Err(PmError::MarksOnNonText("paragraph".into()))
        );
    }

    #[test]
    fn validate_rejects_marks_in_code_block() {
        let v = node(json!({ "type": "doc", "content": [
            { "type": "codeBlock", "content": [
                { "type": "text", "text": "x", "marks": [{ "type": "bold" }] }
            ]}
        ]}));
        assert_eq!(v.validate_document(), Err(PmError::MarksInCodeBlock));
    }

    #[test]
    fn validate_rejects_empty_text() {
        let v = node(json!({ "type": "doc", "content": [
            { "type": "paragraph", "content": [{ "type": "text", "text": "" }] }
        ]}));
        assert_eq!(v.validate_document(), Err(PmError::EmptyText));
    }

    #[test]
    fn validate_rejects_text_with_content() {
        let v = node(json!({ "type": "doc", "content": [
            { "type": "paragraph", "content": [
                { "type": "text", "text": "x", "content": [{ "type": "text", "text": "y" }] }
            ]}
        ]}));
        assert_eq!(v.validate_document(), Err(PmError::TextWithContent));
    }

    #[test]
    fn validate_rejects_non_text_with_text() {
        let v = node(json!({ "type": "doc", "content": [
            { "type": "paragraph", "text": "oops", "content": [] }
        ]}));
        assert_eq!(
            v.validate_document(),
            Err(PmError::NonTextWithText("paragraph".into()))
        );
    }

    #[test]
    fn validate_rejects_disallowed_child() {
        // A bulletList may only contain listItems.
        let v = node(json!({ "type": "doc", "content": [
            { "type": "bulletList", "content": [
                { "type": "paragraph", "content": [{ "type": "text", "text": "x" }] }
            ]}
        ]}));
        assert_eq!(
            v.validate_document(),
            Err(PmError::DisallowedChild {
                parent: "bulletList".into(),
                child: "paragraph".into()
            })
        );
    }

    #[test]
    fn validate_rejects_inline_node_at_block_level() {
        // A bare text node directly under doc is an inline node where a block is required.
        let v = node(json!({ "type": "doc", "content": [{ "type": "text", "text": "x" }] }));
        assert_eq!(
            v.validate_document(),
            Err(PmError::DisallowedChild {
                parent: "doc".into(),
                child: "text".into()
            })
        );
    }

    #[test]
    fn validate_rejects_children_on_a_leaf() {
        let v = node(json!({ "type": "doc", "content": [
            { "type": "horizontalRule", "content": [
                { "type": "paragraph", "content": [{ "type": "text", "text": "x" }] }
            ]}
        ]}));
        assert_eq!(
            v.validate_document(),
            Err(PmError::DisallowedChild {
                parent: "horizontalRule".into(),
                child: "paragraph".into()
            })
        );
    }

    #[test]
    fn validate_rejects_bad_link_scheme() {
        let v = node(json!({ "type": "doc", "content": [
            { "type": "paragraph", "content": [
                { "type": "text", "text": "x",
                  "marks": [{ "type": "link", "attrs": { "href": "javascript:alert(1)" } }] }
            ]}
        ]}));
        assert_eq!(
            v.validate_document(),
            Err(PmError::BadLinkScheme("javascript:alert(1)".into()))
        );
    }

    #[test]
    fn validate_accepts_empty_doc() {
        let v = node(json!({ "type": "doc" }));
        assert_eq!(v.validate_document(), Ok(()));
    }

    #[test]
    fn text_content_of_leaf_text_is_its_text() {
        assert_eq!(
            node(json!({ "type": "text", "text": "x" })).text_content(),
            "x"
        );
    }

    #[test]
    fn text_content_concatenates_descendant_text() {
        let n = node(json!({
            "type": "paragraph",
            "content": [
                { "type": "text", "text": "Hello " },
                { "type": "text", "text": "world", "marks": [{ "type": "bold" }] }
            ]
        }));
        assert_eq!(n.text_content(), "Hello world");
    }

    #[test]
    fn text_content_is_empty_for_a_textless_leaf() {
        assert_eq!(node(json!({ "type": "horizontalRule" })).text_content(), "");
    }

    #[test]
    fn pm_error_messages_are_human_readable() {
        assert_eq!(
            PmError::UnknownNode("script".into()).to_string(),
            "unknown node type \"script\""
        );
        assert_eq!(
            PmError::DisallowedChild {
                parent: "doc".into(),
                child: "text".into()
            }
            .to_string(),
            "node \"text\" is not allowed inside \"doc\""
        );
        assert_eq!(
            PmError::BadLinkScheme("javascript:x".into()).to_string(),
            "link uses a disallowed URL scheme: \"javascript:x\""
        );
    }
}

#[cfg(test)]
mod op_tests {
    use super::*;
    use serde_json::json;

    fn node_json(v: Value) -> Node {
        serde_json::from_value(v).expect("valid node json")
    }

    /// A paragraph block carrying one text node and no id yet.
    fn para(text: &str) -> Node {
        node_json(json!({ "type": "paragraph", "content": [{ "type": "text", "text": text }] }))
    }

    fn ids(doc: &Doc) -> Vec<&str> {
        doc.0.content.iter().filter_map(Node::block_id).collect()
    }

    fn seed() -> Doc {
        let mut doc = Doc::default();
        doc.insert(&Anchor::End, para("one"), "a".into()).unwrap();
        doc.insert(&Anchor::End, para("two"), "b".into()).unwrap();
        doc.insert(&Anchor::End, para("three"), "c".into()).unwrap();
        doc
    }

    #[test]
    fn new_doc_is_empty() {
        let doc = Doc::default();
        assert!(doc.0.content.is_empty());
        assert_eq!(doc.0.r#type, "doc");
    }

    #[test]
    fn parse_accepts_a_valid_document() {
        let value = json!({
            "type": "doc",
            "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": "hi" }] }]
        });
        let doc = Doc::parse(&value).expect("a valid document parses");
        assert_eq!(doc.blocks().len(), 1);
    }

    #[test]
    fn parse_rejects_a_non_doc_root() {
        let value = json!({ "type": "paragraph" });
        assert_eq!(
            Doc::parse(&value),
            Err(PmError::NotADoc("paragraph".into()))
        );
    }

    #[test]
    fn parse_rejects_a_schema_violation() {
        let value = json!({ "type": "doc", "content": [{ "type": "bogus" }] });
        assert_eq!(
            Doc::parse(&value),
            Err(PmError::UnknownNode("bogus".into()))
        );
    }

    #[test]
    fn parse_rejects_values_that_are_not_a_node() {
        // `from_stored` degrades each of these to an empty doc; `parse` must
        // reject them so a write can't silently store nothing.
        for value in [
            json!(42),
            json!("nope"),
            json!({}),
            json!({ "content": [] }),
        ] {
            assert!(
                matches!(Doc::parse(&value), Err(PmError::Malformed(_))),
                "expected Malformed for {value}"
            );
        }
    }

    #[test]
    fn parse_is_stricter_than_from_stored() {
        let garbage = json!({ "not": "a node" });
        assert!(Doc::from_stored(Some(&garbage)).blocks().is_empty());
        assert!(Doc::parse(&garbage).is_err());
    }

    #[test]
    fn insert_into_empty_doc_adds_one_block() {
        let mut doc = Doc::default();
        let created = doc
            .insert(&Anchor::End, para("hello"), "x1".into())
            .unwrap();
        assert_eq!(created.block_id(), Some("x1"));
        assert_eq!(created.r#type, "paragraph");
        assert_eq!(ids(&doc), vec!["x1"]);
    }

    #[test]
    fn insert_stamps_server_id_over_any_provided_id() {
        let mut doc = Doc::default();
        let mut incoming = para("hi");
        incoming.set_block_id("client-chosen");
        let created = doc.insert(&Anchor::End, incoming, "server".into()).unwrap();
        assert_eq!(created.block_id(), Some("server"));
        assert_eq!(ids(&doc), vec!["server"]);
    }

    #[test]
    fn insert_after_places_immediately_after_target() {
        let mut doc = seed();
        doc.insert(&Anchor::After { id: "a".into() }, para("mid"), "z".into())
            .unwrap();
        assert_eq!(ids(&doc), vec!["a", "z", "b", "c"]);
    }

    #[test]
    fn insert_before_places_immediately_before_target() {
        let mut doc = seed();
        doc.insert(&Anchor::Before { id: "c".into() }, para("mid"), "z".into())
            .unwrap();
        assert_eq!(ids(&doc), vec!["a", "b", "z", "c"]);
    }

    #[test]
    fn insert_start_prepends() {
        let mut doc = seed();
        doc.insert(&Anchor::Start, para("first"), "z".into())
            .unwrap();
        assert_eq!(ids(&doc), vec!["z", "a", "b", "c"]);
    }

    #[test]
    fn insert_with_missing_anchor_is_not_found() {
        let mut doc = seed();
        let err = doc
            .insert(&Anchor::After { id: "nope".into() }, para("x"), "z".into())
            .unwrap_err();
        assert_eq!(err, OpError::NotFound("nope".into()));
    }

    #[test]
    fn insert_with_duplicate_id_is_rejected() {
        let mut doc = seed();
        let err = doc.insert(&Anchor::End, para("x"), "b".into()).unwrap_err();
        assert_eq!(err, OpError::DuplicateId("b".into()));
    }

    #[test]
    fn replace_swaps_node_keeping_id_and_position() {
        let mut doc = seed();
        let heading = node_json(json!({
            "type": "heading", "attrs": { "level": 2 },
            "content": [{ "type": "text", "text": "H" }]
        }));
        let replaced = doc.replace("b", heading).unwrap();
        assert_eq!(replaced.block_id(), Some("b"));
        assert_eq!(ids(&doc), vec!["a", "b", "c"]);
        let b = doc.block("b").unwrap();
        assert_eq!(b.r#type, "heading");
    }

    #[test]
    fn replace_preserves_target_id_over_nodes_own_id() {
        let mut doc = seed();
        let mut incoming = para("x");
        incoming.set_block_id("wrong");
        doc.replace("b", incoming).unwrap();
        assert_eq!(ids(&doc), vec!["a", "b", "c"]);
        assert!(doc.block("wrong").is_none());
    }

    #[test]
    fn replace_missing_block_is_not_found() {
        let mut doc = seed();
        assert_eq!(
            doc.replace("nope", para("x")).unwrap_err(),
            OpError::NotFound("nope".into())
        );
    }

    #[test]
    fn delete_removes_block_and_returns_it() {
        let mut doc = seed();
        let removed = doc.delete("b").unwrap();
        assert_eq!(removed.block_id(), Some("b"));
        assert_eq!(ids(&doc), vec!["a", "c"]);
    }

    #[test]
    fn delete_missing_block_is_not_found() {
        let mut doc = seed();
        assert_eq!(
            doc.delete("nope").unwrap_err(),
            OpError::NotFound("nope".into())
        );
    }

    #[test]
    fn move_after_reorders() {
        let mut doc = seed();
        doc.move_block("a", &Anchor::After { id: "b".into() })
            .unwrap();
        assert_eq!(ids(&doc), vec!["b", "a", "c"]);
    }

    #[test]
    fn move_before_reorders() {
        let mut doc = seed();
        doc.move_block("c", &Anchor::Before { id: "a".into() })
            .unwrap();
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
            OpError::SelfAnchor
        );
    }

    #[test]
    fn move_missing_block_is_not_found() {
        let mut doc = seed();
        assert_eq!(
            doc.move_block("nope", &Anchor::End).unwrap_err(),
            OpError::NotFound("nope".into())
        );
    }

    #[test]
    fn move_with_missing_anchor_is_not_found() {
        let mut doc = seed();
        assert_eq!(
            doc.move_block("a", &Anchor::After { id: "nope".into() })
                .unwrap_err(),
            OpError::NotFound("nope".into())
        );
    }

    #[test]
    fn apply_insert_uses_generated_id_and_returns_block() {
        let mut doc = Doc::default();
        let op = DocOp::Insert {
            anchor: Anchor::End,
            node: para("hi"),
        };
        let created = doc.apply(op, || "gen1".into()).unwrap();
        assert_eq!(created.block_id(), Some("gen1"));
        assert_eq!(ids(&doc), vec!["gen1"]);
    }

    #[test]
    fn apply_replace_returns_block() {
        let mut doc = seed();
        let op = DocOp::Replace {
            id: "b".into(),
            node: para("changed"),
        };
        let updated = doc
            .apply(op, || unreachable!("replace must not generate an id"))
            .unwrap();
        assert_eq!(updated.block_id(), Some("b"));
        assert_eq!(updated.r#type, "paragraph");
    }

    #[test]
    fn apply_delete_returns_removed_block() {
        let mut doc = seed();
        let op = DocOp::Delete { id: "a".into() };
        let removed = doc.apply(op, || unreachable!()).unwrap();
        assert_eq!(removed.block_id(), Some("a"));
        assert_eq!(ids(&doc), vec!["b", "c"]);
    }

    #[test]
    fn apply_move_returns_moved_block() {
        let mut doc = seed();
        let op = DocOp::Move {
            id: "a".into(),
            anchor: Anchor::End,
        };
        let moved = doc.apply(op, || unreachable!()).unwrap();
        assert_eq!(moved.block_id(), Some("a"));
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
            DocOp::Insert {
                anchor: Anchor::After { id: "a".into() },
                node: para("x"),
            },
            DocOp::Delete { id: "c".into() },
            DocOp::Replace {
                id: "b".into(),
                node: para("B!"),
            },
        ];
        doc.apply_all(ops, next_id).unwrap();
        assert_eq!(ids(&doc), vec!["a", "g1", "b"]);
    }

    #[test]
    fn apply_all_is_atomic_when_an_op_fails() {
        let mut doc = seed(); // a, b, c
        let ops = vec![
            DocOp::Delete { id: "a".into() },
            DocOp::Delete {
                id: "missing".into(),
            }, // fails here
        ];
        let err = doc
            .apply_all(ops, || unreachable!("no inserts in this batch"))
            .unwrap_err();
        assert_eq!(err, OpError::NotFound("missing".into()));
        assert_eq!(ids(&doc), vec!["a", "b", "c"]); // earlier delete rolled back
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
        let mut doc = Doc::default();
        let mut n = 0;
        let next_id = || {
            n += 1;
            format!("g{n}")
        };
        let ops = vec![
            DocOp::Insert {
                anchor: Anchor::End,
                node: para("first"),
            },
            DocOp::Replace {
                id: "g1".into(),
                node: para("edited"),
            },
        ];
        doc.apply_all(ops, next_id).unwrap();
        assert_eq!(ids(&doc), vec!["g1"]);
    }

    #[test]
    fn apply_all_assigns_distinct_ids_to_multiple_inserts() {
        let mut doc = Doc::default();
        let mut n = 0;
        let next_id = || {
            n += 1;
            format!("g{n}")
        };
        let ops = vec![
            DocOp::Insert {
                anchor: Anchor::End,
                node: para("one"),
            },
            DocOp::Insert {
                anchor: Anchor::End,
                node: para("two"),
            },
        ];
        doc.apply_all(ops, next_id).unwrap();
        assert_eq!(ids(&doc), vec!["g1", "g2"]);
    }

    #[test]
    fn apply_all_rejects_a_batch_that_produces_an_invalid_doc() {
        let mut doc = seed();
        // A bare text node is inline; it may not sit directly under doc.
        let loose = node_json(json!({ "type": "text", "text": "loose" }));
        let err = doc
            .apply_all(
                vec![DocOp::Insert {
                    anchor: Anchor::End,
                    node: loose,
                }],
                || "z".into(),
            )
            .unwrap_err();
        assert_eq!(
            err,
            OpError::Invalid(PmError::DisallowedChild {
                parent: "doc".into(),
                child: "text".into()
            })
        );
        assert_eq!(ids(&doc), vec!["a", "b", "c"]); // unchanged
    }

    #[test]
    fn anchor_parse_keywords_and_relative() {
        assert_eq!(Anchor::parse("start").unwrap(), Anchor::Start);
        assert_eq!(Anchor::parse("end").unwrap(), Anchor::End);
        assert_eq!(
            Anchor::parse("after:b").unwrap(),
            Anchor::After { id: "b".into() }
        );
        assert_eq!(
            Anchor::parse("before:c").unwrap(),
            Anchor::Before { id: "c".into() }
        );
    }

    #[test]
    fn anchor_parse_rejects_garbage_and_empty_ids() {
        assert!(Anchor::parse("sideways").is_err());
        assert!(Anchor::parse("after").is_err());
        assert!(Anchor::parse("after:").is_err());
        assert!(Anchor::parse("before:").is_err());
    }

    #[test]
    fn from_stored_none_is_empty() {
        assert_eq!(Doc::from_stored(None), Doc::default());
    }

    #[test]
    fn from_stored_valid_doc_parses() {
        let doc = seed();
        let stored = doc.to_stored().unwrap();
        assert_eq!(Doc::from_stored(Some(&stored)), doc);
    }

    #[test]
    fn from_stored_non_doc_is_empty() {
        let not_a_doc = json!({ "type": "paragraph", "content": [] });
        assert_eq!(Doc::from_stored(Some(&not_a_doc)), Doc::default());
    }

    #[test]
    fn from_stored_garbage_is_empty() {
        assert_eq!(Doc::from_stored(Some(&json!("nonsense"))), Doc::default());
    }

    #[test]
    fn to_stored_empty_doc_is_none() {
        assert_eq!(Doc::default().to_stored(), None);
    }

    #[test]
    fn to_stored_nonempty_doc_is_some() {
        let doc = seed();
        let stored = doc.to_stored().expect("non-empty doc persists a value");
        assert_eq!(stored, serde_json::to_value(doc.node()).unwrap());
    }

    #[test]
    fn to_stored_then_from_stored_round_trips() {
        let doc = seed();
        let stored = doc.to_stored();
        assert_eq!(Doc::from_stored(stored.as_ref()), doc);
    }

    #[test]
    fn generated_id_is_eight_lowercase_alphanumerics() {
        let id = generate_block_id();
        assert_eq!(id.len(), 8, "id was {id:?}");
        assert!(
            id.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
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
    fn op_insert_serializes_with_tagged_wire_format() {
        let op = DocOp::Insert {
            anchor: Anchor::After { id: "b".into() },
            node: para("hi"),
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(
            v,
            json!({
                "op": "insert",
                "anchor": { "at": "after", "id": "b" },
                "node": { "type": "paragraph", "content": [{ "type": "text", "text": "hi" }] }
            })
        );
    }

    #[test]
    fn op_round_trips_through_json() {
        let op = DocOp::Move {
            id: "x".into(),
            anchor: Anchor::Start,
        };
        let back: DocOp = serde_json::from_value(serde_json::to_value(&op).unwrap()).unwrap();
        assert_eq!(op, back);
    }
}

/// Guards against schema drift between the Rust allow-list ([`NODES`]/[`MARKS`])
/// and the `TipTap` editor schema. `web/scripts/dump-pm-schema.ts` derives
/// `pm_schema.generated.json` from `getSchema(tiptapExtensions)`; `tempo check`
/// regenerates it when the extension files change and fails on a dirty diff.
/// This test closes the loop: the validator must permit exactly the node and
/// mark names the editor can emit — no more, no less.
#[cfg(test)]
mod schema_sync {
    use super::{MARKS, NODES};
    use std::collections::BTreeSet;

    #[derive(serde::Deserialize)]
    struct TiptapSchema {
        nodes: Vec<String>,
        marks: Vec<String>,
    }

    #[test]
    fn rust_allowlist_matches_tiptap_schema() {
        let tiptap: TiptapSchema = serde_json::from_str(include_str!("pm_schema.generated.json"))
            .expect("pm_schema.generated.json is valid JSON");

        let rust_nodes: BTreeSet<&str> = NODES.iter().map(|spec| spec.name).collect();
        let tiptap_nodes: BTreeSet<&str> = tiptap.nodes.iter().map(String::as_str).collect();
        assert_eq!(
            rust_nodes, tiptap_nodes,
            "node allow-list drifted from the TipTap schema; update NODES or \
             regenerate pm_schema.generated.json (`just check`)"
        );

        let rust_marks: BTreeSet<&str> = MARKS.iter().copied().collect();
        let tiptap_marks: BTreeSet<&str> = tiptap.marks.iter().map(String::as_str).collect();
        assert_eq!(
            rust_marks, tiptap_marks,
            "mark allow-list drifted from the TipTap schema; update MARKS or \
             regenerate pm_schema.generated.json (`just check`)"
        );
    }
}
