use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// ─── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionOptions {
    pub temperature: f32,
    pub max_tokens: u32,
    pub stream: bool,
}

impl Default for CompletionOptions {
    fn default() -> Self {
        Self {
            temperature: 0.2,
            max_tokens: 4096,
            stream: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub usage: Option<UsageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Provider not configured: {0}")]
    NotConfigured(String),
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Response parse error: {0}")]
    ParseError(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Provider returned error: {0}")]
    ProviderError(String),
    #[error("Stream error: {0}")]
    StreamError(String),
}

// ─── Trait ──────────────────────────────────────────────────────────────────

/// Abstraction over LLM providers (Ollama, OpenAI, Anthropic, etc.)
/// All provider implementations must be `Send + Sync` for use in async contexts.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generate a completion from a system prompt and user prompt.
    /// Returns the full response text.
    async fn complete(
        &self,
        system: &str,
        prompt: &str,
        options: &CompletionOptions,
    ) -> Result<CompletionResponse, ProviderError>;

    /// Generate a streaming completion.
    /// Returns a stream of text chunks.
    async fn complete_stream(
        &self,
        system: &str,
        prompt: &str,
        options: &CompletionOptions,
    ) -> Result<
        tokio::sync::mpsc::UnboundedReceiver<Result<String, ProviderError>>,
        ProviderError,
    >;

    /// Generate embeddings for a list of texts.
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ProviderError>;

    /// Health check — returns true if the provider is reachable and healthy.
    async fn health_check(&self) -> Result<bool, ProviderError>;

    /// Human-readable provider name (e.g. "ollama", "openai").
    fn provider_name(&self) -> &str;

    /// The model name being used (e.g. "llama3.1:8b", "gpt-4o").
    fn model_name(&self) -> &str;
}

// ─── Disabled Provider ──────────────────────────────────────────────────────

/// A no-op provider that returns errors for all operations.
/// Used when AI is disabled at the config level.
pub struct DisabledProvider;

#[async_trait]
impl LlmProvider for DisabledProvider {
    async fn complete(
        &self,
        _system: &str,
        _prompt: &str,
        _options: &CompletionOptions,
    ) -> Result<CompletionResponse, ProviderError> {
        Err(ProviderError::NotConfigured(
            "AI is disabled. Set SIP_AI_ENABLED=true and configure a provider.".into(),
        ))
    }

    async fn complete_stream(
        &self,
        _system: &str,
        _prompt: &str,
        _options: &CompletionOptions,
    ) -> Result<
        tokio::sync::mpsc::UnboundedReceiver<Result<String, ProviderError>>,
        ProviderError,
    > {
        Err(ProviderError::NotConfigured(
            "AI is disabled. Set SIP_AI_ENABLED=true and configure a provider.".into(),
        ))
    }

    async fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>, ProviderError> {
        Err(ProviderError::NotConfigured(
            "AI is disabled. Embeddings unavailable.".into(),
        ))
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        Ok(false)
    }

    fn provider_name(&self) -> &str {
        "disabled"
    }

    fn model_name(&self) -> &str {
        "none"
    }
}

// ─── Ollama Provider ────────────────────────────────────────────────────────

pub struct OllamaProvider {
    base_url: String,
    model: String,
    embedding_model: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: String, model: String, embedding_model: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
            embedding_model,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn complete(
        &self,
        system: &str,
        prompt: &str,
        options: &CompletionOptions,
    ) -> Result<CompletionResponse, ProviderError> {
        let full_prompt = if system.is_empty() {
            prompt.to_string()
        } else {
            format!("{}\n\n{}", system, prompt)
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": full_prompt,
                "stream": false,
                "options": {
                    "temperature": options.temperature,
                    "num_predict": options.max_tokens,
                }
            }))
            .send()
            .await
            .map_err(|e| ProviderError::RequestFailed(e.to_string()))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        let text = body["response"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(CompletionResponse {
            text,
            usage: None,
        })
    }

    async fn complete_stream(
        &self,
        system: &str,
        prompt: &str,
        options: &CompletionOptions,
    ) -> Result<
        tokio::sync::mpsc::UnboundedReceiver<Result<String, ProviderError>>,
        ProviderError,
    > {
        let full_prompt = if system.is_empty() {
            prompt.to_string()
        } else {
            format!("{}\n\n{}", system, prompt)
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": full_prompt,
                "stream": true,
                "options": {
                    "temperature": options.temperature,
                    "num_predict": options.max_tokens,
                }
            }))
            .send()
            .await
            .map_err(|e| ProviderError::RequestFailed(e.to_string()))?;

        let byte_stream = response.bytes_stream();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            use futures::StreamExt;

