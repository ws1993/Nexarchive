use std::{
    io::{Cursor, Read},
    path::Path,
    time::Duration,
};

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::time::{sleep, Instant};
use zip::ZipArchive;

use crate::models::{AppConfig, MineruConfig};

use super::extractor_service::ExtractedContent;

pub struct MineruService {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct MineruResponse<T> {
    code: i32,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
struct BatchCreateData {
    batch_id: String,
    #[serde(default)]
    file_urls: Vec<FileUploadUrl>,
}

#[derive(Debug, Deserialize)]
struct FileUploadUrl {
    name: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct BatchStatusData {
    state: String,
    #[serde(default)]
    extract_result: Vec<BatchExtractResult>,
    #[serde(default)]
    failed_list: Vec<BatchFailedResult>,
}

#[derive(Debug, Deserialize)]
struct BatchExtractResult {
    full_zip_url: Option<String>,
    md_zip_url: Option<String>,
    json_zip_url: Option<String>,
    err_msg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BatchFailedResult {
    err_msg: Option<String>,
}

struct CreateBatchResult {
    batch_id: String,
    upload_url: String,
}

impl MineruService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn is_configured(&self, config: &AppConfig) -> bool {
        let m = &config.mineru;
        m.enabled && !m.base_uri.trim().is_empty() && !m.api_token_encrypted.trim().is_empty()
    }

    pub fn supports_extension(ext: &str) -> bool {
        matches!(
            ext,
            "pdf" | "doc" | "docx" | "ppt" | "pptx" | "jpg" | "jpeg" | "png"
        )
    }

    pub async fn test_connection(&self, config: &AppConfig) -> Result<()> {
        let m = validate_config(config)?;
        let _ = self
            .create_batch(
                m,
                "connectivity-check.pdf",
                false,
                Duration::from_secs(m.timeout_sec.max(10)),
            )
            .await?;
        Ok(())
    }

    pub async fn extract(&self, config: &AppConfig, file_path: &Path) -> Result<ExtractedContent> {
        let m = validate_config(config)?;
        let file_name = file_path
            .file_name()
            .and_then(|s| s.to_str())
            .context("invalid file name")?;
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        if !Self::supports_extension(&ext) {
            anyhow::bail!("mineru unsupported extension: {}", ext);
        }

        let request_timeout = Duration::from_secs(m.timeout_sec.max(10));
        let batch = self
            .create_batch(m, file_name, m.is_ocr, request_timeout)
            .await?;

        self.upload_file(&batch.upload_url, file_path, request_timeout)
            .await?;

        let zip_url = self
            .wait_extract_done(m, &batch.batch_id, request_timeout)
            .await?;

        let zip_bytes = self.download_zip(m, &zip_url, request_timeout).await?;
        let text = extract_text_from_zip(&zip_bytes)?;

        Ok(ExtractedContent {
            text: limit_text(text),
            image_data_url: None,
        })
    }

    async fn create_batch(
        &self,
        m: &MineruConfig,
        file_name: &str,
        is_ocr: bool,
        timeout: Duration,
    ) -> Result<CreateBatchResult> {
        let endpoint = endpoint(&m.base_uri, "file-urls/batch");
        let token = normalized_token(&m.api_token_encrypted);
        let payload = json!({
          "enable_formula": true,
          "enable_table": true,
          "language": m.language,
          "layout_model": "doclayout_yolo",
          "backend": "pipeline",
          "parse_mode": "auto",
          "url": "",
          "model_version": m.model_version,
          "files": [{
            "name": file_name,
            "is_ocr": is_ocr,
            "data_id": file_name
          }]
        });

        let response: MineruResponse<BatchCreateData> = self
            .client
            .post(endpoint)
            .timeout(timeout)
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await
            .context("mineru create batch request failed")?
            .error_for_status()
            .context("mineru create batch status is not success")?
            .json()
            .await
            .context("mineru create batch response parse failed")?;

        if response.code != 0 {
            anyhow::bail!(
                "mineru create batch failed: {}",
                response.msg.unwrap_or_else(|| "unknown".to_string())
            );
        }

        let data = response.data.context("mineru create batch missing data")?;
        let mut file_urls = data.file_urls.into_iter();
        let upload_url = file_urls
            .find(|f| f.name == file_name)
            .or_else(|| file_urls.next())
            .map(|f| f.url)
            .context("mineru create batch missing upload url")?;

        Ok(CreateBatchResult {
            batch_id: data.batch_id,
            upload_url,
        })
    }

    async fn upload_file(&self, upload_url: &str, file_path: &Path, timeout: Duration) -> Result<()> {
        let bytes = tokio::fs::read(file_path)
            .await
            .with_context(|| format!("read file failed: {}", file_path.display()))?;

        self.client
            .put(upload_url)
            .timeout(timeout)
            .header("Content-Type", "application/octet-stream")
            .body(bytes)
            .send()
            .await
            .context("mineru upload request failed")?
            .error_for_status()
            .context("mineru upload status is not success")?;

        Ok(())
    }

