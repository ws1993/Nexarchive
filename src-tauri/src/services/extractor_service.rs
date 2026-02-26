use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use zip::ZipArchive;

pub struct ExtractorService;

pub struct ExtractedContent {
    pub text: String,
    pub image_data_url: Option<String>,
}

impl ExtractorService {
    pub fn new() -> Self {
        Self
    }

    pub async fn extract(&self, file_path: &Path) -> Result<ExtractedContent> {
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        match ext.as_str() {
            "txt" | "md" => {
                let text = fs::read_to_string(file_path)
                    .with_context(|| format!("read text failed: {}", file_path.display()))?;
                Ok(ExtractedContent {
                    text: limit_text(text),
                    image_data_url: None,
                })
            }
            "docx" => Ok(ExtractedContent {
                text: extract_docx(file_path)?,
                image_data_url: None,
            }),
            "xlsx" => Ok(ExtractedContent {
                text: extract_xlsx(file_path)?,
                image_data_url: None,
            }),
            "pptx" => Ok(ExtractedContent {
                text: extract_pptx(file_path)?,
                image_data_url: None,
            }),
            "pdf" => Ok(ExtractedContent {
                text: extract_pdf(file_path)?,
                image_data_url: None,
            }),
            "jpg" | "jpeg" | "png" => {
                let mime = if ext == "png" {
                    "image/png"
                } else {
                    "image/jpeg"
                };
                Ok(ExtractedContent {
                    text: String::new(),
                    image_data_url: Some(read_image_as_data_url(file_path, mime)?),
                })
            }
            _ => anyhow::bail!("unsupported extension: {}", ext),
        }
    }
}

fn extract_docx(path: &Path) -> Result<String> {
    extract_zip_xml_text(path, |name| {
        name == "word/document.xml"
            || name.starts_with("word/header")
            || name.starts_with("word/footer")
            || name == "word/footnotes.xml"
    })
}

fn extract_xlsx(path: &Path) -> Result<String> {
    extract_zip_xml_text(path, |name| {
        name == "xl/sharedStrings.xml" || name.starts_with("xl/worksheets/sheet")
    })
}

fn extract_pptx(path: &Path) -> Result<String> {
    extract_zip_xml_text(path, |name| {
        name.starts_with("ppt/slides/slide") && name.ends_with(".xml")
    })
}

fn extract_zip_xml_text<F>(path: &Path, mut include: F) -> Result<String>
where
    F: FnMut(&str) -> bool,
{
    let file = File::open(path).with_context(|| format!("open zip failed: {}", path.display()))?;
    let mut archive = ZipArchive::new(file).context("invalid zip format")?;
    let mut sections = Vec::new();

    for idx in 0..archive.len() {
        let mut entry = archive.by_index(idx).context("read zip entry failed")?;
        let name = entry.name().to_string();
        if !include(&name) || !name.ends_with(".xml") {
            continue;
        }

        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        let xml = String::from_utf8_lossy(&bytes).to_string();
        let text = clean_xml_text(&xml);
        if !text.is_empty() {
            sections.push(format!("## {}\n{}", name, text));
        }
    }

    Ok(limit_text(sections.join("\n\n")))
}

fn extract_pdf(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read pdf failed: {}", path.display()))?;
    let mut pieces = Vec::new();
    let mut buf = String::new();
    let mut in_paren = false;
    let mut escaped = false;

    for &b in &bytes {
        let ch = b as char;
        if in_paren {
            if escaped {
                buf.push(ch);
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                ')' => {
                    in_paren = false;
                    let t = buf.trim();
                    if t.len() > 2 {
                        pieces.push(t.to_string());
                    }
                    buf.clear();
                }
                _ => buf.push(ch),
            }
        } else if ch == '(' {
            in_paren = true;
            buf.clear();
        }
    }

    if pieces.is_empty() {
        let fallback = String::from_utf8_lossy(&bytes).to_string();
        for chunk in fallback.split_whitespace() {
            if chunk.chars().all(|c| c.is_ascii_graphic()) && chunk.len() >= 4 {
                pieces.push(chunk.to_string());
                if pieces.len() > 500 {
                    break;
                }
            }
        }
    }

    Ok(limit_text(pieces.join("\n")))
}

fn read_image_as_data_url(path: &Path, mime: &str) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read image failed: {}", path.display()))?;
    let b64 = STANDARD.encode(bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

fn clean_xml_text(xml: &str) -> String {
    let mut text = String::with_capacity(xml.len());
    let mut in_tag = false;

    for ch in xml.chars() {
        match ch {
            '<' => {
                in_tag = true;
                text.push('\n');
            }
            '>' => {
                in_tag = false;
            }
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }

    let decoded = text
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'");

    decoded
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn limit_text(text: String) -> String {
    let mut out = text;
    if out.chars().count() > 8000 {
        out = out.chars().take(8000).collect();
    }
    out
}
