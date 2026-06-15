//! `ProseMirror`/`TipTap` document model and schema validation.
//!
//! The detail-page body is a single `TipTap` document. [`Node`] is a faithful
//! 1:1 mirror of its JSON — any node type round-trips untouched — and
//! [`Node::validate_document`] bounds what may be stored against an allow-list
//! schema. The [`Doc`] newtype hosts the block ops (insert, replace, delete,
//! move by id), addressing blocks at any depth. The editor already guarantees
//! well-formed structure;
//! validation here is the write-path safety net (defense-in-depth, the same
//! posture as the SSR sanitizer).

use std::collections::HashSet;

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
        // An atom block carrying media in attrs (src/alt/caption/kind); the Bun
        // renderer turns it into <figure> with an <img>/<video> + <figcaption>.
        name: "figure",
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
    /// A mark of `kind` carrying no attributes (`bold`, `italic`, …).
    pub fn new(kind: &str) -> Self {
        Self {
            r#type: kind.to_string(),
            attrs: Map::new(),
        }
    }

    /// A `link` mark pointing at `href`.
    pub fn link(href: &str) -> Self {
        let mut attrs = Map::new();
        attrs.insert("href".to_string(), Value::String(href.to_string()));
        Self {
            r#type: "link".to_string(),
            attrs,
        }
    }

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
    /// An empty block node of `kind` — no attrs, children, marks, or text. The
    /// caller fills in `attrs`/`content` as the node type requires.
    pub fn element(kind: &str) -> Self {
        Self {
            r#type: kind.to_string(),
            attrs: Map::new(),
            content: Vec::new(),
            marks: Vec::new(),
            text: None,
        }
    }

    /// A text node carrying `text` and the given inline `marks`.
    pub fn text(text: impl Into<String>, marks: Vec<Mark>) -> Self {
        Self {
            r#type: "text".to_string(),
            attrs: Map::new(),
            content: Vec::new(),
            marks,
            text: Some(text.into()),
        }
    }

    /// This block's stable id, if it carries one.
    pub fn block_id(&self) -> Option<&str> {
        self.attrs.get(ID_ATTR).and_then(Value::as_str)
    }

    /// The path of child indices from this node down to the descendant carrying
    /// `id` (e.g. `[1, 0]` ⇒ `content[1].content[0]`), or `None` if no descendant
    /// has it. Ids are unique within a document, so the first match is the only
    /// one. The returned path is always non-empty (it never names `self`).
    fn find_path(&self, id: &str) -> Option<Vec<usize>> {
        self.content.iter().enumerate().find_map(|(i, child)| {
            if child.block_id() == Some(id) {
                return Some(vec![i]);
            }
            child.find_path(id).map(|mut sub| {
                sub.insert(0, i);
                sub
            })
        })
    }

    /// Stamp `id` into this node's `attrs`, overwriting any id it arrived with.
    fn set_block_id(&mut self, id: &str) {
        self.attrs
            .insert(ID_ATTR.to_string(), Value::String(id.to_string()));
    }

    /// This node's own first line of text, from its direct text children only —
    /// the preview shown for a block in the outline. Container blocks (lists,
    /// blockquotes) have no direct text and so preview as empty, rather than
    /// echoing their descendants' text on every row.
    pub fn direct_text(&self) -> String {
        self.content
            .iter()
            .filter(|c| c.r#type == "text")
            .filter_map(|c| c.text.as_deref())
            .collect()
    }

    /// Whether this node belongs to the inline group (`text`, `hardBreak`).
    /// Unknown node types count as block-level so they still surface in the
    /// outline rather than vanishing.
    fn is_inline(&self) -> bool {
        matches!(
            NodeSpec::lookup(&self.r#type).map(|spec| spec.group),
            Some(Group::Inline)
        )
    }

    /// Whether this node is a known block-group node — exactly the set the
    /// editor's unique-id extension stamps. Unknown types are excluded (a write
    /// carrying one fails validation regardless).
    fn is_block(&self) -> bool {
        matches!(
            NodeSpec::lookup(&self.r#type).map(|spec| spec.group),
            Some(Group::Block)
        )
    }

    /// Record every block id in this subtree into `used`.
    fn collect_block_ids(&self, used: &mut HashSet<String>) {
        if let Some(id) = self.block_id() {
            used.insert(id.to_string());
        }
        for child in &self.content {
            child.collect_block_ids(used);
        }
    }

    /// Stamp a unique id on every block-group descendant that lacks one,
    /// drawing fresh ids from `gen` and skipping any that collide with `used`.
    /// `self` is never stamped — the doc root is not a block — only its subtree.
    fn stamp_missing_block_ids<F: FnMut() -> String>(
        &mut self,
        used: &mut HashSet<String>,
        gen_id: &mut F,
    ) {
        for child in &mut self.content {
            if child.is_block() && child.block_id().is_none() {
                let id = loop {
                    let candidate = gen_id();
                    if used.insert(candidate.clone()) {
                        break candidate;
                    }
                };
                child.set_block_id(&id);
            }
            child.stamp_missing_block_ids(used, gen_id);
        }
    }
}

/// A block's positional address: the child-index path from the document root,
/// so `[3, 0]` is the first child of the fourth top-level block. Rendered
/// jq-style with a leading dot and dotted indices — `.3.0` — which, unlike
/// bracket syntax, carries no shell glob metacharacters and needs no quoting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPath(Vec<usize>);

impl BlockPath {
    /// The child indices, root-to-leaf.
    pub fn indices(&self) -> &[usize] {
        &self.0
    }

    /// Parse the dotted path syntax. A leading dot is optional; bracketed
    /// (`[3][0]`) and explicit (`.content[3]`) jq forms are accepted too, so a
    /// path lifted from a jq expression still resolves. Every segment must be a
    /// non-negative integer index; anything else (e.g. a block id) is rejected,
    /// which is how a caller tells a path apart from an id.
    pub fn parse(s: &str) -> Result<Self, String> {
        let cleaned = s.replace("content", " ");
        let indices = cleaned
            .split(|c: char| matches!(c, '.' | '[' | ']') || c.is_whitespace())
            .filter(|seg| !seg.is_empty())
            .map(|seg| {
                seg.parse::<usize>()
                    .map_err(|_| format!("invalid path segment \"{seg}\" in \"{s}\""))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if indices.is_empty() {
            return Err(format!("empty path \"{s}\"; use indices like .3 or .3.0"));
        }
        Ok(Self(indices))
    }
}

impl std::fmt::Display for BlockPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in &self.0 {
            write!(f, ".{i}")?;
        }
        Ok(())
    }
}

/// How an op names an existing block: by stable id, or by positional path. On
/// the wire and the command line a locator is a single string — a leading dot
/// marks a path (`.3.0`), anything else is a block id (`a1b2c3d4`) — so both
/// address spaces share one surface. Paths work on any content (including
/// seeded blocks that carry no id); ids are stable across reordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Locator {
    Id(String),
    Path(BlockPath),
}

impl std::fmt::Display for Locator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => f.write_str(id),
            Self::Path(path) => write!(f, "{path}"),
        }
    }
}