    async fn wait_extract_done(
        &self,
        m: &MineruConfig,
        batch_id: &str,
        request_timeout: Duration,
    ) -> Result<String> {
        let token = normalized_token(&m.api_token_encrypted);
        let endpoint = endpoint(&m.base_uri, &format!("extract-results/batch/{batch_id}"));
        let deadline = Instant::now() + Duration::from_secs(m.max_wait_sec.max(30));

        loop {
            if Instant::now() > deadline {
                anyhow::bail!("mineru extract timeout after {}s", m.max_wait_sec.max(30));
            }

            let response: MineruResponse<BatchStatusData> = self
                .client
                .get(&endpoint)
                .timeout(request_timeout)
                .bearer_auth(&token)
                .send()
                .await
                .context("mineru query batch result failed")?
                .error_for_status()
                .context("mineru query batch status is not success")?
                .json()
                .await
                .context("mineru query batch response parse failed")?;

            if response.code != 0 {
                anyhow::bail!(
                    "mineru query batch failed: {}",
                    response.msg.unwrap_or_else(|| "unknown".to_string())
                );
            }

            let data = response.data.context("mineru query batch missing data")?;
            match data.state.as_str() {
                "done" => {
                    for item in &data.extract_result {
                        if let Some(url) = pick_zip_url(item) {
                            return Ok(url);
                        }
                    }
                    anyhow::bail!("mineru batch done but no downloadable zip url");
                }
                "failed" => {
                    if let Some(msg) = data
                        .failed_list
                        .iter()
                        .find_map(|f| f.err_msg.as_ref())
                        .or_else(|| data.extract_result.iter().find_map(|f| f.err_msg.as_ref()))
                    {
                        anyhow::bail!("mineru batch failed: {}", msg);
                    }
                    anyhow::bail!("mineru batch failed");
                }
                "pending" | "running" => {
                    sleep(Duration::from_secs(2)).await;
                }
                other => {
                    anyhow::bail!("mineru batch state unexpected: {}", other);
                }
            }
        }
    }

    async fn download_zip(
        &self,
        m: &MineruConfig,
        zip_url: &str,
        request_timeout: Duration,
    ) -> Result<Vec<u8>> {
        let first = self
            .client
            .get(zip_url)
            .timeout(request_timeout)
            .send()
            .await
            .context("mineru download zip request failed")?;

        if first.status().is_success() {
            return first
                .bytes()
                .await
                .map(|b| b.to_vec())
                .context("mineru download zip bytes failed");
        }

        let token = normalized_token(&m.api_token_encrypted);
        self.client
            .get(zip_url)
            .timeout(request_timeout)
            .bearer_auth(token)
            .send()
            .await
            .context("mineru download zip request with auth failed")?
            .error_for_status()
            .context("mineru download zip status is not success")?
            .bytes()
            .await
            .map(|b| b.to_vec())
            .context("mineru download zip bytes with auth failed")
    }
}

fn validate_config(config: &AppConfig) -> Result<&MineruConfig> {
    let m = &config.mineru;
    if !m.enabled {
        anyhow::bail!("mineru is disabled");
    }
    if m.base_uri.trim().is_empty() {
        anyhow::bail!("mineru.base_uri is required");
    }
    if m.api_token_encrypted.trim().is_empty() {
        anyhow::bail!("mineru.api_token is required");
    }
    Ok(m)
}

fn endpoint(base_uri: &str, path: &str) -> String {
    let base = base_uri.trim_end_matches('/');
    let p = path.trim_start_matches('/');
    format!("{base}/{p}")
}

fn normalized_token(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.len() > 7 && trimmed[..7].eq_ignore_ascii_case("bearer ") {
        trimmed[7..].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

fn pick_zip_url(item: &BatchExtractResult) -> Option<String> {
    item.full_zip_url
        .clone()
        .or_else(|| item.md_zip_url.clone())
        .or_else(|| item.json_zip_url.clone())
}

fn extract_text_from_zip(zip_bytes: &[u8]) -> Result<String> {
    let reader = Cursor::new(zip_bytes);
    let mut zip = ZipArchive::new(reader).context("mineru result zip invalid")?;

    let mut md_sections: Vec<(String, String)> = Vec::new();
    let mut txt_sections: Vec<(String, String)> = Vec::new();
    let mut json_sections: Vec<(String, String)> = Vec::new();

    for idx in 0..zip.len() {
        let mut entry = zip.by_index(idx).context("read zip entry failed")?;
        if !entry.is_file() {
            continue;
        }

        let name = entry.name().to_string();
        let lower = name.to_ascii_lowercase();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;

        if lower.ends_with(".md") {
            let content = String::from_utf8_lossy(&bytes).to_string();
            if !content.trim().is_empty() {
                md_sections.push((name, content));
            }
            continue;
        }

        if lower.ends_with(".txt") {
            let content = String::from_utf8_lossy(&bytes).to_string();
            if !content.trim().is_empty() {
                txt_sections.push((name, content));
            }
            continue;
        }

        if lower.ends_with(".json") {
            if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
                let mut texts = Vec::new();
                collect_json_strings(&value, &mut texts);
                if !texts.is_empty() {
                    json_sections.push((name, texts.join("\n")));
                }
            }
        }
    }

    if !md_sections.is_empty() {
        return Ok(join_sections(md_sections));
    }
    if !txt_sections.is_empty() {
        return Ok(join_sections(txt_sections));
    }
    if !json_sections.is_empty() {
        return Ok(join_sections(json_sections));
    }

    anyhow::bail!("mineru result zip has no readable text content")
}

fn collect_json_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => {
            let text = s.trim();
            if text.len() >= 2 {
                out.push(text.to_string());
            }
        }
        Value::Array(arr) => {
            for item in arr {
                collect_json_strings(item, out);
            }
        }
        Value::Object(map) => {
            for v in map.values() {
                collect_json_strings(v, out);
            }
        }
        _ => {}
    }
}

fn join_sections(sections: Vec<(String, String)>) -> String {
    sections
        .into_iter()
        .map(|(name, content)| format!("## {}\n{}", name, content.trim()))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn limit_text(text: String) -> String {
    let mut out = text;
    if out.chars().count() > 12000 {
        out = out.chars().take(12000).collect();
    }
    out
}
