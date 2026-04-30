use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

// ─── Environment ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Environment {
    Local,
    Development,
    Test,
    Staging,
    Production,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Local => write!(f, "local"),
            Environment::Development => write!(f, "development"),
            Environment::Test => write!(f, "test"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

impl Environment {
    pub fn is_development_like(&self) -> bool {
        matches!(self, Environment::Local | Environment::Development | Environment::Test)
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    pub fn is_strict(&self) -> bool {
        matches!(self, Environment::Staging | Environment::Production)
    }
}

// ─── LLM Provider ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LlmProviderType {
    Ollama,
    OpenAI,
    Anthropic,
    OpenRouter,
    AzureOpenAI,
    Bedrock,
    CustomHttp,
    Disabled,
}

impl std::fmt::Display for LlmProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmProviderType::Ollama => write!(f, "ollama"),
            LlmProviderType::OpenAI => write!(f, "openai"),
            LlmProviderType::Anthropic => write!(f, "anthropic"),
            LlmProviderType::OpenRouter => write!(f, "openrouter"),
            LlmProviderType::AzureOpenAI => write!(f, "azure_openai"),
            LlmProviderType::Bedrock => write!(f, "bedrock"),
            LlmProviderType::CustomHttp => write!(f, "custom_http"),
            LlmProviderType::Disabled => write!(f, "disabled"),
        }
    }
}

// ─── Sub-configs ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_host")]
    pub host: String,
    #[serde(default = "default_server_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_db_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_db_connect_timeout_seconds")]
    pub connect_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStorageConfig {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    #[serde(default)]
    pub use_ssl: bool,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiration_seconds")]
    pub jwt_expiration_seconds: u64,
    #[serde(default = "default_refresh_expiration_seconds")]
    pub refresh_expiration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    #[serde(default = "default_ollama_url")]
    pub url: String,
    #[serde(default = "default_ollama_model")]
    pub model: String,
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default = "default_openai_model")]
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default = "default_anthropic_model")]
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    pub api_key: String,
    #[serde(default = "default_openrouter_base_url")]
    pub base_url: String,
    #[serde(default = "default_openrouter_model")]
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureOpenAIConfig {
    pub api_key: String,
    pub endpoint: String,
    pub deployment: String,
    #[serde(default = "default_azure_api_version")]
    pub api_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockConfig {
    #[serde(default = "default_bedrock_region")]
    pub region: String,
    pub model_id: String,
    /// Uses standard AWS credential chain (env vars, ~/.aws/credentials, IAM role)
    #[serde(default)]
    pub use_credential_chain: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomHttpConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    #[serde(default)]
    pub auth_header_name: Option<String>,
    #[serde(default)]
    pub auth_header_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    #[serde(default = "default_ai_enabled")]
    pub enabled: bool,
    #[serde(default = "default_llm_provider")]
    pub provider: LlmProviderType,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_streaming_enabled")]
    pub streaming_enabled: bool,
    #[serde(default)]
    pub request_logging_enabled: bool,
    #[serde(default = "default_temperature")]
    pub default_temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub default_max_tokens: u32,
    #[serde(default)]
    pub ollama: OllamaConfig,
    #[serde(default)]
    pub openai: Option<OpenAIConfig>,
    #[serde(default)]
    pub anthropic: Option<AnthropicConfig>,
    #[serde(default)]
    pub openrouter: Option<OpenRouterConfig>,
    #[serde(default)]
    pub azure_openai: Option<AzureOpenAIConfig>,
    #[serde(default)]
    pub bedrock: Option<BedrockConfig>,
    #[serde(default)]
    pub custom_http: Option<CustomHttpConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    #[serde(default = "default_true")]
    pub ai_chat_enabled: bool,
    #[serde(default = "default_true")]
    pub rag_enabled: bool,
    #[serde(default = "default_true")]
    pub graph_enabled: bool,
    #[serde(default)]
    pub plugin_system_enabled: bool,
    #[serde(default = "default_true")]
    pub document_ingestion_enabled: bool,
    #[serde(default = "default_true")]
    pub semantic_search_enabled: bool,
    #[serde(default = "default_true")]
    pub dispatch_enabled: bool,
    #[serde(default = "default_true")]
    pub parts_inventory_enabled: bool,
    #[serde(default)]
    pub disposables_tracking_enabled: bool,
    #[serde(default)]
    pub billing_enabled: bool,
    #[serde(default = "default_true")]
    pub api_docs_enabled: bool,
    #[serde(default)]
    pub marketplace_enabled: bool,
    #[serde(default)]
    pub experimental_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_cors_allowed_origins")]
    pub cors_allowed_origins: Vec<String>,
    #[serde(default)]
    pub require_https: bool,
    #[serde(default = "default_rate_limit_rpm")]
    pub rate_limit_rpm: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub otlp_endpoint: Option<String>,
    #[serde(default)]
    pub metrics_enabled: bool,
}

// ─── AppConfig (root) ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_env")]
    pub env: Environment,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub redis: Option<RedisConfig>,
    #[serde(default)]
    pub object_storage: Option<ObjectStorageConfig>,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub features: FeatureConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
}

// ─── Default functions ──────────────────────────────────────────────────────