impl std::str::FromStr for Locator {
    type Err = String;
    /// Disambiguate a locator string: a leading `.` or `[` is a positional path,
    /// anything else is a block id. (A bare `3` is therefore an id, not a path —
    /// paths must lead with a dot, which is also what `list` prints.) A leading
    /// `#` on an id is stripped, so the `#abc12345` form `list` prints pastes
    /// back in unchanged.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("empty locator; use a block id or a path like .3".to_string());
        }
        if s.starts_with('.') || s.starts_with('[') {
            BlockPath::parse(s).map(Self::Path)
        } else {
            // `list` prints ids with a leading `#`; accept that form so a copied
            // id resolves without the caller stripping the sigil first.
            let id = s.strip_prefix('#').unwrap_or(s);
            if id.is_empty() {
                return Err("empty locator; use a block id or a path like .3".to_string());
            }
            Ok(Self::Id(id.to_string()))
        }
    }
}

// Bare-string conversions always mean a block id — the unambiguous, common case
// for internal construction. Path locators are built explicitly or parsed from a
// locator string (`FromStr`), which is also what the wire format deserializes.
impl From<&str> for Locator {
    fn from(s: &str) -> Self {
        Self::Id(s.to_string())
    }
}
impl From<String> for Locator {
    fn from(s: String) -> Self {
        Self::Id(s)
    }
}

impl Serialize for Locator {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
impl<'de> Deserialize<'de> for Locator {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Where an insert or move lands. `Start`/`End` address the document's top-level
/// block list. `Before`/`After` land beside a located block, within that block's
/// own parent. `PrependTo`/`AppendTo` land inside a located container, as its
/// first/last child — which is how you add to an empty list or push onto one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "at", rename_all = "snake_case")]
pub enum Anchor {
    Start,
    End,
    After { id: Locator },
    Before { id: Locator },
    PrependTo { id: Locator },
    AppendTo { id: Locator },
}

impl Anchor {
    /// Parse the CLI anchor syntax: `start`, `end`, or `<kind>:<locator>` where
    /// kind is `before`/`after`/`prepend`/`append` and the locator is a path
    /// (`.3`) or block id. `into` is an alias for `append`.
    pub fn parse(s: &str) -> Result<Self, String> {
        if s == "start" {
            return Ok(Self::Start);
        }
        if s == "end" {
            return Ok(Self::End);
        }
        let (kind, rest) = s.split_once(':').ok_or_else(|| invalid_anchor(s))?;
        let id: Locator = rest.parse()?;
        match kind {
            "after" => Ok(Self::After { id }),
            "before" => Ok(Self::Before { id }),
            "prepend" => Ok(Self::PrependTo { id }),
            "append" | "into" => Ok(Self::AppendTo { id }),
            _ => Err(invalid_anchor(s)),
        }
    }
}

fn invalid_anchor(s: &str) -> String {
    format!(
        "invalid position \"{s}\"; use start, end, before:<loc>, after:<loc>, \
         prepend:<loc>, or append:<loc>"
    )
}

impl Anchor {
    /// Whether inserting several blocks one at a time at this anchor lands them
    /// in reverse order — true when the anchor names a fixed slot that each
    /// insert pushes the previous one away from (`start`, `prepend:` push from
    /// the front; `after:` inserts between the target and the prior insert).
    /// `end`/`append:` (slot grows) and `before:` (target drifts right) keep the
    /// natural order. [`DocOp::insert_sequence`] uses this to reverse its input
    /// only when needed, so a batch of single-node inserts reads forward.
    const fn fills_in_reverse(&self) -> bool {
        matches!(
            self,
            Self::Start | Self::PrependTo { .. } | Self::After { .. }
        )
    }
}

/// A single mutating operation over the document's blocks. The target and any
/// anchor are [`Locator`]s — a block id or a positional path — so ops address
/// blocks at any depth, with or without ids.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum DocOp {
    /// Insert `node` as a new block at `anchor`; the server assigns its id.
    Insert { anchor: Anchor, node: Node },
    /// Replace the located block with `node`, keeping its position and (if it had
    /// one) its id.
    Replace { id: Locator, node: Node },
    /// Remove the located block.
    Delete { id: Locator },
    /// Move the located block to `anchor`.
    Move { id: Locator, anchor: Anchor },
}

impl DocOp {
    /// Ops to insert `nodes` as a sequence at `anchor`, landing them in document
    /// order. Each becomes its own [`DocOp::Insert`] against the same anchor;
    /// the input is reversed first for anchors that would otherwise stack the
    /// blocks backwards (see [`Anchor::fills_in_reverse`]). Applied as one batch,
    /// the inserts need no knowledge of the ids the server will mint.
    pub fn insert_sequence(anchor: &Anchor, mut nodes: Vec<Node>) -> Vec<Self> {
        if anchor.fills_in_reverse() {
            nodes.reverse();
        }
        nodes
            .into_iter()
            .map(|node| Self::Insert {
                anchor: anchor.clone(),
                node,
            })
            .collect()
    }

    /// Ops to replace `target` with the first of `nodes`, inserting any rest
    /// immediately after it in order. The first block keeps the target's slot
    /// and id; an empty `nodes` yields no ops.
    pub fn replace_sequence(target: Locator, nodes: Vec<Node>) -> Vec<Self> {
        let mut nodes = nodes.into_iter();
        let Some(first) = nodes.next() else {
            return Vec::new();
        };
        let mut ops = vec![Self::Replace {
            id: target.clone(),
            node: first,
        }];
        let rest: Vec<Node> = nodes.collect();
        if !rest.is_empty() {
            ops.extend(Self::insert_sequence(&Anchor::After { id: target }, rest));
        }
        ops
    }
}

/// Structural failures applying an op, plus a schema failure on the result.
#[derive(Debug, Clone, PartialEq)]
pub enum OpError {
    /// No block with this id exists (target of replace/delete/move, or an anchor).
    NotFound(String),
    /// A freshly generated block id collided with an existing one.
    DuplicateId(String),
    /// A move anchored to itself or to one of its own descendants — once the
    /// block moves, that target no longer names a stable location.
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

