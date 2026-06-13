use crate::cli::ProjectContentCommand;
use crate::cli::client::{ApiClient, check_response};
use crate::cli::output;
use crate::markdown;
use crate::pm::{Anchor, BlockPath, Doc, DocOp, Locator, Node};

type CliResult = Result<(), Box<dyn std::error::Error>>;
type CliError = Box<dyn std::error::Error>;

/// Run a `projects content` subcommand.
pub async fn run(client: ApiClient, command: ProjectContentCommand, json: bool) -> CliResult {
    match command {
        ProjectContentCommand::List { reference } => list(&client, &reference, json).await,
        ProjectContentCommand::Get { reference, locator } => {
            get(&client, &reference, locator.as_deref(), json).await
        }
        ProjectContentCommand::Insert {
            reference,
            at,
            md,
            node,
        } => insert(&client, &reference, &at, body_blocks(md, node)?, json).await,
        ProjectContentCommand::Replace {
            reference,
            locator,
            md,
            node,
        } => replace(&client, &reference, &locator, body_blocks(md, node)?, json).await,
        ProjectContentCommand::Rm { reference, locator } => {
            rm(&client, &reference, &locator, json).await
        }
        ProjectContentCommand::Move {
            reference,
            locator,
            at,
        } => move_block(&client, &reference, &locator, &at, json).await,
    }
}

/// Resolve the body of an insert/replace into one or more blocks. Exactly one
/// of `md`/`node` is present (clap enforces it): `--md` runs Markdown through
/// the converter, `--node` parses a single raw ProseMirror node.
fn body_blocks(md: Option<String>, node: Option<String>) -> Result<Vec<Node>, CliError> {
    match (md, node) {
        (Some(md), _) => markdown::to_blocks(&md).map_err(|e| format!("--md: {e}").into()),
        (None, Some(node)) => Ok(vec![parse_node(&node)?]),
        (None, None) => unreachable!("clap requires one of --md or --node"),
    }
}

async fn list(client: &ApiClient, reference: &str, json: bool) -> CliResult {
    let doc = fetch_doc(client, reference).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(doc.node())?);
    } else {
        output::print_blocks(&doc);
    }
    Ok(())
}

/// Always emits JSON — the raw inspector, distinct from `list`'s outline. With a
/// locator it prints just that block; otherwise the whole document.
async fn get(client: &ApiClient, reference: &str, locator: Option<&str>, _json: bool) -> CliResult {
    let doc = fetch_doc(client, reference).await?;
    let node = match locator {
        Some(loc) => resolve_block(&doc, loc)?,
        None => doc.node(),
    };
    println!("{}", serde_json::to_string_pretty(node)?);
    Ok(())
}

/// Resolve a block by locator: a leading `.` means a positional path
/// (`.3`, `.3.0`), anything else is a stable block id.
fn resolve_block<'a>(doc: &'a Doc, locator: &str) -> Result<&'a Node, Box<dyn std::error::Error>> {
    let found = if locator.starts_with('.') {
        let path = BlockPath::parse(locator)?;
        doc.at_path(&path)
    } else {
        doc.block(locator)
    };
    found.ok_or_else(|| format!("no block at \"{locator}\"").into())
}

async fn insert(
    client: &ApiClient,
    reference: &str,
    at: &str,
    blocks: Vec<Node>,
    json: bool,
) -> CliResult {
    let anchor = Anchor::parse(at)?;
    let count = blocks.len();
    let ops = DocOp::insert_sequence(&anchor, blocks);
    let doc = apply_ops(client, reference, ops).await?;
    report(&doc, json, &format!("Inserted {count} block(s)"))
}

async fn replace(
    client: &ApiClient,
    reference: &str,
    locator: &str,
    blocks: Vec<Node>,
    json: bool,
) -> CliResult {
    let target: Locator = locator.parse()?;
    let ops = DocOp::replace_sequence(target, blocks);
    let doc = apply_ops(client, reference, ops).await?;
    report(&doc, json, &format!("Replaced block {locator}"))
}

async fn rm(client: &ApiClient, reference: &str, locator: &str, json: bool) -> CliResult {
    let op = DocOp::Delete {
        id: locator.parse()?,
    };
    let doc = apply_ops(client, reference, vec![op]).await?;
    report(&doc, json, &format!("Removed block {locator}"))
}

async fn move_block(
    client: &ApiClient,
    reference: &str,
    locator: &str,
    at: &str,
    json: bool,
) -> CliResult {
    let op = DocOp::Move {
        id: locator.parse()?,
        anchor: Anchor::parse(at)?,
    };
    let doc = apply_ops(client, reference, vec![op]).await?;
    report(&doc, json, &format!("Moved block {locator}"))
}

fn parse_node(node_json: &str) -> Result<Node, CliError> {
    serde_json::from_str(node_json)
        .map_err(|e| format!("--node is not a valid ProseMirror node: {e}").into())
}

async fn fetch_doc(client: &ApiClient, reference: &str) -> Result<Doc, Box<dyn std::error::Error>> {
    // GET is public; the cookie (if present) lets admins read hidden projects.
    let response = client
        .get(&format!("/api/projects/{reference}/content"))
        .await?;
    let response = check_response(response).await?;
    let value: serde_json::Value = response.json().await?;
    Ok(Doc::from_stored(Some(&value)))
}

async fn apply_ops(client: &ApiClient, reference: &str, ops: Vec<DocOp>) -> Result<Doc, CliError> {
    let response = client
        .patch_auth(&format!("/api/projects/{reference}/content"), &ops)
        .await?;
    let response = check_response(response).await?;
    let value: serde_json::Value = response.json().await?;
    Ok(Doc::from_stored(Some(&value)))
}

fn report(doc: &Doc, json: bool, msg: &str) -> CliResult {
    if json {
        println!("{}", serde_json::to_string_pretty(doc.node())?);
    } else {
        output::success(msg);
        output::print_blocks(doc);
    }
    Ok(())
}
