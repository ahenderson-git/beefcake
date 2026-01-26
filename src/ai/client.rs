//! AI Client module for integrating OpenAI API into Beefcake.
//!
//! This module provides functionality to interact with OpenAI's API for
//! providing in-application AI support, data analysis guidance, and
//! statistical interpretation.

use anyhow::{Context as _, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
};

pub use crate::config::AIConfig;

/// AI Assistant client for interacting with OpenAI API
pub struct AIAssistant {
    client: Client<OpenAIConfig>,
    config: AIConfig,
}

impl AIAssistant {
    /// Create a new AI Assistant with the provided API key and configuration
    pub fn new(api_key: String, config: AIConfig) -> Result<Self> {
        let openai_config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(openai_config);

        Ok(Self { client, config })
    }

    /// Send a query to the AI assistant with optional context
    ///
    /// # Arguments
    /// * `query` - The user's query
    /// * `context` - Optional context about the current dataset or analysis
    ///
    /// # Returns
    /// The AI's response as a string
    pub async fn send_query(&self, query: &str, context: Option<&str>) -> Result<String> {
        let mut messages: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(Self::system_prompt())
                .build()
                .context("Failed to build system message")?
                .into(),
        ];

        // Add context if provided
        if let Some(ctx) = context {
            messages.push(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(format!("Dataset Context:\n{ctx}"))
                    .build()
                    .context("Failed to build context message")?
                    .into(),
            );
        }

        // Add user query
        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(query)
                .build()
                .context("Failed to build user message")?
                .into(),
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.model)
            .messages(messages)
            .temperature(self.config.temperature)
            .max_tokens(self.config.max_tokens)
            .build()
            .context("Failed to build chat completion request")?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| anyhow::anyhow!("OpenAI API error: {e}"))?;

        response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response content received"))
    }

    /// Test the connection to OpenAI API
    pub async fn test_connection(&self) -> Result<()> {
        // Use a minimal query to test connection
        let response = self.send_query("Hi", None).await?;
        if response.is_empty() {
            anyhow::bail!("Received empty response from OpenAI API");
        }
        Ok(())
    }

    /// Get the system prompt for the AI assistant
    fn system_prompt() -> String {
        r#"You are an AI assistant integrated into Beefcake, a data analysis application.
Your role is to help users understand their data, suggest transformations, interpret statistical results,
and provide guidance on data cleaning and analysis workflows.

When answering:
- Be concise and practical
- Focus on data analysis and Beefcake-specific features
- Suggest specific column operations, filters, or transformations when relevant
- Explain statistical concepts in accessible terms
- If you reference code or formulas, use markdown formatting

You have access to context about the user's current dataset including column names, types, and statistics."#.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_assistant_system_prompt() {
        let prompt = AIAssistant::system_prompt();

        assert!(prompt.contains("Beefcake"));
        assert!(prompt.contains("data analysis"));
    }
}
