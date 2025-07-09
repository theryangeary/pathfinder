pub mod bluesky;

use anyhow::Result;

/// Trait for posting messages to social media platforms
#[allow(async_fn_in_trait)]
pub trait Post {
    /// Posts a message to the social media platform
    async fn post(&self, message: String) -> Result<()>;
}