    /// Validate the document against the schema.
    pub fn validate(&self) -> Result<(), PmError> {
        self.0.validate_document()
    }

    /// Borrow the block at a positional path, or `None` if any index is out of
    /// range (so a stale path fails cleanly rather than panicking).
    pub fn at_path(&self, path: &BlockPath) -> Option<&Node> {
        let mut node = &self.0;
        for &i in path.indices() {
            node = node.content.get(i)?;
        }
        Some(node)
    }

    /// A pre-order walk of the document's block nodes (everything but inline
    /// `text`/`hardBreak`), each paired with its positional path. This is the
    /// outline the CLI renders, and the path is the stable handle edits address.
    pub fn outline(&self) -> Vec<(BlockPath, &Node)> {
        fn walk<'a>(parent: &[usize], nodes: &'a [Node], out: &mut Vec<(BlockPath, &'a Node)>) {
            for (i, node) in nodes.iter().enumerate() {
                if node.is_inline() {
                    continue;
                }
                let mut path = parent.to_vec();
                path.push(i);
                out.push((BlockPath(path.clone()), node));
                walk(&path, &node.content, out);
            }
        }
        let mut out = Vec::new();
        walk(&[], &self.0.content, &mut out);
        out
    }

    /// Borrow the node at `path` — a sequence of child indices descending from
    /// the doc root (so an empty path is the root itself).
    fn node_at(&self, path: &[usize]) -> &Node {
        let mut node = &self.0;
        for &i in path {
            node = &node.content[i];
        }
        node
    }

    /// Borrow, mutably, the content vector of the node at `parent_path` (the doc
    /// root's own content when the path is empty).
    fn content_at_mut(&mut self, parent_path: &[usize]) -> &mut Vec<Node> {
        let mut node = &mut self.0;
        for &i in parent_path {
            node = &mut node.content[i];
        }
        &mut node.content
    }

    /// Borrow the block a locator names — by stable id or positional path — or
    /// `None` if it names nothing. The single read-side resolver every command
    /// shares, so id/`#id`/path handling can't drift between `get`, `replace`,
    /// `rm`, and `move`.
    pub fn at(&self, locator: &Locator) -> Option<&Node> {
        self.resolve_locator(locator)
            .map(|path| self.node_at(&path))
    }

    /// Resolve a locator to the path of the block it names, or `None` if it
    /// names nothing — an unknown id, or a path that runs off the tree.
    fn resolve_locator(&self, locator: &Locator) -> Option<Vec<usize>> {
        match locator {
            Locator::Id(id) => self.0.find_path(id),
            Locator::Path(path) => self.at_path(path).map(|_| path.indices().to_vec()),
        }
    }

