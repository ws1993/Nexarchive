use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use anyhow::{Context, Result};
use zip::ZipArchive;

pub struct ExtractorService;

pub struct ExtractedContent {
    pub text: String,
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
            "txt" | "md" | "markdown" => Ok(ExtractedContent {
                text: read_plain_text(file_path)?,
            }),
            "html" | "htm" => Ok(ExtractedContent {
                text: extract_html(file_path)?,
            }),
            "docx" => Ok(ExtractedContent {
                text: extract_docx(file_path)?,
            }),
            "xlsx" => Ok(ExtractedContent {
                text: extract_xlsx(file_path)?,
            }),
            "pptx" => Ok(ExtractedContent {
                text: extract_pptx(file_path)?,
            }),
            "pdf" => Ok(ExtractedContent {
                text: extract_pdf(file_path)?,
            }),
            "jpg" | "jpeg" | "png" => {
                anyhow::bail!("image files should be sent directly to llm classify");
            }
            _ => anyhow::bail!("unsupported extension: {}", ext),
        }
    }
}

fn read_plain_text(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read text failed: {}", path.display()))?;
    Ok(limit_text(String::from_utf8_lossy(&bytes).to_string()))
}

fn extract_html(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read html failed: {}", path.display()))?;
    let html = String::from_utf8_lossy(&bytes);
    Ok(limit_text(clean_html_text(&html)))
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

    let decoded = decode_basic_entities(text);
    normalize_text_lines(&decoded)
}

fn clean_html_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut tag_buf = String::new();
    let mut skip_until: Option<&str> = None;

    for ch in html.chars() {
        if in_tag {
            if ch == '>' {
                in_tag = false;
                let trimmed = tag_buf.trim().trim_start_matches('!').trim();
                let lower = trimmed.to_ascii_lowercase();
                let (is_closing, rest) = if let Some(rest) = lower.strip_prefix('/') {
                    (true, rest)
                } else {
                    (false, lower.as_str())
                };
                let tag = rest
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .trim_end_matches('/');

                if let Some(target) = skip_until {
                    if is_closing && tag == target {
                        skip_until = None;
                    }
                } else {
                    match tag {
                        "script" if !is_closing => skip_until = Some("script"),
                        "style" if !is_closing => skip_until = Some("style"),
                        "br" | "p" | "div" | "li" | "tr" | "td" | "th" | "h1" | "h2" | "h3"
                        | "h4" | "h5" | "h6" => text.push('\n'),
                        _ => {}
                    }
                }
                tag_buf.clear();
            } else {
                tag_buf.push(ch);
            }
            continue;
        }

        if ch == '<' {
            in_tag = true;
            tag_buf.clear();
            continue;
        }

        if skip_until.is_none() {
            text.push(ch);
        }
    }

    let decoded = decode_basic_entities(text);
    normalize_text_lines(&decoded)
}

fn decode_basic_entities(text: String) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&nbsp;", " ")
}

fn normalize_text_lines(text: &str) -> String {
    text.lines()
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

#[cfg(test)]
mod tests {
    use super::clean_html_text;

    #[test]
    fn clean_html_text_strips_tags_and_ignores_script_style() {
        let html = r#"
          <html>
            <head>
              <style>.a{color:red;}</style>
              <script>console.log("hidden")</script>
            </head>
            <body>
              <h1>标题</h1>
              <p>Hello <b>World</b></p>
              <div>第二行</div>
            </body>
          </html>
        "#;

        let cleaned = clean_html_text(html);

        assert!(cleaned.contains("标题"));
        assert!(cleaned.contains("Hello"));
        assert!(cleaned.contains("World"));
        assert!(cleaned.contains("第二行"));
        assert!(!cleaned.contains("console.log"));
        assert!(!cleaned.contains("color:red"));
    }

    #[test]
    fn clean_html_text_decodes_entities() {
        let html = "<p>&lt;tag&gt; &amp; &#39;A&#39; &nbsp; &quot;B&quot;</p>";
        let cleaned = clean_html_text(html);
        assert!(cleaned.contains("<tag> & 'A'"));
        assert!(cleaned.contains("\"B\""));
    }
}
