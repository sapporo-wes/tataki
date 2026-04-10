use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::bail;

use crate::module::{InvokeOptions, ModuleResult};
use crate::parser::Parser;

pub struct Html;

impl Parser for Html {
    fn determine_from_path(
        &self,
        input_path: &Path,
        _options: &InvokeOptions,
    ) -> anyhow::Result<ModuleResult> {
        // Check file extension first: browsers render any .html/.htm file
        // regardless of content validity.
        let has_html_ext = input_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("html") || e.eq_ignore_ascii_case("htm"))
            .unwrap_or(false);

        if !has_html_ext {
            // Fall back to content check: require BOTH doctype and <html> tag
            // to avoid false positives from markdown/text files that contain
            // <html> inside code blocks.
            let mut buf = [0u8; 512];
            let mut file = File::open(input_path)?;
            let n = file.read(&mut buf)?;
            let text = std::str::from_utf8(&buf[..n])
                .unwrap_or("")
                .to_ascii_lowercase();
            if !text.contains("<!doctype html") || !text.contains("<html") {
                bail!(
                    "Not an HTML file: no .html/.htm extension and content lacks both <!DOCTYPE html> and <html> tag"
                );
            }
        }

        Ok(ModuleResult::with_result(
            Some("HTML".to_string()),
            Some("http://edamontology.org/format_2331".to_string()),
        ))
    }
}