    /// Resolve an anchor to an insertion point: the path of the parent whose
    /// content receives the node, and the index within it. `Start`/`End` address
    /// the document's top-level list; `Before`/`After` land beside the located
    /// block within its own parent; `PrependTo`/`AppendTo` land inside the
    /// located container as its first/last child.
    fn resolve_anchor(&self, anchor: &Anchor) -> Result<(Vec<usize>, usize), OpError> {
        let locate = |locator: &Locator| {
            self.resolve_locator(locator)
                .ok_or_else(|| OpError::NotFound(locator.to_string()))
        };
        match anchor {
            Anchor::Start => Ok((Vec::new(), 0)),
            Anchor::End => Ok((Vec::new(), self.0.content.len())),
            Anchor::Before { id } => {
                let path = locate(id)?;
                let (&index, parent) = path.split_last().expect("a found path is non-empty");
                Ok((parent.to_vec(), index))
            }
            Anchor::After { id } => {
                let path = locate(id)?;
                let (&index, parent) = path.split_last().expect("a found path is non-empty");
                Ok((parent.to_vec(), index + 1))
            }
            Anchor::PrependTo { id } => Ok((locate(id)?, 0)),
            Anchor::AppendTo { id } => {
                let path = locate(id)?;
                let len = self.node_at(&path).content.len();
                Ok((path, len))
            }
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
        if self.0.find_path(&new_id).is_some() {
            return Err(OpError::DuplicateId(new_id));
        }
        let (parent_path, at) = self.resolve_anchor(anchor)?;
        node.set_block_id(&new_id);
        self.content_at_mut(&parent_path).insert(at, node.clone());
        Ok(node)
    }

    /// Replace the located block with `node`, keeping its slot and — if the block
    /// carried a stable id — that id. A path-located block may have none, in
    /// which case the replacement keeps whatever id it arrived with (typically
    /// none).
    pub fn replace(&mut self, target: impl Into<Locator>, mut node: Node) -> Result<Node, OpError> {
        let target = target.into();
        let path = self
            .resolve_locator(&target)
            .ok_or_else(|| OpError::NotFound(target.to_string()))?;
        let existing_id = self.node_at(&path).block_id().map(str::to_string);
        if let Some(id) = existing_id {
            node.set_block_id(&id);
        }
        let (&index, parent) = path.split_last().expect("a found path is non-empty");
        self.content_at_mut(parent)[index] = node.clone();
        Ok(node)
    }

    /// Remove the located block, returning it.
    pub fn delete(&mut self, target: impl Into<Locator>) -> Result<Node, OpError> {
        let target = target.into();
        let path = self
            .resolve_locator(&target)
            .ok_or_else(|| OpError::NotFound(target.to_string()))?;
        let (&index, parent) = path.split_last().expect("a found path is non-empty");
        Ok(self.content_at_mut(parent).remove(index))
    }

    /// Move the located block to `anchor`, returning the moved block.
    pub fn move_block(
        &mut self,
        target: impl Into<Locator>,
        anchor: &Anchor,
    ) -> Result<Node, OpError> {
        let target = target.into();
        let src = self
            .resolve_locator(&target)
            .ok_or_else(|| OpError::NotFound(target.to_string()))?;
        // The destination can't sit inside the moved subtree (anchored to the
        // block itself or a descendant): once it moves, that target stops naming
        // a stable location. The located anchor node being src-or-below covers
        // every variant.
        if let Anchor::After { id }
        | Anchor::Before { id }
        | Anchor::PrependTo { id }
        | Anchor::AppendTo { id } = anchor
        {
            let anchor_path = self
                .resolve_locator(id)
                .ok_or_else(|| OpError::NotFound(id.to_string()))?;
            if anchor_path.starts_with(&src) {
                return Err(OpError::SelfAnchor);
            }
        }
        // Resolve the destination against the original tree (so positional paths
        // mean what the caller currently sees), detach the block, then shift the
        // destination to account for the gap the removal left where it passes
        // through the removed block's parent at a later index.
        let (mut ins_parent, mut ins_index) = self.resolve_anchor(anchor)?;
        let (&src_index, src_parent) = src.split_last().expect("a found path is non-empty");
        let node = self.content_at_mut(src_parent).remove(src_index);
        if ins_parent.starts_with(src_parent) {
            let depth = src_parent.len();
            if ins_parent.len() > depth {
                if ins_parent[depth] > src_index {
                    ins_parent[depth] -= 1;
                }
            } else if ins_index > src_index {
                ins_index -= 1;
            }
        }
        self.content_at_mut(&ins_parent)
            .insert(ins_index, node.clone());
        Ok(node)
    }

    /// Stamp a fresh id on every block-group node in the document that lacks
    /// one, leaving existing ids untouched. This is the same invariant the
    /// editor's unique-id extension maintains — the same block types
    /// ([`Group::Block`]) and the same id format — so a document written through
    /// ops converges on the shape the editor would have produced, with no
    /// per-op special-casing. [`Self::apply_all`] runs this on every write.
    pub fn ensure_block_ids<F: FnMut() -> String>(&mut self, mut gen_id: F) {
        let mut used = HashSet::new();
        self.0.collect_block_ids(&mut used);
        self.0.stamp_missing_block_ids(&mut used, &mut gen_id);
    }

    /// Apply one operation, sourcing a fresh id from `gen_id` for inserts.
    /// Returns the affected block. A building block for [`Self::apply_all`]; it
    /// does not validate the resulting document.
    pub fn apply(&mut self, op: DocOp, gen_id: impl FnOnce() -> String) -> Result<Node, OpError> {
        match op {
            DocOp::Insert { anchor, node } => self.insert(&anchor, node, gen_id()),
            DocOp::Replace { id, node } => self.replace(id, node),
            DocOp::Delete { id } => self.delete(id),
            DocOp::Move { id, anchor } => self.move_block(id, &anchor),
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
        working.ensure_block_ids(&mut gen_id);
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
        assert_eq!(doc.node().content.len(), 1);
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
        assert!(Doc::from_stored(Some(&garbage)).node().content.is_empty());
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
        let b = doc.at(&"b".into()).unwrap();
        assert_eq!(b.r#type, "heading");
    }

    #[test]
    fn replace_preserves_target_id_over_nodes_own_id() {
        let mut doc = seed();
        let mut incoming = para("x");
        incoming.set_block_id("wrong");
        doc.replace("b", incoming).unwrap();
        assert_eq!(ids(&doc), vec!["a", "b", "c"]);
        assert!(doc.at(&"wrong".into()).is_none());
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

    /// A nested document with ids at every depth:
    /// ```text
    /// doc
    ///   paragraph#p
    ///   bulletList#list
    ///     listItem#item1 > paragraph#ip1
    ///     listItem#item2 > paragraph#ip2
    ///   blockquote#quote > paragraph#qp
    /// ```
    fn nested_doc() -> Doc {
        Doc::parse(&json!({
            "type": "doc",
            "content": [
                { "type": "paragraph", "attrs": { "id": "p" },
                  "content": [{ "type": "text", "text": "top" }] },
                { "type": "bulletList", "attrs": { "id": "list" }, "content": [
                    { "type": "listItem", "attrs": { "id": "item1" }, "content": [
                        { "type": "paragraph", "attrs": { "id": "ip1" },
                          "content": [{ "type": "text", "text": "one" }] }
                    ]},
                    { "type": "listItem", "attrs": { "id": "item2" }, "content": [
                        { "type": "paragraph", "attrs": { "id": "ip2" },
                          "content": [{ "type": "text", "text": "two" }] }
                    ]}
                ]},
                { "type": "blockquote", "attrs": { "id": "quote" }, "content": [
                    { "type": "paragraph", "attrs": { "id": "qp" },
                      "content": [{ "type": "text", "text": "q" }] }
                ]}
            ]
        }))
        .expect("nested fixture is a valid document")
    }

    /// Find a node by id anywhere in the tree (test-side, independent of the
    /// addressing under test).
    fn find<'a>(node: &'a Node, id: &str) -> Option<&'a Node> {
        if node.block_id() == Some(id) {
            return Some(node);
        }
        node.content.iter().find_map(|child| find(child, id))
    }

    /// The block ids directly inside the node identified by `parent_id`.
    fn child_ids<'a>(doc: &'a Doc, parent_id: &str) -> Vec<&'a str> {
        find(doc.node(), parent_id)
            .map(|n| n.content.iter().filter_map(Node::block_id).collect())
            .unwrap_or_default()
    }

    #[test]
    fn delete_removes_a_nested_block() {
        let mut doc = nested_doc();
        let removed = doc.delete("item1").unwrap();
        assert_eq!(removed.block_id(), Some("item1"));
        assert_eq!(child_ids(&doc, "list"), vec!["item2"]);
        assert_eq!(ids(&doc), vec!["p", "list", "quote"]); // top level untouched
    }

