use std::{fs, path::Path};

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;
use serde_json::{json, Value};

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
        image_file_path: Option<&Path>,
    ) -> Result<LlmClassification> {
        validate_llm_base(config)?;

        let endpoint = chat_endpoint(&config.llm.base_uri);
        let prompt = build_prompt(file_name, content, image_file_path.is_some());
        let user_content = if let Some(path) = image_file_path {
            let image_data_url = image_path_to_data_url(path)?;
            json!([
              {"type": "text", "text": prompt},
              {"type": "image_url", "image_url": {"url": image_data_url}}
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
        let result = parse_classification_payload(&parsed, file_name)
            .with_context(|| format!("deserialize llm classification failed: {}", preview_text(&parsed)))?;

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
    let vocab = CONTROLLED_VOCAB.join("、");
    format!(
        r#"You are a strict file classification engine for a personal knowledge management system.

## Directory Structure
- 10 (身份基石): Identity, legal documents, certificates, health records, financial credentials
- 20 (责任领域): Ongoing responsibilities — finance, health, housing, career management
- 30 (行动项目): Active projects with goals and deadlines
- 40 (知识金库): Learning materials, research, books, notes, templates
- 50 (数字资产): Media, creative works, software resources
- 99 (历史档案): Completed or expired items to be archived

## Rules
1. doc_type MUST be one of: [{vocab}]
2. target_top_dir MUST be one of: [10, 20, 30, 40, 50, 99]
3. target_subpath: relative path using "/" separators, no "..", no drive letters, max 2 levels deep, use Chinese folder names matching the directory structure, MUST map to an existing folder, and MUST NOT invent new folders (if uncertain, use empty string)
4. core_title: concise Chinese title, 4–16 characters, no punctuation, no date prefix, no doc_type prefix
5. tags: 0–3 short keywords relevant to the content
6. people: names of people mentioned (empty if none)
7. note: one-sentence remark only if truly necessary, otherwise null
8. confidence: float 0.0–1.0 reflecting classification certainty

Return ONLY a JSON object, no markdown, no explanation.
All keys are required: doc_type, core_title, tags, people, note, target_top_dir, target_subpath, confidence."#
    )
}

fn build_prompt(file_name: &str, content: &str, has_image: bool) -> String {
    let mut excerpt = content.to_string();
    if excerpt.chars().count() > 4000 {
        excerpt = excerpt.chars().take(4000).collect();
    }

    let mode = if has_image {
        "An image is attached — treat it as the primary source; use the text excerpt as supplementary context."
    } else {
        "No image. Use the text excerpt as the primary source."
    };

    format!(
        "File name: {file_name}\n{mode}\n\nContent excerpt:\n{excerpt}\n\nStrict JSON schema (all fields required):\n{{\"doc_type\":\"\",\"core_title\":\"\",\"tags\":[],\"people\":[],\"note\":null,\"target_top_dir\":\"\",\"target_subpath\":\"\",\"confidence\":0.0}}\n\nRespond with a single JSON object only. All string values must be in Chinese unless they are proper nouns."
    )
}

fn message_content_to_string(content_node: &Value) -> Option<String> {
    if let Some(s) = content_node.as_str() {
        return Some(s.to_string());
    }

    let arr = content_node.as_array()?;
    let mut out = String::new();
    for item in arr {
        let text = item
            .get("text")
            .and_then(|v| v.as_str())
            .or_else(|| item.get("text").and_then(|v| v.get("value")).and_then(|v| v.as_str()));

        if let Some(text) = text {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(text);
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

fn parse_json_payload(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        if value.is_object() {
            return Ok(trimmed.to_string());
        }
    }

    let mut in_string = false;
    let mut escaped = false;
    let mut depth = 0usize;
    let mut start_idx: Option<usize> = None;
    let mut best_candidate: Option<(usize, String)> = None;

    for (idx, ch) in raw.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start_idx = Some(idx);
                }
                depth += 1;
            }
            '}' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = start_idx {
                        let candidate = &raw[start..=idx];
                        if let Ok(value) = serde_json::from_str::<Value>(candidate) {
                            if value.is_object() {
                                let score = classification_key_score(&value);
                                if score >= 3 {
                                    return Ok(candidate.to_string());
                                }
                                if best_candidate
                                    .as_ref()
                                    .map(|(best, _)| score > *best)
                                    .unwrap_or(true)
                                {
                                    best_candidate = Some((score, candidate.to_string()));
                                }
                            }
                        }
                    }
                    start_idx = None;
                }
            }
            _ => {}
        }
    }

    if let Some((_, payload)) = best_candidate {
        return Ok(payload);
    }

    anyhow::bail!(AppError::InvalidLlmResponse(format!(
        "no valid json object found, raw preview: {}",
        preview_text(raw)
    )));
}

fn classification_key_score(value: &Value) -> usize {
    let Some(obj) = value.as_object() else {
        return 0;
    };
    ["doc_type", "core_title", "target_top_dir", "target_subpath", "confidence"]
        .iter()
        .filter(|k| obj.contains_key(**k))
        .count()
}

