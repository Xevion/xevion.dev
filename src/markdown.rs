//! Markdown → `ProseMirror` block conversion for the content CLI.
//!
//! The `--md` authoring path runs `CommonMark` (via `pulldown-cmark`, with
//! strikethrough enabled) through a small stack machine that mirrors the
//! detail-page schema in `crate::pm`. Headings shift down a level so the body's
//! `#` sits beneath the page title's h1 (`#` → h2 … deeper clamps to h4). The
//! result is plain [`Node`]s; the server still validates them on the write path,
//! so anything the schema forbids fails there rather than here.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde_json::Value;

use crate::pm::{Mark, Node};

/// Why a Markdown snippet couldn't be turned into document blocks.
#[derive(Debug, PartialEq, Eq)]
pub enum MarkdownError {
    /// The input held no block content (empty or whitespace only).
    Empty,
    /// A Markdown construct with no equivalent in the detail schema.
    Unsupported(&'static str),
}

impl std::fmt::Display for MarkdownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("the Markdown produced no content"),
            Self::Unsupported(what) => {
                write!(f, "{what} are not supported in detail content")
            }
        }
    }
}

impl std::error::Error for MarkdownError {}

/// Convert a Markdown snippet into a sequence of detail-document blocks.
pub fn to_blocks(markdown: &str) -> Result<Vec<Node>, MarkdownError> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let mut converter = Converter::default();
    for event in Parser::new_ext(markdown, options) {
        converter.handle(event)?;
    }
    let blocks = converter.finish();
    if blocks.is_empty() {
        Err(MarkdownError::Empty)
    } else {
        Ok(blocks)
    }
}

/// A stack machine over `pulldown-cmark`'s pre-order event stream. Open block
/// nodes live on `stack` (innermost last); active inline marks on `marks`;
/// finished top-level blocks accumulate in `blocks`. A tight list item emits
/// inline text with no enclosing paragraph, so inline content opens an implicit
/// paragraph that closes when its container does.
#[derive(Default)]
struct Converter {
    blocks: Vec<Node>,
    stack: Vec<Node>,
    marks: Vec<Mark>,
}

impl Converter {
    fn handle(&mut self, event: Event) -> Result<(), MarkdownError> {
        match event {
            Event::Start(tag) => self.start(tag)?,
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.append_text(&text),
            Event::Code(code) => self.append_code(&code),
            Event::SoftBreak => self.append_text(" "),
            Event::HardBreak => self.push_inline(Node::element("hardBreak")),
            Event::Rule => {
                self.close_dangling_paragraph();
                self.attach(Node::element("horizontalRule"));
            }
            Event::Html(_) | Event::InlineHtml(_) => {
                return Err(MarkdownError::Unsupported("raw HTML"));
            }
            Event::FootnoteReference(_) => return Err(MarkdownError::Unsupported("footnotes")),
            Event::TaskListMarker(_) => return Err(MarkdownError::Unsupported("task lists")),
            Event::InlineMath(_) | Event::DisplayMath(_) => {
                return Err(MarkdownError::Unsupported("math"));
            }
        }
        Ok(())
    }

