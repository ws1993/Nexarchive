use std::path::{Component, Path, PathBuf};

pub fn sanitize_filename_component(input: &str) -> String {
    let invalid = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut out = input
        .chars()
        .map(|ch| {
            if ch.is_control() || invalid.contains(&ch) {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>();

    while out.ends_with('.') || out.ends_with(' ') {
        out.pop();
    }

    let trimmed = out.trim();
    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn sanitize_relative_subpath(subpath: &str) -> Option<PathBuf> {
    if subpath.trim().is_empty() {
        return Some(PathBuf::new());
    }

    let normalized = subpath.replace('\\', "/");
    let path = Path::new(normalized.as_str());
    if path.is_absolute() {
        return None;
    }

    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(seg) => {
                let seg_text = seg.to_string_lossy();
                if seg_text.contains(':') {
                    return None;
                }
                out.push(sanitize_filename_component(&seg_text));
            }
            Component::CurDir => continue,
            _ => return None,
        }
    }

    Some(out)
}

pub fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn unique_path(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string();
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let parent = path.parent().unwrap_or_else(|| Path::new("."));

    for idx in 1..10000 {
        let candidate = if ext.is_empty() {
            parent.join(format!("{stem}_dup{idx}"))
        } else {
            parent.join(format!("{stem}_dup{idx}.{ext}"))
        };
        if !candidate.exists() {
            return candidate;
        }
    }

    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::{sanitize_filename_component, sanitize_relative_subpath};

    #[test]
    fn test_sanitize_filename_component() {
        assert_eq!(sanitize_filename_component("a<b>c"), "a_b_c");
        assert_eq!(sanitize_filename_component(".. "), "untitled");
    }

    #[test]
    fn test_sanitize_relative_subpath() {
        assert!(sanitize_relative_subpath("foo/bar").is_some());
        assert!(sanitize_relative_subpath("../foo").is_none());
        assert!(sanitize_relative_subpath("C:/x").is_none());
    }
}