    #[test]
    fn replace_swaps_a_nested_block_keeping_id() {
        let mut doc = nested_doc();
        let heading = node_json(json!({
            "type": "heading", "attrs": { "level": 3 },
            "content": [{ "type": "text", "text": "H" }]
        }));
        let replaced = doc.replace("ip1", heading).unwrap();
        assert_eq!(replaced.block_id(), Some("ip1"));
        assert_eq!(doc.at(&"ip1".into()).unwrap().r#type, "heading");
        assert_eq!(child_ids(&doc, "item1"), vec!["ip1"]); // still the only child
    }

    #[test]
    fn insert_after_a_nested_block_lands_in_that_parent() {
        let mut doc = nested_doc();
        let item = node_json(json!({
            "type": "listItem", "content": [
                { "type": "paragraph", "content": [{ "type": "text", "text": "new" }] }
            ]
        }));
        doc.insert(&Anchor::After { id: "item1".into() }, item, "item1b".into())
            .unwrap();
        assert_eq!(child_ids(&doc, "list"), vec!["item1", "item1b", "item2"]);
    }

    #[test]
    fn move_relocates_a_block_across_parents() {
        let mut doc = nested_doc();
        // Move the top-level paragraph into the blockquote, after its paragraph.
        doc.move_block("p", &Anchor::After { id: "qp".into() })
            .unwrap();
        assert_eq!(ids(&doc), vec!["list", "quote"]); // p left the top level
        assert_eq!(child_ids(&doc, "quote"), vec!["qp", "p"]);
    }

    #[test]
    fn move_lifts_a_deeply_nested_block_to_top_level() {
        let mut doc = nested_doc();
        doc.move_block("ip1", &Anchor::End).unwrap();
        assert!(child_ids(&doc, "item1").is_empty()); // item1 emptied
        assert_eq!(ids(&doc), vec!["p", "list", "quote", "ip1"]);
    }

    #[test]
    fn move_into_own_subtree_is_rejected() {
        let mut doc = nested_doc();
        // The list cannot be anchored after one of its own descendants — once it
        // moves, that descendant no longer names a stable location.
        assert_eq!(
            doc.move_block("list", &Anchor::After { id: "item1".into() })
                .unwrap_err(),
            OpError::SelfAnchor
        );
    }

    #[test]
    fn block_lookup_finds_a_nested_block() {
        let doc = nested_doc();
        assert_eq!(
            doc.at(&"ip2".into()).map(|n| n.r#type.as_str()),
            Some("paragraph")
        );
    }

    #[test]
    fn apply_all_rejects_an_illegal_relocation() {
        let mut doc = nested_doc();
        // Moving a paragraph to be a direct child of the bulletList (listItem-only)
        // must fail the whole-doc validation and leave the document untouched.
        let err = doc
            .apply_all(
                vec![DocOp::Move {
                    id: "p".into(),
                    anchor: Anchor::After { id: "item2".into() },
                }],
                || unreachable!("a move generates no id"),
            )
            .unwrap_err();
        assert_eq!(
            err,
            OpError::Invalid(PmError::DisallowedChild {
                parent: "bulletList".into(),
                child: "paragraph".into()
            })
        );
        assert_eq!(ids(&doc), vec!["p", "list", "quote"]);
        assert_eq!(child_ids(&doc, "list"), vec!["item1", "item2"]);
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;
    use serde_json::json;

    /// doc → paragraph#0, bulletList#1 [ listItem#1.0 > paragraph#1.0.0,
    /// listItem#1.1 > paragraph#1.1.0 ], blockquote#2 > paragraph#2.0.
    fn doc() -> Doc {
        Doc::parse(&json!({
            "type": "doc",
            "content": [
                { "type": "paragraph", "content": [{ "type": "text", "text": "intro" }] },
                { "type": "bulletList", "content": [
                    { "type": "listItem", "content": [
                        { "type": "paragraph", "content": [{ "type": "text", "text": "one" }] }
                    ]},
                    { "type": "listItem", "content": [
                        { "type": "paragraph", "content": [{ "type": "text", "text": "two" }] }
                    ]}
                ]},
                { "type": "blockquote", "content": [
                    { "type": "paragraph", "content": [{ "type": "text", "text": "quoted" }] }
                ]}
            ]
        }))
        .expect("valid fixture")
    }

    #[test]
    fn parse_accepts_dotted_paths_with_or_without_leading_dot() {
        assert_eq!(BlockPath::parse(".3").unwrap().indices(), &[3]);
        assert_eq!(BlockPath::parse("3").unwrap().indices(), &[3]);
        assert_eq!(BlockPath::parse(".3.0.1").unwrap().indices(), &[3, 0, 1]);
    }

    #[test]
    fn parse_accepts_jq_bracket_forms() {
        assert_eq!(BlockPath::parse("[3][0]").unwrap().indices(), &[3, 0]);
        assert_eq!(
            BlockPath::parse(".content[3].content[0]")
                .unwrap()
                .indices(),
            &[3, 0]
        );
    }

    #[test]
    fn parse_rejects_non_numeric_segments_and_empty() {
        assert!(BlockPath::parse("a1b2c3d4").is_err()); // looks like a block id
        assert!(BlockPath::parse(".3.x").is_err());
        assert!(BlockPath::parse(".").is_err());
        assert!(BlockPath::parse("").is_err());
    }

    #[test]
    fn display_round_trips_through_parse() {
        for s in [".0", ".3", ".3.0.1"] {
            let path = BlockPath::parse(s).unwrap();
            assert_eq!(path.to_string(), s);
            assert_eq!(BlockPath::parse(&path.to_string()).unwrap(), path);
        }
    }

    #[test]
    fn at_path_resolves_top_level_and_nested() {
        let doc = doc();
        assert_eq!(
            doc.at_path(&BlockPath::parse(".0").unwrap())
                .unwrap()
                .r#type,
            "paragraph"
        );
        assert_eq!(
            doc.at_path(&BlockPath::parse(".1").unwrap())
                .unwrap()
                .r#type,
            "bulletList"
        );
        assert_eq!(
            doc.at_path(&BlockPath::parse(".1.1.0").unwrap())
                .unwrap()
                .direct_text(),
            "two"
        );
    }

    #[test]
    fn at_path_out_of_range_is_none() {
        let doc = doc();
        assert!(doc.at_path(&BlockPath::parse(".9").unwrap()).is_none());
        assert!(doc.at_path(&BlockPath::parse(".1.0.5").unwrap()).is_none());
    }

    #[test]
    fn outline_walks_every_block_in_preorder_with_paths() {
        let doc = doc();
        let outline: Vec<(String, &str)> = doc
            .outline()
            .iter()
            .map(|(path, node)| (path.to_string(), node.r#type.as_str()))
            .collect();
        assert_eq!(
            outline,
            vec![
                (".0".into(), "paragraph"),
                (".1".into(), "bulletList"),
                (".1.0".into(), "listItem"),
                (".1.0.0".into(), "paragraph"),
                (".1.1".into(), "listItem"),
                (".1.1.0".into(), "paragraph"),
                (".2".into(), "blockquote"),
                (".2.0".into(), "paragraph"),
            ]
        );
    }

    #[test]
    fn outline_omits_inline_text_nodes() {
        let doc = doc();
        assert!(
            doc.outline().iter().all(|(_, node)| node.r#type != "text"),
            "text nodes must not appear in the block outline"
        );
    }

    #[test]
    fn direct_text_is_own_line_not_descendants() {
        let doc = doc();
        // The bulletList has no direct text children, only nested blocks.
        let list = doc.at_path(&BlockPath::parse(".1").unwrap()).unwrap();
        assert_eq!(list.direct_text(), "");
        // Its inner paragraph carries the actual text.
        let para = doc.at_path(&BlockPath::parse(".1.0.0").unwrap()).unwrap();
        assert_eq!(para.direct_text(), "one");
    }
}

#[cfg(test)]
mod locator_tests {
    use super::*;
    use serde_json::json;

    fn para(text: &str) -> Node {
        serde_json::from_value(
            json!({ "type": "paragraph", "content": [{ "type": "text", "text": text }] }),
        )
        .unwrap()
    }

    fn list_item(text: &str) -> Node {
        serde_json::from_value(json!({
            "type": "listItem",
            "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": text }] }]
        }))
        .unwrap()
    }

    /// An id-less seeded-style doc: a paragraph and a two-item bullet list.
    fn idless() -> Doc {
        Doc::parse(&json!({
            "type": "doc",
            "content": [
                { "type": "paragraph", "content": [{ "type": "text", "text": "intro" }] },
                { "type": "bulletList", "content": [
                    list_item("one"),
                    list_item("two")
                ]}
            ]
        }))
        .expect("valid fixture")
    }

    fn loc(s: &str) -> Locator {
        s.parse().expect("valid locator")
    }

    fn para_texts(doc: &Doc) -> Vec<String> {
        doc.outline()
            .iter()
            .filter(|(_, n)| n.r#type == "paragraph")
            .map(|(_, n)| n.direct_text())
            .collect()
    }

    #[test]
    fn from_str_disambiguates_path_from_id() {
        assert_eq!(
            loc(".3.0"),
            Locator::Path(BlockPath::parse(".3.0").unwrap())
        );
        assert_eq!(loc("a1b2c3d4"), Locator::Id("a1b2c3d4".into()));
        assert!("".parse::<Locator>().is_err());
    }

    #[test]
    fn from_str_accepts_the_hash_prefixed_id_that_list_prints() {
        // `content list` prints ids as `#abc12345`; pasting that form verbatim
        // must resolve to the same block as the bare id.
        assert_eq!(loc("#a1b2c3d4"), Locator::Id("a1b2c3d4".into()));
        assert_eq!(loc("a1b2c3d4"), Locator::Id("a1b2c3d4".into()));
        // A lone `#` names nothing.
        assert!("#".parse::<Locator>().is_err());
    }

    #[test]
    fn at_resolves_both_locator_forms() {
        let mut doc = Doc::default();
        doc.insert(&Anchor::End, para("hello"), "keepid12".into())
            .unwrap();
        assert_eq!(doc.at(&loc(".0")).unwrap().direct_text(), "hello");
        assert_eq!(doc.at(&loc("keepid12")).unwrap().direct_text(), "hello");
        // The `#abc` form `list` prints resolves to the same block.
        assert_eq!(doc.at(&loc("#keepid12")).unwrap().direct_text(), "hello");
        assert!(doc.at(&loc("nope0000")).is_none());
        assert!(doc.at(&loc(".9")).is_none());
    }

    #[test]
    fn round_trips_through_json_as_a_bare_string() {
        assert_eq!(
            serde_json::to_value(Locator::Id("abc".into())).unwrap(),
            json!("abc")
        );
        assert_eq!(serde_json::to_value(loc(".3.0")).unwrap(), json!(".3.0"));
        for value in [json!("abc"), json!(".3.0")] {
            let parsed: Locator = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(parsed).unwrap(), value);
        }
    }

    #[test]
    fn replace_by_path_edits_idless_content() {
        let mut doc = idless();
        doc.replace(loc(".0"), para("changed")).unwrap();
        assert_eq!(
            doc.at_path(&BlockPath::parse(".0").unwrap())
                .unwrap()
                .direct_text(),
            "changed"
        );
    }

    #[test]
    fn replace_by_path_keeps_an_existing_id() {
        let mut doc = Doc::default();
        doc.insert(&Anchor::End, para("first"), "keep".into())
            .unwrap();
        doc.replace(loc(".0"), para("edited")).unwrap();
        assert_eq!(
            doc.at_path(&BlockPath::parse(".0").unwrap())
                .unwrap()
                .block_id(),
            Some("keep")
        );
    }

    #[test]
    fn delete_by_nested_path() {
        let mut doc = idless();
        doc.delete(loc(".1.0")).unwrap(); // first list item
        assert_eq!(para_texts(&doc), vec!["intro", "two"]);
    }

    #[test]
    fn append_to_container_adds_a_last_child() {
        let mut doc = idless();
        doc.insert(
            &Anchor::AppendTo { id: loc(".1") },
            list_item("three"),
            "x".into(),
        )
        .unwrap();
        assert_eq!(
            doc.at_path(&BlockPath::parse(".1.2.0").unwrap())
                .unwrap()
                .direct_text(),
            "three"
        );
    }

    #[test]
    fn prepend_to_container_adds_a_first_child() {
        let mut doc = idless();
        doc.insert(
            &Anchor::PrependTo { id: loc(".1") },
            list_item("zero"),
            "x".into(),
        )
        .unwrap();
        assert_eq!(
            doc.at_path(&BlockPath::parse(".1.0.0").unwrap())
                .unwrap()
                .direct_text(),
            "zero"
        );
    }

    #[test]
    fn move_by_path_addresses_the_pre_move_tree() {
        let mut doc = Doc::default();
        for (text, id) in [("a", "a"), ("b", "b"), ("c", "c")] {
            doc.insert(&Anchor::End, para(text), id.into()).unwrap();
        }
        // "move .0 to after:.2" means after the block currently at .2.
        doc.move_block(loc(".0"), &Anchor::After { id: loc(".2") })
            .unwrap();
        assert_eq!(para_texts(&doc), vec!["b", "c", "a"]);
    }

    #[test]
    fn append_into_own_subtree_is_rejected() {
        let mut doc = idless();
        assert_eq!(
            doc.move_block(loc(".1"), &Anchor::AppendTo { id: loc(".1") })
                .unwrap_err(),
            OpError::SelfAnchor
        );
    }

    #[test]
    fn anchor_parse_handles_prepend_append_and_into() {
        assert_eq!(
            Anchor::parse("prepend:.1").unwrap(),
            Anchor::PrependTo { id: loc(".1") }
        );
        assert_eq!(
            Anchor::parse("append:.1").unwrap(),
            Anchor::AppendTo { id: loc(".1") }
        );
        assert_eq!(
            Anchor::parse("into:abc").unwrap(),
            Anchor::AppendTo { id: "abc".into() }
        );
        assert!(Anchor::parse("sideways:.1").is_err());
    }
}

#[cfg(test)]
mod sequence_tests {
    use super::*;
    use serde_json::json;

    fn para(text: &str) -> Node {
        serde_json::from_value(
            json!({ "type": "paragraph", "content": [{ "type": "text", "text": text }] }),
        )
        .unwrap()
    }

    fn list_item(text: &str) -> Node {
        serde_json::from_value(json!({
            "type": "listItem",
            "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": text }] }]
        }))
        .unwrap()
    }

    /// Apply `ops` atomically with deterministic ids, then read every
    /// paragraph's text in document order.
    fn apply(doc: &mut Doc, ops: Vec<DocOp>) {
        let mut n = 0;
        doc.apply_all(ops, || {
            n += 1;
            format!("g{n}")
        })
        .expect("sequence applies cleanly");
    }

    fn para_texts(doc: &Doc) -> Vec<String> {
        doc.outline()
            .iter()
            .filter(|(_, node)| node.r#type == "paragraph")
            .map(|(_, node)| node.direct_text())
            .collect()
    }

    fn seeded(texts: &[&str]) -> Doc {
        let mut doc = Doc::default();
        for text in texts {
            doc.insert(&Anchor::End, para(text), (*text).into())
                .unwrap();
        }
        doc
    }

    #[test]
    fn insert_sequence_at_end_keeps_order() {
        let mut doc = seeded(&["x"]);
        apply(
            &mut doc,
            DocOp::insert_sequence(&Anchor::End, vec![para("a"), para("b"), para("c")]),
        );
        assert_eq!(para_texts(&doc), vec!["x", "a", "b", "c"]);
    }

    #[test]
    fn insert_sequence_at_start_keeps_order() {
        let mut doc = seeded(&["x"]);
        apply(
            &mut doc,
            DocOp::insert_sequence(&Anchor::Start, vec![para("a"), para("b"), para("c")]),
        );
        assert_eq!(para_texts(&doc), vec!["a", "b", "c", "x"]);
    }

    #[test]
    fn insert_sequence_after_keeps_order() {
        let mut doc = seeded(&["x", "y"]);
        apply(
            &mut doc,
            DocOp::insert_sequence(
                &Anchor::After { id: "x".into() },
                vec![para("a"), para("b"), para("c")],
            ),
        );
        assert_eq!(para_texts(&doc), vec!["x", "a", "b", "c", "y"]);
    }

    #[test]
    fn insert_sequence_before_keeps_order() {
        let mut doc = seeded(&["x", "y"]);
        apply(
            &mut doc,
            DocOp::insert_sequence(
                &Anchor::Before { id: "y".into() },
                vec![para("a"), para("b"), para("c")],
            ),
        );
        assert_eq!(para_texts(&doc), vec!["x", "a", "b", "c", "y"]);
    }

    #[test]
    fn insert_sequence_append_to_container_keeps_order() {
        let mut doc = Doc::default();
        doc.insert(
            &Anchor::End,
            serde_json::from_value(json!({ "type": "bulletList", "content": [] })).unwrap(),
            "list".into(),
        )
        .unwrap();
        apply(
            &mut doc,
            DocOp::insert_sequence(
                &Anchor::AppendTo { id: "list".into() },
                vec![list_item("a"), list_item("b"), list_item("c")],
            ),
        );
        assert_eq!(para_texts(&doc), vec!["a", "b", "c"]);
    }

    #[test]
    fn insert_sequence_prepend_to_container_keeps_order() {
        let mut doc = Doc::default();
        doc.insert(
            &Anchor::End,
            serde_json::from_value(json!({
                "type": "bulletList",
                "content": [{ "type": "listItem", "content": [
                    { "type": "paragraph", "content": [{ "type": "text", "text": "z" }] }
                ]}]
            }))
            .unwrap(),
            "list".into(),
        )
        .unwrap();
        apply(
            &mut doc,
            DocOp::insert_sequence(
                &Anchor::PrependTo { id: "list".into() },
                vec![list_item("a"), list_item("b"), list_item("c")],
            ),
        );
        assert_eq!(para_texts(&doc), vec!["a", "b", "c", "z"]);
    }

    #[test]
    fn insert_sequence_of_one_is_a_single_insert() {
        let ops = DocOp::insert_sequence(&Anchor::End, vec![para("only")]);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], DocOp::Insert { .. }));
    }

    #[test]
    fn replace_sequence_single_block_just_replaces() {
        let mut doc = seeded(&["x", "y"]);
        apply(
            &mut doc,
            DocOp::replace_sequence("x".into(), vec![para("a")]),
        );
        assert_eq!(para_texts(&doc), vec!["a", "y"]);
        // the replaced block keeps its id
        assert_eq!(doc.at(&"x".into()).map(Node::direct_text), Some("a".into()));
    }

    #[test]
    fn replace_sequence_multi_block_replaces_then_inserts_rest_in_order() {
        let mut doc = seeded(&["x", "y"]);
        apply(
            &mut doc,
            DocOp::replace_sequence("x".into(), vec![para("a"), para("b"), para("c")]),
        );
        assert_eq!(para_texts(&doc), vec!["a", "b", "c", "y"]);
        // the first block lands in the target's slot, keeping its id
        assert_eq!(doc.at(&"x".into()).map(Node::direct_text), Some("a".into()));
    }

    #[test]
    fn replace_sequence_by_path_keeps_order() {
        let mut doc = seeded(&["x", "y"]);
        apply(
            &mut doc,
            DocOp::replace_sequence(".0".parse().unwrap(), vec![para("a"), para("b")]),
        );
        assert_eq!(para_texts(&doc), vec!["a", "b", "y"]);
    }
}