    fn start(&mut self, tag: Tag) -> Result<(), MarkdownError> {
        // Inline marks decorate the open block rather than opening one, so they
        // must not flush a paragraph mid-line.
        match &tag {
            Tag::Emphasis => {
                self.marks.push(Mark::new("italic"));
                return Ok(());
            }
            Tag::Strong => {
                self.marks.push(Mark::new("bold"));
                return Ok(());
            }
            Tag::Strikethrough => {
                self.marks.push(Mark::new("strike"));
                return Ok(());
            }
            Tag::Link { dest_url, .. } => {
                self.marks.push(Mark::link(dest_url));
                return Ok(());
            }
            _ => {}
        }
        self.close_dangling_paragraph();
        let block = match tag {
            Tag::Paragraph => Node::element("paragraph"),
            Tag::Heading { level, .. } => {
                let mut node = Node::element("heading");
                node.attrs
                    .insert("level".to_string(), Value::from(shift_heading(level)));
                node
            }
            Tag::BlockQuote(_) => Node::element("blockquote"),
            Tag::CodeBlock(kind) => {
                let mut node = Node::element("codeBlock");
                if let CodeBlockKind::Fenced(language) = kind
                    && !language.is_empty()
                {
                    node.attrs
                        .insert("language".to_string(), Value::from(language.into_string()));
                }
                node
            }
            Tag::List(first) => {
                let mut node = Node::element(if first.is_some() {
                    "orderedList"
                } else {
                    "bulletList"
                });
                if let Some(start) = first
                    && start != 1
                {
                    node.attrs.insert("start".to_string(), Value::from(start));
                }
                node
            }
            Tag::Item => Node::element("listItem"),
            Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link { .. } => {
                unreachable!("inline marks handled above")
            }
            Tag::Image { .. } => return Err(MarkdownError::Unsupported("images")),
            Tag::Table(_) | Tag::TableHead | Tag::TableRow | Tag::TableCell => {
                return Err(MarkdownError::Unsupported("tables"));
            }
            Tag::FootnoteDefinition(_) => return Err(MarkdownError::Unsupported("footnotes")),
            Tag::HtmlBlock => return Err(MarkdownError::Unsupported("raw HTML")),
            Tag::DefinitionList | Tag::DefinitionListTitle | Tag::DefinitionListDefinition => {
                return Err(MarkdownError::Unsupported("definition lists"));
            }
            Tag::MetadataBlock(_) => return Err(MarkdownError::Unsupported("metadata blocks")),
            Tag::Superscript | Tag::Subscript => {
                return Err(MarkdownError::Unsupported("super/subscript"));
            }
        };
        self.stack.push(block);
        Ok(())
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => self.finish_block(),
            TagEnd::CodeBlock => self.finish_code_block(),
            TagEnd::Heading(_) | TagEnd::BlockQuote(_) | TagEnd::List(_) | TagEnd::Item => {
                self.close_dangling_paragraph();
                self.finish_block();
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => {
                self.marks.pop();
            }
            _ => {}
        }
    }

    /// Close a code block, dropping the single trailing newline `pulldown-cmark`
    /// appends to fenced content (and the now-empty text node if that's all
    /// there was).
    fn finish_code_block(&mut self) {
        if let Some(code) = self.stack.last_mut() {
            if let Some(text) = code.content.last_mut().and_then(|node| node.text.as_mut())
                && text.ends_with('\n')
            {
                text.pop();
            }
            if code
                .content
                .last()
                .is_some_and(|node| node.text.as_deref() == Some(""))
            {
                code.content.pop();
            }
        }
        self.finish_block();
    }

