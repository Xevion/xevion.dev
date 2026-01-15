use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// Site settings models

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbSiteIdentity {
    pub display_name: String,
    pub occupation: String,
    pub bio: String,
    pub site_title: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbSocialLink {
    pub id: Uuid,
    pub platform: String,
    pub label: String,
    pub value: String,
    pub icon: String,
    pub visible: bool,
    pub display_order: i32,
}

// API response types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSiteIdentity {
    pub display_name: String,
    pub occupation: String,
    pub bio: String,
    pub site_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSocialLink {
    pub id: String,
    pub platform: String,
    pub label: String,
    pub value: String,
    pub icon: String,
    pub visible: bool,
    pub display_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSiteSettings {
    pub identity: ApiSiteIdentity,
    pub social_links: Vec<ApiSocialLink>,
}

// Request types for updates
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSiteIdentityRequest {
    pub display_name: String,
    pub occupation: String,
    pub bio: String,
    pub site_title: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSocialLinkRequest {
    pub id: String,
    pub platform: String,
    pub label: String,
    pub value: String,
    pub icon: String,
    pub visible: bool,
    pub display_order: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSiteSettingsRequest {
    pub identity: UpdateSiteIdentityRequest,
    pub social_links: Vec<UpdateSocialLinkRequest>,
}

// Conversion implementations
impl DbSiteIdentity {
    pub fn to_api(&self) -> ApiSiteIdentity {
        ApiSiteIdentity {
            display_name: self.display_name.clone(),
            occupation: self.occupation.clone(),
            bio: self.bio.clone(),
            site_title: self.site_title.clone(),
        }
    }
}

impl DbSocialLink {
    pub fn to_api(&self) -> ApiSocialLink {
        ApiSocialLink {
            id: self.id.to_string(),
            platform: self.platform.clone(),
            label: self.label.clone(),
            value: self.value.clone(),
            icon: self.icon.clone(),
            visible: self.visible,
            display_order: self.display_order,
        }
    }
}

// Query functions
pub async fn get_site_settings(pool: &PgPool) -> Result<ApiSiteSettings, sqlx::Error> {
    // Get identity (single row)
    let identity = sqlx::query_as!(
        DbSiteIdentity,
        r#"
        SELECT display_name, occupation, bio, site_title
        FROM site_identity
        WHERE id = 1
        "#
    )
    .fetch_one(pool)
    .await?;

    // Get social links (ordered)
    let social_links = sqlx::query_as!(
        DbSocialLink,
        r#"
        SELECT id, platform, label, value, icon, visible, display_order
        FROM social_links
        ORDER BY display_order ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(ApiSiteSettings {
        identity: identity.to_api(),
        social_links: social_links.into_iter().map(|sl| sl.to_api()).collect(),
    })
}

pub async fn update_site_identity(
    pool: &PgPool,
    req: &UpdateSiteIdentityRequest,
) -> Result<DbSiteIdentity, sqlx::Error> {
    sqlx::query_as!(
        DbSiteIdentity,
        r#"
        UPDATE site_identity
        SET display_name = $1, occupation = $2, bio = $3, site_title = $4
        WHERE id = 1
        RETURNING display_name, occupation, bio, site_title
        "#,
        req.display_name,
        req.occupation,
        req.bio,
        req.site_title
    )
    .fetch_one(pool)
    .await
}

pub async fn update_social_link(
    pool: &PgPool,
    link_id: Uuid,
    req: &UpdateSocialLinkRequest,
) -> Result<DbSocialLink, sqlx::Error> {
    sqlx::query_as!(
        DbSocialLink,
        r#"
        UPDATE social_links
        SET platform = $2, label = $3, value = $4, icon = $5, visible = $6, display_order = $7
        WHERE id = $1
        RETURNING id, platform, label, value, icon, visible, display_order
        "#,
        link_id,
        req.platform,
        req.label,
        req.value,
        req.icon,
        req.visible,
        req.display_order
    )
    .fetch_one(pool)
    .await
}

pub async fn update_site_settings(
    pool: &PgPool,
    req: &UpdateSiteSettingsRequest,
) -> Result<ApiSiteSettings, sqlx::Error> {
    // Update identity
    let identity = update_site_identity(pool, &req.identity).await?;

    // Update each social link
    let mut updated_links = Vec::new();
    for link_req in &req.social_links {
        let link_id = Uuid::parse_str(&link_req.id).map_err(|_| {
            sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid UUID format",
            )))
        })?;
        let link = update_social_link(pool, link_id, link_req).await?;
        updated_links.push(link);
    }

    Ok(ApiSiteSettings {
        identity: identity.to_api(),
        social_links: updated_links.into_iter().map(|sl| sl.to_api()).collect(),
    })
}
