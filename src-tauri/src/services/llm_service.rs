use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;

use crate::{
    constants::{CONTROLLED_VOCAB, TOP_DIR_CODES},
    errors::AppError,
    models::{AppConfig, LlmClassification},
    utils::path_utils::sanitize_relative_subpath,
};

#[derive(Clone)]
pub struct LlmService {
    client: Client,
}

impl LlmService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn test_connection(&self, config: &AppConfig) -> Result<()> {
        validate_llm_base(config)?;

        let endpoint = chat_endpoint(&config.llm.base_uri);
        let body = json!({
          "model": config.llm.model,
          "messages": [
            {"role": "system", "content": "You are a test endpoint. Reply with OK."},
            {"role": "user", "content": "Reply exactly: OK"}
          ],
          "temperature": 0
        });

        self.client
            .post(endpoint)
            .timeout(std::time::Duration::from_secs(
                config.llm.timeout_sec.max(5),
            ))
            .bearer_auth(&config.llm.api_key_encrypted)
            .json(&body)
            .send()
            .await
            .context("llm request failed")?
            .error_for_status()
            .context("llm response status is not success")?;

        Ok(())
    }

    pub async fn classify(
        &self,
        config: &AppConfig,
        file_name: &str,
        content: &str,
        image_data_url: Option<&str>,
    ) -> Result<LlmClassification> {
        validate_llm_base(config)?;

        let endpoint = chat_endpoint(&config.llm.base_uri);
        let prompt = build_prompt(file_name, content, image_data_url.is_some());
        let user_content = if let Some(url) = image_data_url {
            json!([
              {"type": "text", "text": prompt},
              {"type": "image_url", "image_url": {"url": url}}
            ])
        } else {
            json!(prompt)
        };
        let body = json!({
          "model": config.llm.model,
          "temperature": 0.1,
          "messages": [
            {"role": "system", "content": system_prompt()},
            {"role": "user", "content": user_content}
          ]
        });

        let response: serde_json::Value = self
            .client
            .post(endpoint)
            .timeout(std::time::Duration::from_secs(
                config.llm.timeout_sec.max(5),
            ))
            .bearer_auth(&config.llm.api_key_encrypted)
            .json(&body)
            .send()
            .await
            .context("llm request failed")?
            .error_for_status()
            .context("llm response status is not success")?
            .json()
            .await
            .context("llm response parse failed")?;

        let content_node = response
            .get("choices")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("message"))
            .and_then(|v| v.get("content"))
            .ok_or_else(|| {
                AppError::InvalidLlmResponse("missing choices[0].message.content".to_string())
            })?;
        let raw = message_content_to_string(content_node).ok_or_else(|| {
            AppError::InvalidLlmResponse("cannot parse message content".to_string())
        })?;

        let parsed = parse_json_payload(&raw)?;
        let mut result: LlmClassification =
            serde_json::from_str(&parsed).context("deserialize llm classification failed")?;

        result.doc_type = result.doc_type.trim().to_string();
        result.core_title = result.core_title.trim().to_string();
        result.target_top_dir = result.target_top_dir.trim().to_string();
        result.target_subpath = result.target_subpath.trim().to_string();
        result.tags = result
            .tags
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        result.people = result
            .people
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        validate_classification(&result)?;
        Ok(result)
    }
}

fn validate_llm_base(config: &AppConfig) -> Result<()> {
    if config.llm.base_uri.trim().is_empty() {
        anyhow::bail!(AppError::InvalidConfig(
            "llm.base_uri is required".to_string()
        ));
    }
    if config.llm.model.trim().is_empty() {
        anyhow::bail!(AppError::InvalidConfig("llm.model is required".to_string()));
    }
    if config.llm.api_key_encrypted.trim().is_empty() {
        anyhow::bail!(AppError::InvalidConfig(
            "llm.api_key is required".to_string()
        ));
    }
    Ok(())
}

fn chat_endpoint(base_uri: &str) -> String {
    let trimmed = base_uri.trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/chat/completions")
    }
}

fn system_prompt() -> String {
    let vocab = CONTROLLED_VOCAB.join(", ");
    format!(
    "You are a strict file classification engine. Return ONLY JSON with keys: doc_type, core_title, tags, people, note, target_top_dir, target_subpath, confidence. doc_type must be one of: [{vocab}]. target_top_dir must be one of [10,20,30,40,50,99]. target_subpath must be a relative path using / separators without .. . confidence must be a float 0..1."
  )
}

fn build_prompt(file_name: &str, content: &str, has_image: bool) -> String {
    let mut excerpt = content.to_string();
    if excerpt.chars().count() > 4000 {
        excerpt = excerpt.chars().take(4000).collect();
    }

    let mode = if has_image {
        "This request includes an image. Use the image as the primary source and text excerpt as secondary context."
    } else {
        "This request is text-first. Use the text excerpt as the primary source."
    };

    format!(
    "File name: {file_name}\n{mode}\n\nContent excerpt:\n{excerpt}\n\nClassify and respond as JSON only."
  )
}

fn message_content_to_string(content_node: &serde_json::Value) -> Option<String> {
    if let Some(s) = content_node.as_str() {
        return Some(s.to_string());
    }

    let arr = content_node.as_array()?;
    let mut out = String::new();
    for item in arr {
        if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(text);
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

fn parse_json_payload(raw: &str) -> Result<String> {
    let start = raw
        .find('{')
        .ok_or_else(|| AppError::InvalidLlmResponse("no json object start".to_string()))?;
    let end = raw
        .rfind('}')
        .ok_or_else(|| AppError::InvalidLlmResponse("no json object end".to_string()))?;

    if end <= start {
        anyhow::bail!(AppError::InvalidLlmResponse(
            "json bounds invalid".to_string()
        ));
    }

    Ok(raw[start..=end].to_string())
}

fn validate_classification(result: &LlmClassification) -> Result<()> {
    if !CONTROLLED_VOCAB.contains(&result.doc_type.as_str()) {
        anyhow::bail!(AppError::InvalidLlmResponse(format!(
            "doc_type '{}' not in controlled vocabulary",
            result.doc_type
        )));
    }

    if !TOP_DIR_CODES.contains(&result.target_top_dir.as_str()) {
        anyhow::bail!(AppError::InvalidLlmResponse(format!(
            "target_top_dir '{}' invalid",
            result.target_top_dir
        )));
    }

    if result.core_title.trim().is_empty() {
        anyhow::bail!(AppError::InvalidLlmResponse(
            "core_title is empty".to_string()
        ));
    }

    if !(0.0..=1.0).contains(&result.confidence) {
        anyhow::bail!(AppError::InvalidLlmResponse(
            "confidence out of range".to_string()
        ));
    }

    if sanitize_relative_subpath(&result.target_subpath).is_none() {
        anyhow::bail!(AppError::InvalidLlmResponse(
            "target_subpath invalid".to_string()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_json_payload;

    #[test]
    fn test_parse_json_payload() {
        let raw = "```json\n{\"doc_type\":\"笔记\"}\n```";
        let parsed = parse_json_payload(raw).expect("should parse");
        assert_eq!(parsed, "{\"doc_type\":\"笔记\"}");
    }
}