fn parse_classification_payload(payload: &str, file_name: &str) -> Result<LlmClassification> {
    if let Ok(mut result) = serde_json::from_str::<LlmClassification>(payload) {
        normalize_classification(&mut result);
        return Ok(result);
    }

    let value: Value = serde_json::from_str(payload).with_context(|| {
        format!(
            "classification payload is not valid json object: {}",
            preview_text(payload)
        )
    })?;
    let obj = value.as_object().ok_or_else(|| {
        AppError::InvalidLlmResponse("classification payload is not json object".to_string())
    })?;

    let mut result = LlmClassification {
        doc_type: read_string(obj.get("doc_type")).unwrap_or_else(|| "素材".to_string()),
        core_title: read_string(obj.get("core_title")).unwrap_or_else(|| fallback_core_title(file_name)),
        tags: read_string_list(obj.get("tags")),
        people: read_string_list(obj.get("people")),
        note: read_string(obj.get("note")),
        target_top_dir: read_string(obj.get("target_top_dir")).unwrap_or_else(|| "50".to_string()),
        target_subpath: read_string(obj.get("target_subpath")).unwrap_or_default(),
        confidence: read_confidence(obj.get("confidence")).unwrap_or(0.0),
    };

    normalize_classification(&mut result);

    if !CONTROLLED_VOCAB.contains(&result.doc_type.as_str()) {
        result.doc_type = "素材".to_string();
    }
    if !TOP_DIR_CODES.contains(&result.target_top_dir.as_str()) {
        result.target_top_dir = "50".to_string();
    }
    if sanitize_relative_subpath(&result.target_subpath).is_none() {
        result.target_subpath.clear();
    }
    if !(0.0..=1.0).contains(&result.confidence) {
        result.confidence = 0.0;
    }

    Ok(result)
}

fn normalize_classification(result: &mut LlmClassification) {
    result.doc_type = result.doc_type.trim().to_string();
    result.core_title = result.core_title.trim().to_string();
    result.target_top_dir = result.target_top_dir.trim().to_string();
    result.target_subpath = result.target_subpath.trim().to_string();
    result.tags = result
        .tags
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    result.people = result
        .people
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    result.note = result
        .note
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if result.core_title.is_empty() {
        result.core_title = "图片待复核".to_string();
    }
}

fn read_string(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Some(Value::Number(n)) => Some(n.to_string()),
        Some(Value::Bool(v)) => Some(v.to_string()),
        _ => None,
    }
}

fn read_string_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| read_string(Some(v)))
            .collect(),
        Some(Value::String(s)) => s
            .split([',', '，', ';', '；', '、'])
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(ToString::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn read_confidence(value: Option<&Value>) -> Option<f32> {
    match value {
        Some(Value::Number(n)) => n.to_string().parse::<f32>().ok(),
        Some(Value::String(s)) => s.trim().parse::<f32>().ok(),
        _ => None,
    }
}

fn fallback_core_title(file_name: &str) -> String {
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or("图片待复核")
        .trim();
    if stem.chars().count() >= 4 {
        stem.chars().take(16).collect()
    } else {
        "图片待复核".to_string()
    }
}

fn preview_text(text: &str) -> String {
    let single_line = text.replace(['\r', '\n'], " ");
    let chars = single_line.chars().collect::<Vec<_>>();
    if chars.len() > 240 {
        let head: String = chars.into_iter().take(240).collect();
        format!("{head}...")
    } else {
        single_line
    }
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

fn image_path_to_data_url(path: &Path) -> Result<String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        _ => anyhow::bail!("unsupported image extension for llm classify: {}", ext),
    };
    let bytes = fs::read(path)
        .with_context(|| format!("read image for llm classify failed: {}", path.display()))?;
    let b64 = STANDARD.encode(bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

#[cfg(test)]
mod tests {
    use super::{parse_classification_payload, parse_json_payload};

    #[test]
    fn test_parse_json_payload() {
        let raw = "```json\n{\"doc_type\":\"笔记\"}\n```";
        let parsed = parse_json_payload(raw).expect("should parse");
        assert_eq!(parsed, "{\"doc_type\":\"笔记\"}");
    }

    #[test]
    fn test_parse_json_payload_skip_invalid_braces() {
        let raw = "analysis {not valid json}\n{\"doc_type\":\"素材\",\"core_title\":\"图片素材\",\"target_top_dir\":\"50\",\"target_subpath\":\"\",\"confidence\":0.4}";
        let parsed = parse_json_payload(raw).expect("should parse");
        assert!(parsed.contains("\"doc_type\":\"素材\""));
    }

    #[test]
    fn test_parse_classification_payload_with_string_confidence_and_tags() {
        let payload = "{\"doc_type\":\"素材\",\"core_title\":\"报销票据\",\"target_top_dir\":\"50\",\"target_subpath\":\"\",\"confidence\":\"0.35\",\"tags\":\"报销,票据\",\"people\":[],\"note\":null}";
        let parsed = parse_classification_payload(payload, "receipt.jpg").expect("should parse");
        assert_eq!(parsed.doc_type, "素材");
        assert_eq!(parsed.target_top_dir, "50");
        assert_eq!(parsed.tags, vec!["报销".to_string(), "票据".to_string()]);
        assert!((parsed.confidence - 0.35).abs() < f32::EPSILON);
    }
}