#[cfg(test)]
mod block_id_tests {
    use super::*;
    use serde_json::json;

    fn doc(value: &Value) -> Doc {
        Doc::parse(value).expect("valid fixture")
    }

    /// Every block id present in the subtree, in pre-order.
    fn all_ids(node: &Node) -> Vec<String> {
        let mut out = Vec::new();
        fn walk(node: &Node, out: &mut Vec<String>) {
            if let Some(id) = node.block_id() {
                out.push(id.to_string());
            }
            for child in &node.content {
                walk(child, out);
            }
        }
        walk(node, &mut out);
        out
    }

    fn counter() -> impl FnMut() -> String {
        let mut n = 0;
        move || {
            n += 1;
            format!("g{n}")
        }
    }

    #[test]
    fn ensure_block_ids_stamps_every_block_node_missing_one() {
        let mut doc = doc(&json!({ "type": "doc", "content": [
            { "type": "paragraph", "content": [{ "type": "text", "text": "p" }] },
            { "type": "bulletList", "content": [
                { "type": "listItem", "content": [
                    { "type": "paragraph", "content": [{ "type": "text", "text": "i" }] }
                ]}
            ]}
        ]}));
        doc.ensure_block_ids(counter());
        // paragraph, bulletList, listItem, nested paragraph — four block nodes.
        assert_eq!(all_ids(doc.node()).len(), 4);
        // the doc root and the inline text nodes are never stamped.
        assert!(doc.node().block_id().is_none());
        let para = doc.at_path(&BlockPath::parse(".0").unwrap()).unwrap();
        assert!(
            para.content[0].block_id().is_none(),
            "text node was stamped"
        );
    }