            let mut stream = byte_stream;
            let mut buffer = String::new();

            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send(Err(ProviderError::StreamError(e.to_string())));
                        return;
                    }
                };
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].trim().to_string();
                    buffer = buffer[pos + 1..].to_string();

                    if line.is_empty() {
                        continue;
                    }

                    let parsed: serde_json::Value = match serde_json::from_str(&line) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    if let Some(response_text) = parsed.get("response").and_then(|v| v.as_str()) {
                        let _ = tx.send(Ok(response_text.to_string()));
                    }

                    if parsed.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                        return;
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ProviderError> {
        let mut embeddings = Vec::with_capacity(texts.len());

        for text in texts {
            let response = self
                .client
                .post(format!("{}/api/embeddings", self.base_url))
                .json(&serde_json::json!({
                    "model": self.embedding_model,
                    "prompt": text,
                }))
                .send()
                .await
                .map_err(|e| ProviderError::RequestFailed(e.to_string()))?;

            let body: serde_json::Value = response
                .json()
                .await
                .map_err(|e| ProviderError::ParseError(e.to_string()))?;

            let embedding: Vec<f32> = body["embedding"]
                .as_array()
                .ok_or_else(|| ProviderError::ParseError("No embedding in response".into()))?
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect();

            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

// ─── OpenAI Provider ────────────────────────────────────────────────────────

pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String, base_url: Option<String>, model: String) -> Self {
        let base_url = base_url
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
            .trim_end_matches('/')
            .to_string();
        Self {
            api_key,
            base_url,
            model,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    async fn complete(
        &self,
        system: &str,
        prompt: &str,
        options: &CompletionOptions,
    ) -> Result<CompletionResponse, ProviderError> {
        let mut messages = Vec::new();
        if !system.is_empty() {
            messages.push(serde_json::json!({"role": "system", "content": system}));
        }
        messages.push(serde_json::json!({"role": "user", "content": prompt}));

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": messages,
                "temperature": options.temperature,
                "max_tokens": options.max_tokens,
                "stream": false,
            }))
            .send()
            .await
            .map_err(|e| ProviderError::RequestFailed(e.to_string()))?;

        if response.status() == 401 {
            return Err(ProviderError::AuthenticationFailed);
        }
        if response.status() == 429 {
            return Err(ProviderError::RateLimited);
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        if let Some(error) = body.get("error") {
            return Err(ProviderError::ProviderError(error.to_string()));
        }

        let text = body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = body.get("usage").map(|u| UsageInfo {
            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(CompletionResponse { text, usage })
    }

    async fn complete_stream(
        &self,
        system: &str,
        prompt: &str,
        options: &CompletionOptions,
    ) -> Result<
        tokio::sync::mpsc::UnboundedReceiver<Result<String, ProviderError>>,
        ProviderError,
    > {
        let mut messages = Vec::new();
        if !system.is_empty() {
            messages.push(serde_json::json!({"role": "system", "content": system}));
        }
        messages.push(serde_json::json!({"role": "user", "content": prompt}));

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": messages,
                "temperature": options.temperature,
                "max_tokens": options.max_tokens,
                "stream": true,
            }))
            .send()
            .await
            .map_err(|e| ProviderError::RequestFailed(e.to_string()))?;

        if response.status() == 401 {
            return Err(ProviderError::AuthenticationFailed);
        }
        if response.status() == 429 {
            return Err(ProviderError::RateLimited);
        }

        let byte_stream = response.bytes_stream();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            use futures::StreamExt;

            let mut stream = byte_stream;
            let mut buffer = String::new();

            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send(Err(ProviderError::StreamError(e.to_string())));
                        return;
                    }
                };
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].trim().to_string();
                    buffer = buffer[pos + 1..].to_string();

                    if line.is_empty() || line == "data: [DONE]" {
                        continue;
                    }

                    let data = line.strip_prefix("data: ").unwrap_or(&line);
                    let parsed: serde_json::Value = match serde_json::from_str(data) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str() {
                        let _ = tx.send(Ok(content.to_string()));
                    }

                    if parsed["choices"][0]["finish_reason"].as_str().is_some() {
                        return;
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ProviderError> {
        let input = if texts.len() == 1 {
            serde_json::json!(texts[0])
        } else {
            serde_json::json!(texts)
        };

        let response = self
            .client
            .post(format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": "text-embedding-3-small",
                "input": input,
            }))
            .send()
            .await
            .map_err(|e| ProviderError::RequestFailed(e.to_string()))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        let data = body["data"]
            .as_array()
            .ok_or_else(|| ProviderError::ParseError("No data in embedding response".into()))?;

        let embeddings: Vec<Vec<f32>> = data
            .iter()
            .map(|item| {
                item["embedding"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect()
            })
            .collect();

        Ok(embeddings)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    fn provider_name(&self) -> &str {
        "openai"
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

// ─── Factory ────────────────────────────────────────────────────────────────

use sip_config::{AiConfig, LlmProviderType};

/// Create the appropriate LLM provider based on configuration.
pub fn create_provider(config: &AiConfig) -> Box<dyn LlmProvider> {
    if !config.enabled || config.provider == LlmProviderType::Disabled {
        return Box::new(DisabledProvider);
    }

    match config.provider {
        LlmProviderType::Ollama => Box::new(OllamaProvider::new(
            config.ollama.url.clone(),
            config.ollama.model.clone(),
            config.ollama.embedding_model.clone(),
        )),
        LlmProviderType::OpenAI => {
            if let Some(ref openai) = config.openai {
                Box::new(OpenAIProvider::new(
                    openai.api_key.clone(),
                    openai.base_url.clone(),
                    openai.model.clone(),
                ))
            } else {
                tracing::warn!("OpenAI provider selected but no API key configured. Using disabled provider.");
                Box::new(DisabledProvider)
            }
        }
        LlmProviderType::Anthropic | LlmProviderType::OpenRouter
        | LlmProviderType::AzureOpenAI | LlmProviderType::Bedrock
        | LlmProviderType::CustomHttp => {
            // Placeholder for future provider implementations
            tracing::warn!(
                "Provider '{}' is not yet fully implemented. Using disabled provider.",
                config.provider.to_string()
            );
            Box::new(DisabledProvider)
        }
        LlmProviderType::Disabled => Box::new(DisabledProvider),
    }
}