fn default_env() -> Environment {
    Environment::Development
}
fn default_server_host() -> String {
    "0.0.0.0".to_string()
}
fn default_server_port() -> u16 {
    8000
}
fn default_db_max_connections() -> u32 {
    10
}
fn default_db_connect_timeout_seconds() -> u64 {
    5
}
fn default_jwt_expiration_seconds() -> u64 {
    900
}
fn default_refresh_expiration_seconds() -> u64 {
    604800
}
fn default_true() -> bool {
    true
}
fn default_ai_enabled() -> bool {
    false
}
fn default_llm_provider() -> LlmProviderType {
    LlmProviderType::Disabled
}
fn default_timeout_seconds() -> u64 {
    60
}
fn default_max_retries() -> u32 {
    2
}
fn default_streaming_enabled() -> bool {
    true
}
fn default_temperature() -> f32 {
    0.2
}
fn default_max_tokens() -> u32 {
    4096
}
fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}
fn default_ollama_model() -> String {
    "llama3.1:8b".to_string()
}
fn default_embedding_model() -> String {
    "nomic-embed-text".to_string()
}
fn default_openai_model() -> String {
    "gpt-4o".to_string()
}
fn default_anthropic_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}
fn default_openrouter_base_url() -> String {
    "https://openrouter.ai/api/v1".to_string()
}
fn default_openrouter_model() -> String {
    "openai/gpt-4o".to_string()
}
fn default_azure_api_version() -> String {
    "2024-08-01-preview".to_string()
}
fn default_bedrock_region() -> String {
    "us-east-1".to_string()
}
fn default_cors_allowed_origins() -> Vec<String> {
    vec!["http://localhost:3000".to_string()]
}
fn default_rate_limit_rpm() -> u32 {
    300
}
fn default_log_level() -> String {
    "info".to_string()
}

// ─── Config loading ─────────────────────────────────────────────────────────

/// Load config with precedence: hardcoded defaults → Sip.toml → SIP_ env vars
pub fn load_config() -> Result<AppConfig, figment::Error> {
    Figment::new()
        .merge(Toml::file("Sip.toml"))
        .merge(Env::prefixed("SIP_"))
        .extract()
}

/// Load config from a specific TOML file path (useful for testing)
pub fn load_config_from(path: &str) -> Result<AppConfig, figment::Error> {
    Figment::new()
        .merge(Toml::file(path))
        .merge(Env::prefixed("SIP_"))
        .extract()
}

/// Load config for the SIP configuration sub-section from env vars only
/// Used by Docker where structured TOML is inconvenient; we map flat SIP_ vars
/// into the nested config tree via figment's join/downcast.
///
/// This is a convenience: in Docker, users set SIP_DATABASE_URL, SIP_AI_ENABLED,
/// SIP_LLM_PROVIDER, etc. directly as env vars.
pub fn load_config_docker() -> Result<AppConfig, figment::Error> {
    // In Docker, we rely entirely on env vars (no Sip.toml present)
    Figment::new()
        .merge(Env::prefixed("SIP_"))
        .extract()
}

// ─── Backward-compatible flat env mapping ───────────────────────────────────
//
// The new config is nested (ai.ollama.url, ai.ollama.model, etc.)
// but existing users have SIP_OLLAMA_URL and SIP_OLLAMA_MODEL.
//
// Figment's Env::prefixed("SIP_") maps SIP_OLLAMA_URL → ollama_url (flat).
// For nested access, it also tries SIP_AI_OLLAMA_URL → ai.ollama.url.
//
// To support both, we provide a flattening adapter that maps old flat
// env vars to their new nested equivalents.

/// Apply backward-compatible aliases for old flat env var names.
/// Call this after load_config() to ensure old SIP_OLLAMA_URL works.
pub fn apply_backward_compat(config: &mut AppConfig) {
    // Read flat old-style env vars directly
    if let Ok(val) = std::env::var("SIP_OLLAMA_URL") {
        config.ai.ollama.url = val;
    }
    if let Ok(val) = std::env::var("SIP_OLLAMA_MODEL") {
        config.ai.ollama.model = val;
    }
}

// ─── Validation ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ConfigValidation {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ConfigValidation {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl std::fmt::Display for ConfigValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in &self.errors {
            writeln!(f, "ERROR: {}", e)?;
        }
        for w in &self.warnings {
            writeln!(f, "WARNING: {}", w)?;
        }
        Ok(())
    }
}

