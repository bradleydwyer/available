use rmcp::{
    ErrorData as McpError, ServerHandler, handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters, model::*, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::check;
use crate::generate;
use crate::provider;
use crate::types::{AvailableResult, Config, NameCandidate};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindNamesParams {
    #[schemars(description = "Description of the project to generate names for")]
    pub prompt: String,
    #[schemars(description = "Maximum number of names to generate (default: 20)")]
    pub max_names: Option<usize>,
    #[schemars(description = "Comma-separated TLDs to check (default: com,dev,io)")]
    pub tlds: Option<String>,
    #[schemars(
        description = "Comma-separated registry IDs to check (default: popular registries)"
    )]
    pub registries: Option<String>,
    #[schemars(
        description = "Comma-separated model names to use (default: auto-detect from API keys)"
    )]
    pub models: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CheckNamesParams {
    #[schemars(description = "List of names to check availability for")]
    pub names: Vec<String>,
    #[schemars(description = "Comma-separated TLDs to check (default: com,dev,io)")]
    pub tlds: Option<String>,
    #[schemars(
        description = "Comma-separated registry IDs to check (default: popular registries)"
    )]
    pub registries: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListModelsParams {}

pub struct AvailableMcp {
    tool_router: ToolRouter<Self>,
}

impl Default for AvailableMcp {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_config(tlds: &Option<String>, registries: &Option<String>) -> Config {
    let mut config = Config::default();
    if let Some(tlds) = tlds {
        config.tlds = tlds.split(',').map(|s| s.trim().to_string()).collect();
    }
    if let Some(registries) = registries {
        config.registry_ids = registries
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
    }
    config
}

#[tool_router]
impl AvailableMcp {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[rmcp::tool(
        description = "Generate project name suggestions using AI and check their availability across domains and package registries. Returns scored results ranked by availability."
    )]
    async fn find_names(
        &self,
        Parameters(params): Parameters<FindNamesParams>,
    ) -> Result<CallToolResult, McpError> {
        let models = match params.models {
            Some(ref m) => m.split(',').map(|s| s.trim().to_string()).collect(),
            None => provider::default_models(),
        };
        if models.is_empty() {
            return Err(McpError::invalid_params(
                "No LLM models available. Set at least one API key (ANTHROPIC_API_KEY, OPENAI_API_KEY, GOOGLE_API_KEY, XAI_API_KEY).",
                None,
            ));
        }

        let multi = provider::build_provider(&models)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let mut config = parse_config(&params.tlds, &params.registries);
        config.max_names = params.max_names.unwrap_or(20);

        let (candidates, errors) =
            generate::generate_names(&multi, &params.prompt, config.max_names).await;

        if candidates.is_empty() {
            let error_msg = if errors.is_empty() {
                "No valid names generated. Try a different prompt.".to_string()
            } else {
                format!(
                    "No valid names generated. Errors: {}",
                    errors
                        .iter()
                        .map(|e| format!("{}: {}", e.model, e.error))
                        .collect::<Vec<_>>()
                        .join("; ")
                )
            };
            return Err(McpError::internal_error(error_msg, None));
        }

        let results = check::check_names(&candidates, &config).await;

        let output = AvailableResult {
            results,
            models_used: models,
            errors,
        };
        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| McpError::internal_error(format!("Serialization error: {e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[rmcp::tool(
        description = "Check availability of specific project names across domains and package registries. Use this when you already have name ideas."
    )]
    async fn check_names(
        &self,
        Parameters(params): Parameters<CheckNamesParams>,
    ) -> Result<CallToolResult, McpError> {
        if params.names.is_empty() {
            return Err(McpError::invalid_params("names list cannot be empty", None));
        }
        if params.names.len() > 50 {
            return Err(McpError::invalid_params(
                "Maximum 50 names per request",
                None,
            ));
        }

        let config = parse_config(&params.tlds, &params.registries);
        let candidates: Vec<NameCandidate> = params
            .names
            .iter()
            .map(|n| NameCandidate {
                name: n.clone(),
                suggested_by: vec![],
            })
            .collect();

        let results = check::check_names(&candidates, &config).await;

        let output = AvailableResult {
            results,
            models_used: vec![],
            errors: vec![],
        };
        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| McpError::internal_error(format!("Serialization error: {e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[rmcp::tool(
        description = "List configured LLM models available for name generation, based on which API keys are set."
    )]
    async fn list_models(
        &self,
        Parameters(_params): Parameters<ListModelsParams>,
    ) -> Result<CallToolResult, McpError> {
        let models = provider::default_models();
        let info = if models.is_empty() {
            "No API keys configured. Set at least one of: ANTHROPIC_API_KEY, OPENAI_API_KEY, GOOGLE_API_KEY, XAI_API_KEY".to_string()
        } else {
            serde_json::to_string_pretty(&models)
                .map_err(|e| McpError::internal_error(format!("Serialization error: {e}"), None))?
        };
        Ok(CallToolResult::success(vec![Content::text(info)]))
    }
}

#[tool_handler]
impl ServerHandler for AvailableMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "AI-powered project name finder. Use find_names to generate and check names, \
                 check_names to check specific names, or list_models to see available LLMs."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            ..Default::default()
        }
    }
}
