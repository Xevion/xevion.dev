use crate::cli::ProjectContentCommand;
use crate::cli::client::{ApiClient, check_response};
use crate::cli::output;
use crate::pm::{Anchor, BlockPath, Doc, DocOp, Node};

type CliResult = Result<(), Box<dyn std::error::Error>>;

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
            node,
        } => insert(&client, &reference, &at, &node, json).await,
        ProjectContentCommand::Replace {
            reference,
            locator,
            node,
        } => replace(&client, &reference, &locator, &node, json).await,
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
    node_json: &str,
    json: bool,
) -> CliResult {
    let anchor = Anchor::parse(at)?;
    let node = parse_node(node_json)?;
    let doc = apply_op(client, reference, DocOp::Insert { anchor, node }).await?;
    report(&doc, json, "Inserted block")
}

async fn replace(
    client: &ApiClient,
    reference: &str,
    locator: &str,
    node_json: &str,
    json: bool,
) -> CliResult {
    let node = parse_node(node_json)?;
    let op = DocOp::Replace {
        id: locator.parse()?,
        node,
    };
    let doc = apply_op(client, reference, op).await?;
    report(&doc, json, &format!("Replaced block {locator}"))
}

async fn rm(client: &ApiClient, reference: &str, locator: &str, json: bool) -> CliResult {
    let op = DocOp::Delete {
        id: locator.parse()?,
    };
    let doc = apply_op(client, reference, op).await?;
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
    let doc = apply_op(client, reference, op).await?;
    report(&doc, json, &format!("Moved block {locator}"))
}

fn parse_node(node_json: &str) -> Result<Node, Box<dyn std::error::Error>> {
    serde_json::from_str(node_json)
        .map_err(|e| format!("--json is not a valid ProseMirror node: {e}").into())
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

async fn apply_op(
    client: &ApiClient,
    reference: &str,
    op: DocOp,
) -> Result<Doc, Box<dyn std::error::Error>> {
    let response = client
        .patch_auth(&format!("/api/projects/{reference}/content"), &vec![op])
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
