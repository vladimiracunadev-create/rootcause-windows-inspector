//! Adaptador opcional de IA.
//!
//! RootCause no depende de este módulo para detectar, alertar o guardar
//! evidencia. Solo se usa bajo demanda para enriquecer un incidente ya
//! resumido.

use crate::config::AiConfig;
use crate::models::{AiIncidentAdvice, IncidentSummary};
use crate::services::windows;
use anyhow::{Context, Result, anyhow, bail};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::env;
use std::fs;

pub struct AiAdvisor {
    config: AiConfig,
}

impl AiAdvisor {
    pub fn new(config: AiConfig) -> Self {
        Self { config }
    }

    pub fn enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn summarize_incident(&self, incident: &IncidentSummary) -> Result<AiIncidentAdvice> {
        if !self.config.enabled {
            bail!("La integración IA está desactivada en la configuración");
        }
        if self.config.endpoint.trim().is_empty() {
            bail!("Falta ai.endpoint en la configuración");
        }

        let api_key = env::var(&self.config.api_key_env_var).with_context(|| {
            format!(
                "No existe la variable de entorno {} con la API key",
                self.config.api_key_env_var
            )
        })?;

        let response = invoke_openai_compatible(
            &self.config.endpoint,
            &api_key,
            &self.config.model,
            self.config.timeout_secs,
            incident,
        )?;
        parse_openai_compatible_response(&response, &self.config)
    }
}

#[derive(Debug, Serialize)]
struct AiRequestMessage {
    role: &'static str,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AiOutputShape {
    summary: String,
    probable_causes: Vec<String>,
    suggested_actions: Vec<String>,
    confidence: String,
    warnings: Vec<String>,
}

fn invoke_openai_compatible(
    endpoint: &str,
    api_key: &str,
    model: &str,
    timeout_secs: u64,
    incident: &IncidentSummary,
) -> Result<String> {
    let payload = json!({
        "model": model,
        "temperature": 0.1,
        "response_format": { "type": "json_object" },
        "messages": [
            AiRequestMessage {
                role: "system",
                content: "Eres un analista SRE. Responde solo JSON con las claves summary, probable_causes, suggested_actions, confidence y warnings.".to_owned(),
            },
            AiRequestMessage {
                role: "user",
                content: build_user_prompt(incident),
            }
        ]
    });

    let request_path = std::env::temp_dir().join(format!(
        "rootcause-ai-request-{}.json",
        Utc::now().format("%Y%m%d-%H%M%S-%3f")
    ));
    fs::write(&request_path, serde_json::to_string(&payload)?)
        .with_context(|| format!("No se pudo escribir {}", request_path.display()))?;

    let safe_endpoint = endpoint.replace('\'', "''");
    let safe_key = api_key.replace('\'', "''");
    let safe_path = request_path.display().to_string().replace('\'', "''");

    let script = format!(
        "$headers = @{{ Authorization = 'Bearer {safe_key}'; 'Content-Type' = 'application/json' }}; \
         $body = Get-Content -Raw -Path '{safe_path}'; \
         $response = Invoke-RestMethod -Uri '{safe_endpoint}' -Method Post -Headers $headers -Body $body -TimeoutSec {timeout_secs}; \
         $response | ConvertTo-Json -Depth 20 -Compress"
    );

    let result = windows::powershell(&script);
    let _ = fs::remove_file(&request_path);
    result
}

fn build_user_prompt(incident: &IncidentSummary) -> String {
    format!(
        "Analiza este incidente de RootCause y devuelve solo JSON.\n\
         Título: {}\n\
         Severidad: {:?}\n\
         Tipo: {}\n\
         Resumen: {}\n\
         Causas probables detectadas: {}\n\
         Evidencia: {}\n\
         Acciones recomendadas locales: {}",
        incident.title,
        incident.severity,
        incident.kind,
        incident.summary,
        incident.probable_causes.join(" | "),
        incident
            .evidence
            .iter()
            .map(|item| format!("{}={} ({})", item.label, item.value, item.kind))
            .collect::<Vec<_>>()
            .join(" | "),
        incident.recommended_actions.join(" | ")
    )
}

fn parse_openai_compatible_response(raw: &str, config: &AiConfig) -> Result<AiIncidentAdvice> {
    let value = serde_json::from_str::<Value>(raw)
        .with_context(|| "La respuesta del proveedor IA no es JSON válido")?;
    let content = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("No se encontró choices[0].message.content en la respuesta IA"))?;

    let cleaned = strip_code_fences(content);
    let parsed = serde_json::from_str::<AiOutputShape>(cleaned).with_context(
        || "El contenido generado por IA no respeta el JSON esperado para RootCause",
    )?;

    Ok(AiIncidentAdvice {
        provider: config.endpoint.clone(),
        model: config.model.clone(),
        summary: parsed.summary,
        probable_causes: parsed.probable_causes,
        suggested_actions: parsed.suggested_actions,
        confidence: parsed.confidence,
        warnings: parsed.warnings,
        generated_at: Utc::now().to_rfc3339(),
    })
}

fn strip_code_fences(text: &str) -> &str {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("```json") {
        return rest.trim().trim_end_matches("```").trim();
    }
    if let Some(rest) = trimmed.strip_prefix("```") {
        return rest.trim().trim_end_matches("```").trim();
    }
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remueve_code_fences() {
        let raw = "```json\n{\"summary\":\"ok\",\"probable_causes\":[],\"suggested_actions\":[],\"confidence\":\"alta\",\"warnings\":[]}\n```";
        assert!(strip_code_fences(raw).starts_with('{'));
    }
}