/// Validate the configuration, returning actionable errors and warnings.
pub fn validate_config(config: &AppConfig) -> ConfigValidation {
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let is_strict = config.env.is_strict();
    let is_dev = config.env.is_development_like();

    // ── Database ──
    if config.database.is_none() || config.database.as_ref().map_or(true, |d| d.url.is_empty()) {
        errors.push("SIP_DATABASE_URL is required".into());
    }

    // ── Redis ──
    if config.redis.is_none() && is_strict {
        warnings.push("Redis not configured. Caching and rate limiting will be disabled.".into());
    }

    // ── Object Storage ──
    if config.object_storage.is_some() {
        let os = config.object_storage.as_ref().unwrap();
        if os.enabled {
            if os.endpoint.is_empty() {
                errors.push("Object storage is enabled but endpoint is empty".into());
            }
            if os.bucket.is_empty() {
                errors.push("Object storage is enabled but bucket is empty".into());
            }
        }
    }

    // ── Auth ──
    if let Some(ref auth) = config.auth {
        if auth.jwt_secret.len() < 32 && is_strict {
            errors.push("JWT secret must be at least 32 characters in staging/production".into());
        }
        if auth.jwt_secret.len() < 32 && !is_strict {
            warnings.push("JWT secret should be at least 32 characters for security".into());
        }
        if auth.jwt_secret == "change-me-in-production-jwt-secret-min-32-chars" && is_strict {
            errors.push("JWT secret is still the default placeholder. Set a real secret.".into());
        }
    }

    // ── AI ──
    if config.ai.enabled {
        match config.ai.provider {
            LlmProviderType::Disabled => {
                errors.push("AI is enabled but LLM provider is 'disabled'".into());
            }
            LlmProviderType::Ollama => {
                if config.ai.ollama.url.is_empty() {
                    errors.push("Ollama provider selected but SIP_OLLAMA_URL is empty".into());
                }
            }
            LlmProviderType::OpenAI => {
                match &config.ai.openai {
                    Some(oc) if oc.api_key.is_empty() => {
                        errors.push("OpenAI provider selected but SIP_OPENAI_API_KEY is empty".into());
                    }
                    None => {
                        errors.push("OpenAI provider selected but no OpenAI config provided".into());
                    }
                    _ => {}
                }
            }
            LlmProviderType::Anthropic => {
                match &config.ai.anthropic {
                    Some(ac) if ac.api_key.is_empty() => {
                        errors.push("Anthropic provider selected but SIP_ANTHROPIC_API_KEY is empty".into());
                    }
                    None => {
                        errors.push("Anthropic provider selected but no Anthropic config provided".into());
                    }
                    _ => {}
                }
            }
            LlmProviderType::OpenRouter => {
                match &config.ai.openrouter {
                    Some(oc) if oc.api_key.is_empty() => {
                        errors.push("OpenRouter provider selected but SIP_OPENROUTER_API_KEY is empty".into());
                    }
                    None => {
                        errors.push("OpenRouter provider selected but no OpenRouter config provided".into());
                    }
                    _ => {}
                }
            }
            LlmProviderType::AzureOpenAI => {
                match &config.ai.azure_openai {
                    Some(ac) => {
                        if ac.api_key.is_empty() {
                            errors.push("Azure OpenAI provider selected but SIP_AZURE_OPENAI_API_KEY is empty".into());
                        }
                        if ac.endpoint.is_empty() {
                            errors.push("Azure OpenAI provider selected but SIP_AZURE_OPENAI_ENDPOINT is empty".into());
                        }
                        if ac.deployment.is_empty() {
                            errors.push("Azure OpenAI provider selected but SIP_AZURE_OPENAI_DEPLOYMENT is empty".into());
                        }
                    }
                    None => {
                        errors.push("Azure OpenAI provider selected but no Azure config provided".into());
                    }
                }
            }
            LlmProviderType::Bedrock => {
                match &config.ai.bedrock {
                    Some(bc) if bc.model_id.is_empty() => {
                        errors.push("Bedrock provider selected but SIP_BEDROCK_MODEL_ID is empty".into());
                    }
                    None => {
                        errors.push("Bedrock provider selected but no Bedrock config provided".into());
                    }
                    _ => {}
                }
            }
            LlmProviderType::CustomHttp => {
                match &config.ai.custom_http {
                    Some(cc) if cc.base_url.is_empty() => {
                        errors.push("Custom HTTP provider selected but SIP_CUSTOM_LLM_BASE_URL is empty".into());
                    }
                    None => {
                        errors.push("Custom HTTP provider selected but no custom HTTP config provided".into());
                    }
                    _ => {}
                }
            }
        }
    } else {
        // AI disabled — ensure provider is set to disabled (or warn)
        if config.ai.provider != LlmProviderType::Disabled && is_strict {
            warnings.push("AI is disabled but LLM provider is not set to 'disabled'".into());
        }
    }

    // ── Feature dependencies ──
    if config.features.rag_enabled && !config.ai.enabled {
        if is_strict {
            errors.push("RAG feature requires AI to be enabled".into());
        } else {
            warnings.push("RAG feature enabled but AI is disabled — RAG will not function".into());
        }
    }
    if config.features.semantic_search_enabled && !config.ai.enabled {
        if is_strict {
            warnings.push("Semantic search requires AI for embeddings. Enable AI or disable semantic search.".into());
        }
    }
    if config.features.document_ingestion_enabled && config.object_storage.is_none() {
        if is_strict {
            errors.push("Document ingestion requires object storage configuration".into());
        } else {
            warnings.push("Document ingestion enabled but no object storage configured".into());
        }
    }
    if config.features.marketplace_enabled && !config.features.plugin_system_enabled {
        errors.push("Marketplace requires plugin system to be enabled".into());
    }
    if config.features.experimental_enabled && is_strict {
        warnings.push("Experimental features are enabled in a strict environment".into());
    }

    // ── Production validation ──
    if config.env.is_production() {
        // CORS must be explicitly configured in production
        if config.security.cors_allowed_origins == default_cors_allowed_origins() {
            warnings.push("Production uses default CORS origins (localhost:3000). Configure SIP_CORS_ALLOWED_ORIGINS for your domain.".into());
        }
        // Require HTTPS
        if !config.security.require_https {
            warnings.push("SIP_REQUIRE_HTTPS is not enabled in production. Consider enabling it.".into());
        }
        // Rate limiting should be reasonable
        if config.security.rate_limit_rpm >= 1000 {
            warnings.push("Rate limit is very high for production. Consider lowering SIP_RATE_LIMIT_RPM.".into());
        }
    }

    // ── Dev mode relaxations ──
    if is_dev {
        // These are just informational for dev mode
        if config.ai.provider == LlmProviderType::Ollama && config.ai.ollama.url.is_empty() {
            warnings.push("Ollama URL not set; defaulting to http://localhost:11434".into());
        }
    }

    ConfigValidation { errors, warnings }
}

