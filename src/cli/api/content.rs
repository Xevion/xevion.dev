use crate::cli::client::{ApiClient, check_response};
use crate::cli::output;
use crate::cli::{BlockContentArgs, ProjectContentCommand};
use crate::content::{Anchor, BlockInput, BlockPatch, ContentDoc, ContentOp};

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
            r#type,
            at,
            content,
        } => insert(&client, &reference, r#type, &at, &content, json).await,
        ProjectContentCommand::Set {
            reference,
            block_id,
            r#type,
            content,
        } => set(&client, &reference, &block_id, r#type, &content, json).await,
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
                .blocks
                .iter()
                .find(|b| b.id == id)
                .ok_or_else(|| format!("block \"{id}\" not found"))?;
            // A block's data is opaque, so JSON is the faithful representation.
            println!("{}", serde_json::to_string_pretty(block)?);
            Ok(())
        }
        None => print_doc(&doc, json),
    }
}

async fn insert(
    client: &ApiClient,
    reference: &str,
    r#type: Option<String>,
    at: &str,
    content: &BlockContentArgs,
    json: bool,
) -> CliResult {
    let (data, is_prose) = content.build_data()?;
    let r#type = match r#type {
        Some(t) => t,
        None if is_prose => "prose".to_string(),
        None => return Err("--type is required unless using --md/--file".into()),
    };
    let anchor = Anchor::parse(at)?;
    let op = ContentOp::Insert {
        anchor: anchor.clone(),
        block: BlockInput { r#type, data },
    };
    let doc = apply_op(client, reference, op).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&doc)?);
    } else {
        match doc.block_at_anchor(&anchor) {
            Some(b) => output::success(&format!("Inserted block {} ({})", b.id, b.r#type)),
            None => output::success("Inserted block"),
        }
        output::print_blocks(&doc);
    }
    Ok(())
}

async fn set(
    client: &ApiClient,
    reference: &str,
    block_id: &str,
    r#type: Option<String>,
    content: &BlockContentArgs,
    json: bool,
) -> CliResult {
    let (data, _) = content.build_data()?;
    let op = ContentOp::Set {
        id: block_id.to_string(),
        block: BlockPatch { r#type, data },
    };
    let doc = apply_op(client, reference, op).await?;
    report(&doc, json, &format!("Updated block {block_id}"))
}

async fn rm(client: &ApiClient, reference: &str, block_id: &str, json: bool) -> CliResult {
    let op = ContentOp::Delete {
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
    let op = ContentOp::Move {
        id: block_id.to_string(),
        anchor: Anchor::parse(at)?,
    };
    let doc = apply_op(client, reference, op).await?;
    report(&doc, json, &format!("Moved block {block_id}"))
}

async fn fetch_doc(
    client: &ApiClient,
    reference: &str,
) -> Result<ContentDoc, Box<dyn std::error::Error>> {
    // GET is public; the cookie (if present) lets admins read hidden projects.
    let response = client
        .get(&format!("/api/projects/{reference}/content"))
        .await?;
    let response = check_response(response).await?;
    Ok(response.json().await?)
}

async fn apply_op(
    client: &ApiClient,
    reference: &str,
    op: ContentOp,
) -> Result<ContentDoc, Box<dyn std::error::Error>> {
    let response = client
        .patch_auth(&format!("/api/projects/{reference}/content"), &vec![op])
        .await?;
    let response = check_response(response).await?;
    Ok(response.json().await?)
}

fn report(doc: &ContentDoc, json: bool, msg: &str) -> CliResult {
    if json {
        println!("{}", serde_json::to_string_pretty(doc)?);
    } else {
        output::success(msg);
        output::print_blocks(doc);
    }
    Ok(())
}

fn print_doc(doc: &ContentDoc, json: bool) -> CliResult {
    if json {
        println!("{}", serde_json::to_string_pretty(doc)?);
    } else {
        output::print_blocks(doc);
    }
    Ok(())
}