    /// Append text to the current inline container under the active marks,
    /// merging into the previous run when its marks match (so soft breaks and
    /// split runs collapse into one text node). Text inside a code block carries
    /// no marks.
    fn append_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.ensure_inline_container();
        let parent = self.stack.last_mut().expect("a container is open");
        let marks = if parent.r#type == "codeBlock" {
            Vec::new()
        } else {
            self.marks.clone()
        };
        if let Some(last) = parent.content.last_mut()
            && last.r#type == "text"
            && last.marks == marks
            && let Some(existing) = last.text.as_mut()
        {
            existing.push_str(text);
            return;
        }
        parent.content.push(Node::text(text, marks));
    }

    /// Append an inline code span: a text run carrying the `code` mark on top of
    /// any active marks. Its own run, never merged with adjacent text. A trailing
    /// `{:lang}` / `{:.token}` hint is peeled off into the mark's attrs.
    fn append_code(&mut self, code: &str) {
        self.ensure_inline_container();
        let (text, hint) = split_code_hint(code);
        let mut code_mark = Mark::new("code");
        if let Some(hint) = hint {
            let (key, value) = hint.into_attr();
            code_mark.attrs.insert(key.to_string(), Value::from(value));
        }
        let mut marks = self.marks.clone();
        marks.push(code_mark);
        self.stack
            .last_mut()
            .expect("a container is open")
            .content
            .push(Node::text(text, marks));
    }

    fn push_inline(&mut self, node: Node) {
        self.ensure_inline_container();
        self.stack
            .last_mut()
            .expect("a container is open")
            .content
            .push(node);
    }

    /// Ensure the top of the stack accepts inline content, opening an implicit
    /// paragraph when it doesn't (a tight list item, blockquote, or the document
    /// root receiving bare text).
    fn ensure_inline_container(&mut self) {
        let accepts = matches!(
            self.stack.last().map(|node| node.r#type.as_str()),
            Some("paragraph" | "heading" | "codeBlock")
        );
        if !accepts {
            self.stack.push(Node::element("paragraph"));
        }
    }

    /// Close an implicit paragraph an inline run opened inside a container. An
    /// explicit paragraph is always balanced by its own end event before any
    /// sibling block, so a paragraph still on top here is necessarily implicit.
    fn close_dangling_paragraph(&mut self) {
        if matches!(
            self.stack.last().map(|node| node.r#type.as_str()),
            Some("paragraph")
        ) {
            self.finish_block();
        }
    }

    /// Pop the open block and attach it to its parent (or the top-level list).
    fn finish_block(&mut self) {
        if let Some(node) = self.stack.pop() {
            self.attach(node);
        }
    }

    fn attach(&mut self, node: Node) {
        match self.stack.last_mut() {
            Some(parent) => parent.content.push(node),
            None => self.blocks.push(node),
        }
    }

    fn finish(mut self) -> Vec<Node> {
        while !self.stack.is_empty() {
            self.finish_block();
        }
        self.blocks
    }
}

/// Map a Markdown heading level into the schema's h2–h4 band: shift down one
/// level so `#` becomes h2 (the page title is the h1), clamping deeper levels
/// to h4.
fn shift_heading(level: HeadingLevel) -> u8 {
    (level as u8 + 1).min(4)
}

/// A highlight hint peeled off a code span's trailing `{:…}` suffix: either a
/// Shiki grammar (`lang`) or an author-declared semantic token (`token`). The two
/// map to the `code` mark's `lang` / `token` attrs respectively.
enum Hint<'a> {
    Lang(&'a str),
    Token(&'a str),
}

impl<'a> Hint<'a> {
    /// The `(attr-key, value)` this hint writes onto the `code` mark.
    const fn into_attr(self) -> (&'static str, &'a str) {
        match self {
            Self::Lang(value) => ("lang", value),
            Self::Token(value) => ("token", value),
        }
    }
}

/// Split an inline-code span into its content and an optional trailing highlight
/// hint, following Shiki's tailing-curly-colon convention: `…{:lang}` requests a
/// grammar (`lang` attr), `…{:.kind}` a semantic token color (`token` attr). Only
/// the value's *shape* is checked here — the write path validates `token` against
/// the vocabulary, and `lang` is permissive (an unknown grammar degrades to plain
/// at render). A span with no well-formed suffix is returned verbatim, so literal
/// braces survive; the raw `--node` path is the escape hatch for a literal `{:…}`.
fn split_code_hint(code: &str) -> (&str, Option<Hint<'_>>) {
    let Some(inner) = code.strip_suffix('}') else {
        return (code, None);
    };
    let Some(open) = inner.rfind("{:") else {
        return (code, None);
    };
    let text = &code[..open];
    let spec = &inner[open + 2..];
    if text.is_empty() {
        return (code, None);
    }
    // `{:.kind}` is a semantic token; a bare `{:lang}` a grammar. A dotted spec can
    // never also match the `lang` arm — the leading `.` fails `is_hint_value` — so
    // the two checks don't overlap and an empty spec falls through to literal.
    if let Some(kind) = spec.strip_prefix('.')
        && is_hint_value(kind)
    {
        return (text, Some(Hint::Token(kind)));
    }
    if is_hint_value(spec) {
        return (text, Some(Hint::Lang(spec)));
    }
    (code, None)
}

/// A highlight hint's value is a short identifier — alphanumerics plus the
/// punctuation a few language ids carry (`c++`, `objective-c`). Anything else
/// means the trailing `{:…}` wasn't a hint, so the span stays literal.
fn is_hint_value(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '+' | '#'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    /// Convert, then serialize the blocks to JSON for exact comparison.
    fn blocks(markdown: &str) -> Value {
        serde_json::to_value(to_blocks(markdown).expect("converts")).unwrap()
    }

    #[test]
    fn paragraph_with_inline_marks() {
        assert_eq!(
            blocks("Hello **bold** and *italic* and `code`."),
            json!([{
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": "Hello " },
                    { "type": "text", "text": "bold", "marks": [{ "type": "bold" }] },
                    { "type": "text", "text": " and " },
                    { "type": "text", "text": "italic", "marks": [{ "type": "italic" }] },
                    { "type": "text", "text": " and " },
                    { "type": "text", "text": "code", "marks": [{ "type": "code" }] },
                    { "type": "text", "text": "." }
                ]
            }])
        );
    }

    #[test]
    fn nested_marks_stack() {
        // Marks accumulate outer-to-inner: `***x***` nests emphasis around
        // strong, so italic precedes bold. Order is cosmetic (both apply).
        assert_eq!(
            blocks("***both***"),
            json!([{
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": "both",
                      "marks": [{ "type": "italic" }, { "type": "bold" }] }
                ]
            }])
        );
    }

    #[test]
    fn heading_levels_shift_down() {
        let out = blocks("# A\n\n## B\n\n### C\n\n#### D");
        let levels: Vec<i64> = out
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n["attrs"]["level"].as_i64().unwrap())
            .collect();
        assert_eq!(levels, vec![2, 3, 4, 4]);
    }

    #[test]
    fn tight_bullet_list_wraps_items_in_paragraphs() {
        assert_eq!(
            blocks("- one\n- two"),
            json!([{
                "type": "bulletList",
                "content": [
                    { "type": "listItem", "content": [
                        { "type": "paragraph", "content": [{ "type": "text", "text": "one" }] }
                    ]},
                    { "type": "listItem", "content": [
                        { "type": "paragraph", "content": [{ "type": "text", "text": "two" }] }
                    ]}
                ]
            }])
        );
    }

    #[test]
    fn ordered_list_maps_to_ordered_list() {
        let out = blocks("1. a\n2. b");
        assert_eq!(out[0]["type"], "orderedList");
        assert_eq!(out[0]["content"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn blockquote_holds_a_paragraph() {
        assert_eq!(
            blocks("> quoted"),
            json!([{
                "type": "blockquote",
                "content": [
                    { "type": "paragraph", "content": [{ "type": "text", "text": "quoted" }] }
                ]
            }])
        );
    }

    #[test]
    fn fenced_code_block_keeps_language_and_text() {
        assert_eq!(
            blocks("```rust\nfn main() {}\n```"),
            json!([{
                "type": "codeBlock",
                "attrs": { "language": "rust" },
                "content": [{ "type": "text", "text": "fn main() {}" }]
            }])
        );
    }

    #[test]
    fn fenced_code_block_without_language_has_no_language_attr() {
        let out = blocks("```\nplain\n```");
        assert_eq!(out[0]["type"], "codeBlock");
        assert!(out[0]["attrs"].get("language").is_none());
        assert_eq!(out[0]["content"][0]["text"], "plain");
    }

    #[test]
    fn link_becomes_a_link_mark() {
        assert_eq!(
            blocks("see [the site](https://example.com)"),
            json!([{
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": "see " },
                    { "type": "text", "text": "the site",
                      "marks": [{ "type": "link", "attrs": { "href": "https://example.com" } }] }
                ]
            }])
        );
    }

    // The `{:…}` test inputs trip the nursery format-arg heuristic; they are
    // literal Markdown, not format strings.
    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn inline_code_language_suffix_sets_lang_attr() {
        // Tailing-curly-colon: `…{:lang}` highlights the span as that grammar.
        assert_eq!(
            blocks("run `let x = 1{:ts}` now"),
            json!([{
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": "run " },
                    { "type": "text", "text": "let x = 1",
                      "marks": [{ "type": "code", "attrs": { "lang": "ts" } }] },
                    { "type": "text", "text": " now" }
                ]
            }])
        );
    }

    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn inline_code_token_suffix_sets_token_attr() {
        // A leading dot (`{:.kind}`) marks an author-declared semantic token.
        assert_eq!(
            blocks("`Arc{:.type}`"),
            json!([{
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": "Arc",
                      "marks": [{ "type": "code", "attrs": { "token": "type" } }] }
                ]
            }])
        );
    }

    #[test]
    fn inline_code_without_curly_colon_stays_literal() {
        // `{foo}` lacks the `:` sigil, so the braces are part of the code text and
        // the span carries a bare `code` mark.
        assert_eq!(
            blocks("`obj{foo}`"),
            json!([{
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": "obj{foo}", "marks": [{ "type": "code" }] }
                ]
            }])
        );
    }

    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn inline_code_unknown_suffix_becomes_lang() {
        // Any shape-valid bare suffix is taken as a `lang` — the renderer degrades
        // an unknown grammar to plain — so `{:foo}` is consumed, not left literal.
        assert_eq!(
            blocks("`obj{:foo}`"),
            json!([{
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": "obj",
                      "marks": [{ "type": "code", "attrs": { "lang": "foo" } }] }
                ]
            }])
        );
    }

    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn inline_code_suffix_only_stays_literal() {
        // Nothing precedes the suffix, so there's nothing to highlight — the span
        // stays literal and an empty `text` node (which the schema rejects) is
        // never produced.
        assert_eq!(
            blocks("`{:rust}`"),
            json!([{
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": "{:rust}", "marks": [{ "type": "code" }] }
                ]
            }])
        );
    }

    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn inline_code_empty_token_kind_stays_literal() {
        // `{:.}` carries the token sigil but names no kind, so it isn't a hint.
        assert_eq!(
            blocks("`x{:.}`"),
            json!([{
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": "x{:.}", "marks": [{ "type": "code" }] }
                ]
            }])
        );
    }

    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn inline_code_multiple_suffixes_takes_last() {
        // Only the trailing `{:…}` is the hint; an earlier one stays code text.
        assert_eq!(
            blocks("`a{:b}{:rust}`"),
            json!([{
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": "a{:b}",
                      "marks": [{ "type": "code", "attrs": { "lang": "rust" } }] }
                ]
            }])
        );
    }

    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn inline_code_non_ascii_suffix_stays_literal() {
        // Hint values are ASCII identifiers; a non-ASCII spec isn't a hint, and the
        // byte-offset split must stay on a char boundary around the multibyte char.
        assert_eq!(
            blocks("`x{:rüst}`"),
            json!([{
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": "x{:rüst}", "marks": [{ "type": "code" }] }
                ]
            }])
        );
    }

    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn inline_code_markdown_bad_token_fails_validation() {
        // A Markdown-authored `{:.kind}` carries through verbatim as a `token`; an
        // unknown kind is then rejected on the write path, not silently dropped.
        let mut doc = Node::element("doc");
        doc.content = to_blocks("`x{:.bogus}`").expect("converts");
        assert_eq!(
            doc.validate_document(),
            Err(crate::pm::PmError::BadCodeToken("bogus".into()))
        );
    }

    #[test]
    fn horizontal_rule_is_a_block() {
        assert_eq!(blocks("---")[0]["type"], "horizontalRule");
    }

    #[test]
    fn hard_break_becomes_a_hard_break_node() {
        assert_eq!(
            blocks("a  \nb"),
            json!([{
                "type": "paragraph",
                "content": [
                    { "type": "text", "text": "a" },
                    { "type": "hardBreak" },
                    { "type": "text", "text": "b" }
                ]
            }])
        );
    }

    #[test]
    fn soft_break_joins_lines_with_a_space() {
        assert_eq!(
            blocks("line one\nline two"),
            json!([{
                "type": "paragraph",
                "content": [{ "type": "text", "text": "line one line two" }]
            }])
        );
    }

    #[test]
    fn multiple_top_level_blocks_in_order() {
        let out = blocks("# Title\n\nfirst\n\nsecond");
        let types: Vec<&str> = out
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n["type"].as_str().unwrap())
            .collect();
        assert_eq!(types, vec!["heading", "paragraph", "paragraph"]);
    }

    #[test]
    fn empty_input_is_empty_error() {
        assert_eq!(to_blocks(""), Err(MarkdownError::Empty));
        assert_eq!(to_blocks("   \n  "), Err(MarkdownError::Empty));
    }

    #[test]
    fn images_are_unsupported() {
        assert_eq!(
            to_blocks("![alt](photo.png)"),
            Err(MarkdownError::Unsupported("images"))
        );
    }

    #[test]
    fn raw_html_is_unsupported() {
        assert!(matches!(
            to_blocks("<div>hi</div>"),
            Err(MarkdownError::Unsupported(_))
        ));
    }
}