// ─── Redacted display ───────────────────────────────────────────────────────

/// Returns a redacted representation of the config safe for logging/diagnostics.
/// API keys, secrets, and tokens are replaced with "[REDACTED]".
pub fn redacted_config(config: &AppConfig) -> serde_json::Value {
    let ai_redacted = redact_ai_config(&config.ai);
    let auth_redacted = config.auth.as_ref().map(|a| {
        serde_json::json!({
            "jwt_secret": "[REDACTED]",
            "jwt_expiration_seconds": a.jwt_expiration_seconds,
            "refresh_expiration_seconds": a.refresh_expiration_seconds,
        })
    });
    let storage_redacted = config.object_storage.as_ref().map(|os| {
        serde_json::json!({
            "endpoint": os.endpoint,
            "bucket": os.bucket,
            "access_key": "[REDACTED]",
            "secret_key": "[REDACTED]",
            "use_ssl": os.use_ssl,
            "enabled": os.enabled,
        })
    });

    serde_json::json!({
        "env": config.env.to_string(),
        "server": {
            "host": config.server.host,
            "port": config.server.port,
        },
        "database": config.database.as_ref().map(|d| {
            serde_json::json!({
                "url": redact_url_password(&d.url),
                "max_connections": d.max_connections,
            })
        }),
        "redis": config.redis.as_ref().map(|r| {
            serde_json::json!({
                "url": redact_url_password(&r.url),
                "enabled": r.enabled,
            })
        }),
        "object_storage": storage_redacted,
        "auth": auth_redacted,
        "ai": ai_redacted,
        "features": serde_json::to_value(&config.features).unwrap_or_default(),
        "security": {
            "cors_allowed_origins": &config.security.cors_allowed_origins,
            "require_https": config.security.require_https,
            "rate_limit_rpm": config.security.rate_limit_rpm,
        },
        "observability": {
            "log_level": config.observability.log_level,
            "otlp_endpoint": config.observability.otlp_endpoint,
            "metrics_enabled": config.observability.metrics_enabled,
        },
    })
}

fn redact_url_password(url: &str) -> String {
    // Replace password in postgres://user:pass@host/db
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let prefix = &url[..colon_pos + 1];
            let suffix = &url[at_pos..];
            return format!("{}[REDACTED]{}", prefix, suffix);
        }
    }
    url.to_string()
}

fn redact_ai_config(ai: &AiConfig) -> serde_json::Value {
    let ollama = serde_json::json!({
        "url": ai.ollama.url,
        "model": ai.ollama.model,
        "embedding_model": ai.ollama.embedding_model,
    });

    let openai = ai.openai.as_ref().map(|o| {
        serde_json::json!({
            "api_key": redact_secret(&o.api_key),
            "base_url": o.base_url,
            "model": o.model,
        })
    });

    let anthropic = ai.anthropic.as_ref().map(|a| {
        serde_json::json!({
            "api_key": redact_secret(&a.api_key),
            "base_url": a.base_url,
            "model": a.model,
        })
    });

    let openrouter = ai.openrouter.as_ref().map(|o| {
        serde_json::json!({
            "api_key": redact_secret(&o.api_key),
            "base_url": o.base_url,
            "model": o.model,
        })
    });

    let azure = ai.azure_openai.as_ref().map(|a| {
        serde_json::json!({
            "api_key": redact_secret(&a.api_key),
            "endpoint": a.endpoint,
            "deployment": a.deployment,
            "api_version": a.api_version,
        })
    });

    let bedrock = ai.bedrock.as_ref().map(|b| {
        serde_json::json!({
            "region": b.region,
            "model_id": b.model_id,
            "use_credential_chain": b.use_credential_chain,
        })
    });

    let custom = ai.custom_http.as_ref().map(|c| {
        serde_json::json!({
            "base_url": c.base_url,
            "api_key": c.api_key.as_ref().map(|k| redact_secret(k)).unwrap_or_else(|| "none".to_string()),
            "model": c.model,
            "auth_header_name": c.auth_header_name,
            "auth_header_prefix": c.auth_header_prefix,
        })
    });

    serde_json::json!({
        "enabled": ai.enabled,
        "provider": ai.provider.to_string(),
        "timeout_seconds": ai.timeout_seconds,
        "max_retries": ai.max_retries,
        "streaming_enabled": ai.streaming_enabled,
        "request_logging_enabled": ai.request_logging_enabled,
        "default_temperature": ai.default_temperature,
        "default_max_tokens": ai.default_max_tokens,
        "ollama": ollama,
        "openai": openai,
        "anthropic": anthropic,
        "openrouter": openrouter,
        "azure_openai": azure,
        "bedrock": bedrock,
        "custom_http": custom,
    })
}

