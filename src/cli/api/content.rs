use crate::cli::ProjectContentCommand;
use crate::cli::client::{ApiClient, check_response};
use crate::cli::output;
use crate::pm::{Anchor, Doc, DocOp, Node};

type CliResult = Result<(), Box<dyn std::error::Error>>;

/// Run a `projects content` subcommand.
pub async fn run(client: ApiClient, command: ProjectContentCommand, json: bool) -> CliResult {
    match command {
        ProjectContentCommand::List { reference } => list(&client, &reference, json).await,
        ProjectContentCommand::Get {
            reference,
            block_id,
        } => get(&client, &reference, block_id.as_deref(), json).await,
        ProjectContentCommand::Insert {
            reference,
            at,
            json: node,
        } => insert(&client, &reference, &at, &node, json).await,
        ProjectContentCommand::Replace {
            reference,
            block_id,
            json: node,
        } => replace(&client, &reference, &block_id, &node, json).await,
        ProjectContentCommand::Rm {
            reference,
            block_id,
        } => rm(&client, &reference, &block_id, json).await,
        ProjectContentCommand::Move {
            reference,
            block_id,
            at,
        } => move_block(&client, &reference, &block_id, &at, json).await,
    }
}

async fn list(client: &ApiClient, reference: &str, json: bool) -> CliResult {
    let doc = fetch_doc(client, reference).await?;
    print_doc(&doc, json)
}

async fn get(client: &ApiClient, reference: &str, block_id: Option<&str>, json: bool) -> CliResult {
    let doc = fetch_doc(client, reference).await?;
    match block_id {
        Some(id) => {
            let block = doc
                .block(id)
                .ok_or_else(|| format!("block \"{id}\" not found"))?;
            println!("{}", serde_json::to_string_pretty(block)?);
            Ok(())
        }
        None => print_doc(&doc, json),
    }
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
    block_id: &str,
    node_json: &str,
    json: bool,
) -> CliResult {
    let node = parse_node(node_json)?;
    let op = DocOp::Replace {
        id: block_id.to_string(),
        node,
    };
    let doc = apply_op(client, reference, op).await?;
    report(&doc, json, &format!("Replaced block {block_id}"))
}

async fn rm(client: &ApiClient, reference: &str, block_id: &str, json: bool) -> CliResult {
    let op = DocOp::Delete {
        id: block_id.to_string(),
    };
    let doc = apply_op(client, reference, op).await?;
    report(&doc, json, &format!("Removed block {block_id}"))
}

async fn move_block(
    client: &ApiClient,
    reference: &str,
    block_id: &str,
    at: &str,
    json: bool,
) -> CliResult {
    let op = DocOp::Move {
        id: block_id.to_string(),
        anchor: Anchor::parse(at)?,
    };
    let doc = apply_op(client, reference, op).await?;
    report(&doc, json, &format!("Moved block {block_id}"))
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

fn print_doc(doc: &Doc, json: bool) -> CliResult {
    if json {
        println!("{}", serde_json::to_string_pretty(doc.node())?);
    } else {
        output::print_blocks(doc);
    }
    Ok(())
}
