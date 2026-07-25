use crate::types::{Content, Tool, ToolResult};
use serde::Deserialize;
use serde_json::json;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

const MAX_LINES: usize = 2000;
const MAX_BYTES: usize = 51200; // 50KB

#[derive(Debug, Deserialize)]
struct ReadParams {
    path: String,
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Clone)]
enum TruncatedBy {
    Lines,
    Bytes,
}

#[derive(Debug, Clone)]
struct TruncationResult {
    content: String,
    truncated: bool,
    truncated_by: Option<TruncatedBy>,
    total_lines: usize,
    total_bytes: usize,
    output_lines: usize,
    output_bytes: usize,
    first_line_exceeds_limit: bool,
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn expand_path(file_path: &str) -> String {
    let mut path = file_path.to_string();

    // Strip leading @
    if path.starts_with('@') {
        path = path[1..].to_string();
    }

    // Normalize Unicode spaces to regular space (U+0020)
    path = path
        .replace('\u{00A0}', " ") // No-break space
        .replace('\u{2000}', " ") // En quad
        .replace('\u{2001}', " ") // Em quad
        .replace('\u{2002}', " ") // En space
        .replace('\u{2003}', " ") // Em space
        .replace('\u{2004}', " ") // Three-per-em space
        .replace('\u{2005}', " ") // Four-per-em space
        .replace('\u{2006}', " ") // Six-per-em space
        .replace('\u{2007}', " ") // Figure space
        .replace('\u{2008}', " ") // Punctuation space
        .replace('\u{2009}', " ") // Thin space
        .replace('\u{200A}', " ") // Hair space
        .replace('\u{202F}', " ") // Narrow no-break space
        .replace('\u{205F}', " ") // Medium mathematical space
        .replace('\u{3000}', " "); // Ideographic space

    // Expand ~ to home directory
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs::home_dir() {
            if path == "~" {
                return home.to_string_lossy().to_string();
            } else {
                return home.join(&path[2..]).to_string_lossy().to_string();
            }
        }
    }

    path
}