fn redact_secret(s: &str) -> String {
    if s.is_empty() {
        return "(empty)".to_string();
    }
    if s.len() <= 8 {
        return "[REDACTED]".to_string();
    }
    format!("{}...{}", &s[..4], &s[s.len() - 4..])
}

// ─── Config inspection ───────────────────────────────────────────────────────

/// Summary of effective configuration for display to operators.
#[derive(Debug, Serialize)]
pub struct ConfigStatus {
    pub env: String,
    pub ai_enabled: bool,
    pub llm_provider: String,
    pub model: Option<String>,
    pub features_enabled: Vec<String>,
    pub features_disabled: Vec<String>,
    pub validation_errors: Vec<String>,
    pub validation_warnings: Vec<String>,
    pub is_valid: bool,
    pub redacted_config: serde_json::Value,
}

pub fn config_status(config: &AppConfig) -> ConfigStatus {
    let validation = validate_config(config);
    let model_name = match config.ai.provider {
        LlmProviderType::Ollama => Some(config.ai.ollama.model.clone()),
        LlmProviderType::OpenAI => config.ai.openai.as_ref().map(|o| o.model.clone()),
        LlmProviderType::Anthropic => config.ai.anthropic.as_ref().map(|a| a.model.clone()),
        LlmProviderType::OpenRouter => config.ai.openrouter.as_ref().map(|o| o.model.clone()),
        LlmProviderType::AzureOpenAI => config.ai.azure_openai.as_ref().map(|a| a.deployment.clone()),
        LlmProviderType::Bedrock => config.ai.bedrock.as_ref().map(|b| b.model_id.clone()),
        LlmProviderType::CustomHttp => config.ai.custom_http.as_ref().map(|c| c.model.clone()),
        LlmProviderType::Disabled => None,
    };

    let mut features_enabled = Vec::new();
    let mut features_disabled = Vec::new();

    macro_rules! check_feature {
        ($cond:expr, $name:expr) => {
            if $cond {
                features_enabled.push($name.to_string());
            } else {
                features_disabled.push($name.to_string());
            }
        };
    }

    check_feature!(config.features.ai_chat_enabled, "ai_chat");
    check_feature!(config.features.rag_enabled, "rag");
    check_feature!(config.features.graph_enabled, "graph");
    check_feature!(config.features.plugin_system_enabled, "plugin_system");
    check_feature!(config.features.document_ingestion_enabled, "document_ingestion");
    check_feature!(config.features.semantic_search_enabled, "semantic_search");
    check_feature!(config.features.dispatch_enabled, "dispatch");
    check_feature!(config.features.parts_inventory_enabled, "parts_inventory");
    check_feature!(config.features.disposables_tracking_enabled, "disposables_tracking");
    check_feature!(config.features.billing_enabled, "billing");
    check_feature!(config.features.api_docs_enabled, "api_docs");
    check_feature!(config.features.marketplace_enabled, "marketplace");
    check_feature!(config.features.experimental_enabled, "experimental");

    let is_valid = validation.is_valid();
    ConfigStatus {
        env: config.env.to_string(),
        ai_enabled: config.ai.enabled,
        llm_provider: config.ai.provider.to_string(),
        model: model_name,
        features_enabled,
        features_disabled,
        validation_errors: validation.errors,
        validation_warnings: validation.warnings,
        is_valid,
        redacted_config: redacted_config(config),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_loaded() {
        let config = Figment::new().extract::<AppConfig>();
        assert!(config.is_ok(), "Should load with defaults: {:?}", config.err());
        let c = config.unwrap();
        assert_eq!(c.env, Environment::Development);
        assert_eq!(c.server.port, 8000);
        assert!(c.database.is_none(), "Database should be None when not configured");
    }

    #[test]
    fn test_environment_display() {
        assert_eq!(Environment::Local.to_string(), "local");
        assert_eq!(Environment::Production.to_string(), "production");
    }

    #[test]
    fn test_environment_is_helpers() {
        assert!(Environment::Development.is_development_like());
        assert!(!Environment::Production.is_development_like());
        assert!(Environment::Staging.is_strict());
        assert!(!Environment::Development.is_strict());
        assert!(Environment::Production.is_production());
    }

    #[test]
    fn test_llm_provider_display() {
        assert_eq!(LlmProviderType::Ollama.to_string(), "ollama");
        assert_eq!(LlmProviderType::Disabled.to_string(), "disabled");
    }

    #[test]
    fn test_secret_redaction() {
        assert_eq!(redact_secret(""), "(empty)");
        assert_eq!(redact_secret("ab"), "[REDACTED]");
        assert_eq!(redact_secret("sk-1234567890abcdef"), "sk-1...cdef");
        assert_eq!(redact_secret("short"), "[REDACTED]");
    }

    #[test]
    fn test_url_password_redaction() {
        let url = "postgres://user:secretpass@localhost:5432/db";
        let redacted = redact_url_password(url);
        assert!(redacted.contains("user:"));
        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("secretpass"));
        assert!(redacted.contains("@localhost"));
    }

    #[test]
    fn test_redacted_config_no_secrets() {
        let config = AppConfig {
            env: Environment::Development,
            server: ServerConfig {
                host: "0.0.0.0".into(),
                port: 8000,
            },
            database: Some(DatabaseConfig {
                url: "postgres://sip:pass@localhost:5432/sip".into(),
                max_connections: 10,
                connect_timeout_seconds: 5,
            }),
            redis: Some(RedisConfig {
                url: "redis://localhost:6379".into(),
                enabled: true,
            }),
            object_storage: Some(ObjectStorageConfig {
                endpoint: "http://localhost:9000".into(),
                bucket: "sip-docs".into(),
                access_key: "minioadmin".into(),
                secret_key: "minioadmin".into(),
                use_ssl: false,
                enabled: true,
            }),
            auth: Some(AuthConfig {
                jwt_secret: "super-secret-key-that-is-long-enough".into(),
                jwt_expiration_seconds: 900,
                refresh_expiration_seconds: 604800,
            }),
            ai: AiConfig {
                enabled: true,
                provider: LlmProviderType::OpenAI,
                timeout_seconds: 60,
                max_retries: 2,
                streaming_enabled: true,
                request_logging_enabled: false,
                default_temperature: 0.2,
                default_max_tokens: 4096,
                ollama: OllamaConfig {
                    url: "http://localhost:11434".into(),
                    model: "llama3.1:8b".into(),
                    embedding_model: "nomic-embed-text".into(),
                },
                openai: Some(OpenAIConfig {
                    api_key: "sk-proj-abcdefghijklmnop".into(),
                    base_url: None,
                    model: "gpt-4o".into(),
                }),
                anthropic: None,
                openrouter: None,
                azure_openai: None,
                bedrock: None,
                custom_http: None,
            },
            features: FeatureConfig {
                ai_chat_enabled: true,
                rag_enabled: true,
                graph_enabled: true,
                plugin_system_enabled: false,
                document_ingestion_enabled: true,
                semantic_search_enabled: true,
                dispatch_enabled: true,
                parts_inventory_enabled: true,
                disposables_tracking_enabled: false,
                billing_enabled: false,
                api_docs_enabled: true,
                marketplace_enabled: false,
                experimental_enabled: false,
            },
            security: SecurityConfig {
                cors_allowed_origins: vec!["http://localhost:3000".into()],
                require_https: false,
                rate_limit_rpm: 300,
            },
            observability: ObservabilityConfig {
                log_level: "info".into(),
                otlp_endpoint: None,
                metrics_enabled: false,
            },
        };

        let redacted = redacted_config(&config);
        let s = redacted.to_string();

        // Check that secrets are not in the output
        assert!(!s.contains("minioadmin"));
        assert!(!s.contains("super-secret-key"));
        assert!(!s.contains("sk-proj-abcdefghijklmnop"));

        // Check that redaction markers are present
        assert!(s.contains("[REDACTED]"));

        // Check that non-secret info is preserved
        assert!(s.contains("development"));
        assert!(s.contains("8000"));
        assert!(s.contains("gpt-4o"));
    }

    #[test]
    fn test_ai_disabled_validation() {
        let mut config = AppConfig {
            env: Environment::Development,
            server: ServerConfig { host: "0.0.0.0".into(), port: 8000 },
            database: Some(DatabaseConfig {
                url: "postgres://localhost/db".into(),
                max_connections: 10,
                connect_timeout_seconds: 5,
            }),
            redis: None,
            object_storage: None,
            auth: Some(AuthConfig {
                jwt_secret: "a-reasonable-jwt-secret-that-is-long".into(),
                jwt_expiration_seconds: 900,
                refresh_expiration_seconds: 604800,
            }),
            ai: AiConfig {
                enabled: false,
                provider: LlmProviderType::Disabled,
                timeout_seconds: 60,
                max_retries: 2,
                streaming_enabled: true,
                request_logging_enabled: false,
                default_temperature: 0.2,
                default_max_tokens: 4096,
                ollama: OllamaConfig { url: String::new(), model: String::new(), embedding_model: String::new() },
                openai: None, anthropic: None, openrouter: None, azure_openai: None, bedrock: None, custom_http: None,
            },
            features: FeatureConfig {
                ai_chat_enabled: false,
                rag_enabled: false,
                graph_enabled: true,
                plugin_system_enabled: false,
                document_ingestion_enabled: false,
                semantic_search_enabled: false,
                dispatch_enabled: true,
                parts_inventory_enabled: true,
                disposables_tracking_enabled: false,
                billing_enabled: false,
                api_docs_enabled: true,
                marketplace_enabled: false,
                experimental_enabled: false,
            },
            security: SecurityConfig {
                cors_allowed_origins: vec!["http://localhost:3000".into()],
                require_https: false,
                rate_limit_rpm: 300,
            },
            observability: ObservabilityConfig {
                log_level: "info".into(),
                otlp_endpoint: None,
                metrics_enabled: false,
            },
        };

        let validation = validate_config(&config);
        assert!(validation.is_valid(), "AI-disabled config should be valid: {:?}", validation.errors);
    }

    #[test]
    fn test_openai_missing_key_validation() {
        let mut config = AppConfig {
            env: Environment::Production,
            server: ServerConfig { host: "0.0.0.0".into(), port: 8000 },
            database: Some(DatabaseConfig {
                url: "postgres://localhost/db".into(),
                max_connections: 10,
                connect_timeout_seconds: 5,
            }),
            redis: None,
            object_storage: None,
            auth: Some(AuthConfig {
                jwt_secret: "a-very-secure-jwt-secret-that-is-long-enough-for-production".into(),
                jwt_expiration_seconds: 900,
                refresh_expiration_seconds: 604800,
            }),
            ai: AiConfig {
                enabled: true,
                provider: LlmProviderType::OpenAI,
                timeout_seconds: 60,
                max_retries: 2,
                streaming_enabled: true,
                request_logging_enabled: false,
                default_temperature: 0.2,
                default_max_tokens: 4096,
                ollama: OllamaConfig { url: String::new(), model: String::new(), embedding_model: String::new() },
                openai: Some(OpenAIConfig {
                    api_key: String::new(),
                    base_url: None,
                    model: "gpt-4o".into(),
                }),
                anthropic: None, openrouter: None, azure_openai: None, bedrock: None, custom_http: None,
            },
            features: FeatureConfig {
                ai_chat_enabled: true, rag_enabled: false, graph_enabled: true,
                plugin_system_enabled: false, document_ingestion_enabled: false,
                semantic_search_enabled: false, dispatch_enabled: true,
                parts_inventory_enabled: true, disposables_tracking_enabled: false,
                billing_enabled: false, api_docs_enabled: true,
                marketplace_enabled: false, experimental_enabled: false,
            },
            security: SecurityConfig {
                cors_allowed_origins: vec!["https://sip.example.com".into()],
                require_https: true,
                rate_limit_rpm: 300,
            },
            observability: ObservabilityConfig {
                log_level: "info".into(),
                otlp_endpoint: None,
                metrics_enabled: false,
            },
        };

        let validation = validate_config(&config);
        assert!(!validation.is_valid(), "Should have errors for missing API key");
        assert!(
            validation.errors.iter().any(|e| e.contains("API_KEY")),
            "Should mention API key: {:?}", validation.errors
        );
    }

    #[test]
    fn test_feature_dependency_validation() {
        let mut config = AppConfig {
            env: Environment::Staging,
            server: ServerConfig { host: "0.0.0.0".into(), port: 8000 },
            database: Some(DatabaseConfig {
                url: "postgres://localhost/db".into(),
                max_connections: 10,
                connect_timeout_seconds: 5,
            }),
            redis: None,
            object_storage: None,
            auth: Some(AuthConfig {
                jwt_secret: "a-very-secure-jwt-secret-that-is-long-enough-for-production".into(),
                jwt_expiration_seconds: 900,
                refresh_expiration_seconds: 604800,
            }),
            ai: AiConfig {
                enabled: false,
                provider: LlmProviderType::Disabled,
                timeout_seconds: 60, max_retries: 2, streaming_enabled: true,
                request_logging_enabled: false, default_temperature: 0.2, default_max_tokens: 4096,
                ollama: OllamaConfig { url: String::new(), model: String::new(), embedding_model: String::new() },
                openai: None, anthropic: None, openrouter: None, azure_openai: None, bedrock: None, custom_http: None,
            },
            features: FeatureConfig {
                ai_chat_enabled: false, rag_enabled: true, graph_enabled: true,
                plugin_system_enabled: false, document_ingestion_enabled: false,
                semantic_search_enabled: true, dispatch_enabled: true,
                parts_inventory_enabled: true, disposables_tracking_enabled: false,
                billing_enabled: false, api_docs_enabled: true,
                marketplace_enabled: false, experimental_enabled: false,
            },
            security: SecurityConfig {
                cors_allowed_origins: vec!["https://sip.example.com".into()],
                require_https: true,
                rate_limit_rpm: 300,
            },
            observability: ObservabilityConfig {
                log_level: "info".into(),
                otlp_endpoint: None,
                metrics_enabled: false,
            },
        };

        let validation = validate_config(&config);
        // RAG without AI in staging
        assert!(
            !validation.is_valid() || validation.errors.iter().any(|e| e.contains("RAG")),
            "RAG without AI should cause error in staging: {:?}", validation.errors
        );
    }

    #[test]
    fn test_production_jwt_placeholder() {
        let mut config = AppConfig {
            env: Environment::Production,
            server: ServerConfig { host: "0.0.0.0".into(), port: 8000 },
            database: Some(DatabaseConfig {
                url: "postgres://localhost/db".into(),
                max_connections: 10,
                connect_timeout_seconds: 5,
            }),
            redis: None,
            object_storage: None,
            auth: Some(AuthConfig {
                jwt_secret: "change-me-in-production-jwt-secret-min-32-chars".into(),
                jwt_expiration_seconds: 900,
                refresh_expiration_seconds: 604800,
            }),
            ai: AiConfig {
                enabled: false, provider: LlmProviderType::Disabled,
                timeout_seconds: 60, max_retries: 2, streaming_enabled: true,
                request_logging_enabled: false, default_temperature: 0.2, default_max_tokens: 4096,
                ollama: OllamaConfig { url: String::new(), model: String::new(), embedding_model: String::new() },
                openai: None, anthropic: None, openrouter: None, azure_openai: None, bedrock: None, custom_http: None,
            },
            features: FeatureConfig {
                ai_chat_enabled: false, rag_enabled: false, graph_enabled: true,
                plugin_system_enabled: false, document_ingestion_enabled: false,
                semantic_search_enabled: false, dispatch_enabled: true,
                parts_inventory_enabled: true, disposables_tracking_enabled: false,
                billing_enabled: false, api_docs_enabled: true,
                marketplace_enabled: false, experimental_enabled: false,
            },
            security: SecurityConfig {
                cors_allowed_origins: vec!["https://sip.example.com".into()],
                require_https: true,
                rate_limit_rpm: 300,
            },
            observability: ObservabilityConfig {
                log_level: "info".into(),
                otlp_endpoint: None,
                metrics_enabled: false,
            },
        };

        let validation = validate_config(&config);
        assert!(validation.errors.iter().any(|e| e.contains("JWT secret")),
            "Placeholder JWT in prod should error: {:?}", validation.errors);
    }

    #[test]
    fn test_config_status_no_secrets() {
        let config = AppConfig {
            env: Environment::Development,
            server: ServerConfig { host: "0.0.0.0".into(), port: 8000 },
            database: Some(DatabaseConfig {
                url: "postgres://sip:secret@localhost/db".into(),
                max_connections: 10,
                connect_timeout_seconds: 5,
            }),
            redis: None, object_storage: None,
            auth: Some(AuthConfig {
                jwt_secret: "a-reasonable-jwt-secret-that-is-long".into(),
                jwt_expiration_seconds: 900,
                refresh_expiration_seconds: 604800,
            }),
            ai: AiConfig {
                enabled: false, provider: LlmProviderType::Disabled,
                timeout_seconds: 60, max_retries: 2, streaming_enabled: true,
                request_logging_enabled: false, default_temperature: 0.2, default_max_tokens: 4096,
                ollama: OllamaConfig { url: String::new(), model: String::new(), embedding_model: String::new() },
                openai: None, anthropic: None, openrouter: None, azure_openai: None, bedrock: None, custom_http: None,
            },
            features: FeatureConfig::default(),
            security: SecurityConfig {
                cors_allowed_origins: vec![],
                require_https: false,
                rate_limit_rpm: 300,
            },
            observability: ObservabilityConfig {
                log_level: "info".into(),
                otlp_endpoint: None,
                metrics_enabled: false,
            },
        };

        let status = config_status(&config);
        let json_str = serde_json::to_string(&status).unwrap();
        assert!(!json_str.contains("a-reasonable-jwt-secret-that-is-long"), "JWT secret value leaked");
        // Check that the DB password is not present (the field name jwt_secret is expected)
        assert!(!json_str.contains("sip:secret@"), "DB password leaked");
    }

    #[test]
    fn test_backward_compat_ollama() {
        std::env::set_var("SIP_OLLAMA_URL", "http://ollama:11434");
        std::env::set_var("SIP_OLLAMA_MODEL", "llama3.2:1b");
        std::env::set_var("SIP_DATABASE_URL", "postgres://localhost/test");

        let mut config = Figment::new()
            .merge(Env::prefixed("SIP_"))
            .extract::<AppConfig>()
            .unwrap();

        apply_backward_compat(&mut config);

        assert_eq!(config.ai.ollama.url, "http://ollama:11434");
        assert_eq!(config.ai.ollama.model, "llama3.2:1b");

        std::env::remove_var("SIP_OLLAMA_URL");
        std::env::remove_var("SIP_OLLAMA_MODEL");
        std::env::remove_var("SIP_DATABASE_URL");
    }
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            ai_chat_enabled: true,
            rag_enabled: true,
            graph_enabled: true,
            plugin_system_enabled: false,
            document_ingestion_enabled: true,
            semantic_search_enabled: true,
            dispatch_enabled: true,
            parts_inventory_enabled: true,
            disposables_tracking_enabled: false,
            billing_enabled: false,
            api_docs_enabled: true,
            marketplace_enabled: false,
            experimental_enabled: false,
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: LlmProviderType::Disabled,
            timeout_seconds: default_timeout_seconds(),
            max_retries: default_max_retries(),
            streaming_enabled: default_streaming_enabled(),
            request_logging_enabled: false,
            default_temperature: default_temperature(),
            default_max_tokens: default_max_tokens(),
            ollama: OllamaConfig {
                url: default_ollama_url(),
                model: default_ollama_model(),
                embedding_model: default_embedding_model(),
            },
            openai: None,
            anthropic: None,
            openrouter: None,
            azure_openai: None,
            bedrock: None,
            custom_http: None,
        }
    }
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            url: default_ollama_url(),
            model: default_ollama_model(),
            embedding_model: default_embedding_model(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_server_host(),
            port: default_server_port(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            cors_allowed_origins: default_cors_allowed_origins(),
            require_https: false,
            rate_limit_rpm: default_rate_limit_rpm(),
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            otlp_endpoint: None,
            metrics_enabled: false,
        }
    }
}