    #[test]
    fn ensure_block_ids_preserves_existing_ids() {
        let mut doc = doc(&json!({ "type": "doc", "content": [
            { "type": "paragraph", "attrs": { "id": "keep" },
              "content": [{ "type": "text", "text": "a" }] },
            { "type": "paragraph", "content": [{ "type": "text", "text": "b" }] }
        ]}));
        doc.ensure_block_ids(counter());
        assert_eq!(doc.node().content[0].block_id(), Some("keep"));
        assert_eq!(doc.node().content[1].block_id(), Some("g1"));
    }

    #[test]
    fn ensure_block_ids_does_not_collide_with_an_existing_id() {
        // The generator's first output (`g1`) already exists, so the unstamped
        // block must skip to the next free id rather than duplicate it.
        let mut doc = doc(&json!({ "type": "doc", "content": [
            { "type": "paragraph", "attrs": { "id": "g1" },
              "content": [{ "type": "text", "text": "a" }] },
            { "type": "paragraph", "content": [{ "type": "text", "text": "b" }] }
        ]}));
        doc.ensure_block_ids(counter());
        assert_eq!(doc.node().content[1].block_id(), Some("g2"));
    }

    #[test]
    fn apply_all_backfills_ids_across_the_whole_document() {
        // A seeded-style document with no ids anywhere.
        let mut doc = doc(&json!({ "type": "doc", "content": [
            { "type": "paragraph", "content": [{ "type": "text", "text": "seed" }] }
        ]}));
        assert!(doc.node().content[0].block_id().is_none());
        let new_para = serde_json::from_value(json!({ "type": "paragraph",
                "content": [{ "type": "text", "text": "added" }] }))
        .unwrap();
        doc.apply_all(
            vec![DocOp::Insert {
                anchor: Anchor::End,
                node: new_para,
            }],
            counter(),
        )
        .unwrap();
        // both the pre-existing seed block and the inserted block carry ids.
        assert!(doc.node().content.iter().all(|b| b.block_id().is_some()));
    }