fn resolve_to_cwd(file_path: &str, cwd: &Path) -> PathBuf {
    let expanded = expand_path(file_path);
    let path = Path::new(&expanded);

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn resolve_read_path(file_path: &str, cwd: &Path) -> PathBuf {
    let resolved = resolve_to_cwd(file_path, cwd);

    // If file exists, return it
    if resolved.exists() {
        return resolved;
    }

    // Try macOS-specific variants
    let path_str = resolved.to_string_lossy();

    // Variant 1: AM/PM with narrow no-break space
    let am_pm_variant = path_str
        .replace(" AM.", "\u{202F}AM.")
        .replace(" PM.", "\u{202F}PM.");
    let am_pm_path = PathBuf::from(&am_pm_variant);
    if am_pm_path.exists() {
        return am_pm_path;
    }

    // Variant 2: NFD (decomposed) Unicode
    let nfd_variant: String = path_str.nfd().collect();
    let nfd_path = PathBuf::from(&nfd_variant);
    if nfd_path.exists() {
        return nfd_path;
    }

    // Variant 3: Curly quote
    let curly_variant = path_str.replace('\'', "\u{2019}");
    let curly_path = PathBuf::from(&curly_variant);
    if curly_path.exists() {
        return curly_path;
    }

    // Variant 4: NFD + curly quote
    let nfd_curly: String = curly_variant.nfd().collect();
    let nfd_curly_path = PathBuf::from(&nfd_curly);
    if nfd_curly_path.exists() {
        return nfd_curly_path;
    }

    // Return original if no variant exists
    resolved
}

fn truncate_head(content: &str) -> TruncationResult {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let total_bytes = content.len();

    // No truncation needed
    if total_lines <= MAX_LINES && total_bytes <= MAX_BYTES {
        return TruncationResult {
            content: content.to_string(),
            truncated: false,
            truncated_by: None,
            total_lines,
            total_bytes,
            output_lines: total_lines,
            output_bytes: total_bytes,
            first_line_exceeds_limit: false,
        };
    }

    // Check if first line exceeds limit
    if !lines.is_empty() && lines[0].len() > MAX_BYTES {
        return TruncationResult {
            content: String::new(),
            truncated: true,
            truncated_by: Some(TruncatedBy::Bytes),
            total_lines,
            total_bytes,
            output_lines: 0,
            output_bytes: 0,
            first_line_exceeds_limit: true,
        };
    }

    // Collect lines that fit
    let mut collected_lines = Vec::new();
    let mut current_bytes = 0;
    let mut truncated_by = None;

    for (i, line) in lines.iter().enumerate() {
        let line_bytes = line.len();
        let newline_bytes = if i > 0 { 1 } else { 0 }; // Add newline for all but first
        let total_with_line = current_bytes + line_bytes + newline_bytes;

        if total_with_line > MAX_BYTES {
            truncated_by = Some(TruncatedBy::Bytes);
            break;
        }

        if i >= MAX_LINES {
            truncated_by = Some(TruncatedBy::Lines);
            break;
        }

        collected_lines.push(*line);
        current_bytes = total_with_line;
    }

    let output_content = collected_lines.join("\n");
    let output_lines = collected_lines.len();
    let output_bytes = output_content.len();

    TruncationResult {
        content: output_content,
        truncated: true,
        truncated_by,
        total_lines,
        total_bytes,
        output_lines,
        output_bytes,
        first_line_exceeds_limit: false,
    }
}

fn read_tool_impl(params: ReadParams) -> Result<ToolResult, String> {
    // Get current working directory
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Resolve path
    let path = resolve_read_path(&params.path, &cwd);

    // Check if file exists and is readable
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    std::fs::File::open(&path)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    // Read file as UTF-8
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file as UTF-8: {}", e))?;

    // Split into lines
    let all_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let total_file_lines = all_lines.len();

    // Apply offset
    let start_line = match params.offset {
        Some(offset_1indexed) => {
            if offset_1indexed == 0 {
                return Err("Offset must be >= 1 (1-indexed)".to_string());
            }
            let start_line = offset_1indexed - 1; // Convert to 0-indexed
            if start_line >= total_file_lines {
                return Err(format!(
                    "Offset {} is beyond end of file ({} lines total)",
                    offset_1indexed, total_file_lines
                ));
            }
            start_line
        }
        None => 0,
    };

    // Apply user limit
    let (selected_content, user_limited_lines) = match params.limit {
        Some(limit) => {
            let end_line = (start_line + limit).min(all_lines.len());
            let content = all_lines[start_line..end_line].join("\n");
            (content, Some(end_line - start_line))
        }
        None => {
            let content = all_lines[start_line..].join("\n");
            (content, None)
        }
    };

    // Apply truncation
    let truncation = truncate_head(&selected_content);

    // Build output message
    let output_text = if truncation.first_line_exceeds_limit {
        let start_line_display = start_line + 1;
        let first_line_size = format_size(all_lines[start_line].len());
        format!(
            "[Line {} is {}, exceeds 50.0KB limit. Use bash: sed -n '{}p' {} | head -c 51200]",
            start_line_display,
            first_line_size,
            start_line_display,
            path.display()
        )
    } else if truncation.truncated {
        let start_line_display = start_line + 1;
        let end_line_display = start_line_display + truncation.output_lines - 1;
        let next_offset = end_line_display + 1;

        let mut output = truncation.content.clone();

        match truncation.truncated_by {
            Some(TruncatedBy::Lines) => {
                output.push_str(&format!(
                    "\n\n[Showing lines {}-{} of {}. Use offset={} to continue.]",
                    start_line_display, end_line_display, total_file_lines, next_offset
                ));
            }
            Some(TruncatedBy::Bytes) => {
                output.push_str(&format!(
                    "\n\n[Showing lines {}-{} of {} (50.0KB limit). Use offset={} to continue.]",
                    start_line_display, end_line_display, total_file_lines, next_offset
                ));
            }
            None => {}
        }

        output
    } else if let Some(user_limited_lines) = user_limited_lines {
        // Check if there's more content available
        if start_line + user_limited_lines < all_lines.len() {
            let remaining = all_lines.len() - (start_line + user_limited_lines);
            let next_offset = start_line + user_limited_lines + 1;
            format!(
                "{}\n\n[{} more lines in file. Use offset={} to continue.]",
                truncation.content, remaining, next_offset
            )
        } else {
            truncation.content
        }
    } else {
        truncation.content
    };

    Ok(ToolResult {
        content: vec![Content::Text { text: output_text }],
    })
}

pub fn create_read_tool() -> Tool {
    Tool {
        name: "read".to_string(),
        description: "Read the contents of a text file. Output is truncated to 2000 lines or 50KB \
                      (whichever is hit first). Use offset/limit for large files. When you need the full file, \
                      continue with offset until complete."
            .to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read (relative or absolute)"
                },
                "offset": {
                    "type": "integer",
                    "description": "Line number to start reading from (1-indexed)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read"
                }
            },
            "required": ["path"]
        }),
        execute: Box::new(|args: serde_json::Value| {
            let params: ReadParams = serde_json::from_value(args)
                .map_err(|e| format!("Invalid parameters: {}", e))?;

            read_tool_impl(params)
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_small_file() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_read_small.txt");
        fs::write(&file_path, "line 1\nline 2\nline 3").unwrap();

        let params = ReadParams {
            path: file_path.to_string_lossy().to_string(),
            offset: None,
            limit: None,
        };

        let result = read_tool_impl(params).unwrap();
        assert_eq!(result.content.len(), 1);
        
        if let Content::Text { text } = &result.content[0] {
            assert_eq!(text, "line 1\nline 2\nline 3");
        } else {
            panic!("Expected Text content");
        }

        fs::remove_file(file_path).ok();
    }

    #[test]
    fn test_read_with_offset() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_read_offset.txt");
        fs::write(&file_path, "line 1\nline 2\nline 3\nline 4").unwrap();

        let params = ReadParams {
            path: file_path.to_string_lossy().to_string(),
            offset: Some(2),
            limit: None,
        };

        let result = read_tool_impl(params).unwrap();
        
        if let Content::Text { text } = &result.content[0] {
            assert_eq!(text, "line 2\nline 3\nline 4");
        } else {
            panic!("Expected Text content");
        }

        fs::remove_file(file_path).ok();
    }

    #[test]
    fn test_read_with_limit() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_read_limit.txt");
        fs::write(&file_path, "line 1\nline 2\nline 3\nline 4\nline 5").unwrap();

        let params = ReadParams {
            path: file_path.to_string_lossy().to_string(),
            offset: None,
            limit: Some(3),
        };

        let result = read_tool_impl(params).unwrap();
        
        if let Content::Text { text } = &result.content[0] {
            assert!(text.contains("line 1\nline 2\nline 3"));
            assert!(text.contains("2 more lines in file"));
        } else {
            panic!("Expected Text content");
        }

        fs::remove_file(file_path).ok();
    }

    #[test]
    fn test_file_not_found() {
        let params = ReadParams {
            path: "/nonexistent/file/path.txt".to_string(),
            offset: None,
            limit: None,
        };

        let result = read_tool_impl(params);
        match result {
            Err(err) => assert!(err.contains("File not found")),
            Ok(_) => panic!("Expected error for non-existent file"),
        }
    }
}
