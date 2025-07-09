use anyhow::{anyhow, Result};
use bsky_sdk::BskyAgent;
use atrium_api::types::string::Datetime;

use super::Post;

/// BlueSky poster implementation using the official bsky-sdk
pub struct BlueSkyPoster {
    /// BlueSky agent
    agent: Option<BskyAgent>,
    /// User handle
    handle: String,
    /// User password (app password)
    password: Option<String>,
}

impl BlueSkyPoster {
    /// Creates a new BlueSky poster
    pub fn new(handle: String) -> Self {
        Self {
            agent: None,
            handle,
            password: None,
        }
    }

    /// Sets the app password for authentication
    pub fn with_password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    /// Authenticates with BlueSky using the provided credentials
    pub async fn authenticate(&mut self) -> Result<()> {
        let password = self
            .password
            .as_ref()
            .ok_or_else(|| anyhow!("No password provided"))?;

        let agent = BskyAgent::builder().build().await?;
        
        agent
            .login(&self.handle, password)
            .await
            .map_err(|e| anyhow!("Authentication failed: {}", e))?;

        self.agent = Some(agent);
        Ok(())
    }

    /// Posts a message to BlueSky
    async fn create_post(&self, text: String) -> Result<()> {
        let agent = self
            .agent
            .as_ref()
            .ok_or_else(|| anyhow!("Not authenticated. Call authenticate() first"))?;

        agent
            .create_record(
                atrium_api::app::bsky::feed::post::RecordData {
                    text,
                    created_at: Datetime::now(),
                    embed: None,
                    entities: None,
                    facets: None,
                    labels: None,
                    langs: None,
                    reply: None,
                    tags: None,
                }
            )
            .await
            .map_err(|e| anyhow!("Failed to create post: {}", e))?;

        Ok(())
    }
}

impl Post for BlueSkyPoster {
    async fn post(&self, message: String) -> Result<()> {
        self.create_post(message).await
    }
}