    #[test]
    fn apply_all_stamps_nested_blocks_of_an_inserted_container() {
        let mut doc = Doc::default();
        let list = serde_json::from_value(json!({ "type": "bulletList", "content": [
            { "type": "listItem", "content": [
                { "type": "paragraph", "content": [{ "type": "text", "text": "x" }] }
            ]}
        ]}))
        .unwrap();
        doc.apply_all(
            vec![DocOp::Insert {
                anchor: Anchor::End,
                node: list,
            }],
            counter(),
        )
        .unwrap();
        let list = &doc.node().content[0];
        assert!(list.block_id().is_some(), "bulletList");
        assert!(list.content[0].block_id().is_some(), "listItem");
        assert!(list.content[0].content[0].block_id().is_some(), "paragraph");
    }
}

/// Guards against schema drift between the Rust allow-list ([`NODES`]/[`MARKS`])
/// and the `TipTap` editor schema. `web/scripts/dump-pm-schema.ts` derives
/// `pm_schema.generated.json` from `getSchema(tiptapExtensions)`; `tempo check`
/// regenerates it when the extension files change and fails on a dirty diff.
/// This test closes the loop: the validator must permit exactly the node and
/// mark names the editor can emit, and id stamping ([`Doc::ensure_block_ids`])
/// must cover exactly the node types the editor's unique-id extension does — no
/// more, no less.
#[cfg(test)]
mod schema_sync {
    use super::{Group, MARKS, NODES};
    use std::collections::BTreeSet;

    #[derive(serde::Deserialize)]
    struct TiptapSchema {
        nodes: Vec<String>,
        marks: Vec<String>,
        #[serde(rename = "idTypes")]
        id_types: Vec<String>,
    }

    fn tiptap() -> TiptapSchema {
        serde_json::from_str(include_str!("pm_schema.generated.json"))
            .expect("pm_schema.generated.json is valid JSON")
    }

    #[test]
    fn rust_allowlist_matches_tiptap_schema() {
        let tiptap = tiptap();

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

    #[test]
    fn block_group_matches_editor_id_types() {
        let tiptap = tiptap();

        // The set Rust stamps ids on (every block-group node) must equal the
        // set the editor's unique-id extension stamps, or content written
        // through ops and content saved from the editor would carry ids on
        // different nodes.
        let rust_blocks: BTreeSet<&str> = NODES
            .iter()
            .filter(|spec| matches!(spec.group, Group::Block))
            .map(|spec| spec.name)
            .collect();
        let editor_id_types: BTreeSet<&str> = tiptap.id_types.iter().map(String::as_str).collect();
        assert_eq!(
            rust_blocks, editor_id_types,
            "block-group nodes drifted from the editor's unique-id types; update \
             the uniqueId config or NODES, then regenerate pm_schema.generated.json"
        );
    }
}
