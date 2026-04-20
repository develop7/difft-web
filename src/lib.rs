use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub line: u32,
    pub start_col: u32,
    pub end_col: u32,
    pub novel: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredDiff {
    pub language: String,
    pub lhs_path: String,
    pub rhs_path: String,
    pub lhs_source: String,
    pub rhs_source: String,
    pub lhs_spans: Vec<Span>,
    pub rhs_spans: Vec<Span>,
}

#[cfg(not(target_os = "wasi"))]
pub fn generate_structured_diff(
    language_extension: &str,
    lhs_path: &str,
    rhs_path: &str,
    lhs_source: &str,
    rhs_source: &str,
) -> Result<StructuredDiff, String> {
    use difft_lib::options::{
        self, BackgroundColor, DisplayMode, DisplayOptions, DEFAULT_BYTE_LIMIT, DEFAULT_GRAPH_LIMIT,
        DEFAULT_TAB_WIDTH,
    };
    use difft_lib::diff_file;
    use std::ffi::OsStr;
    use std::fs;
    use tempfile::tempdir;

    let temp = tempdir().map_err(|err| format!("failed to create temp dir: {err}"))?;
    let lhs_file = temp.path().join(format!("left.{language_extension}"));
    let rhs_file = temp.path().join(format!("right.{language_extension}"));

    fs::write(&lhs_file, lhs_source).map_err(|err| format!("failed to write lhs file: {err}"))?;
    fs::write(&rhs_file, rhs_source).map_err(|err| format!("failed to write rhs file: {err}"))?;

    let display_options = DisplayOptions {
        background_color: BackgroundColor::Dark,
        use_color: false,
        display_mode: DisplayMode::Inline,
        print_unchanged: true,
        tab_width: DEFAULT_TAB_WIDTH,
        display_width: 120,
        in_vcs: false,
        syntax_highlight: false,
    };

    let language_override = options::guess_language::from_extension(OsStr::new(language_extension));

    let diff = diff_file(
        lhs_path,
        rhs_path,
        &lhs_file,
        &rhs_file,
        &display_options,
        false,
        DEFAULT_GRAPH_LIMIT,
        DEFAULT_BYTE_LIMIT,
        language_override,
    );

    Ok(StructuredDiff {
        language: diff
            .language
            .unwrap_or_else(|| fallback_language_name(language_extension)),
        lhs_path: diff.lhs_display_path,
        rhs_path: diff.rhs_display_path,
        lhs_source: lhs_source.to_string(),
        rhs_source: rhs_source.to_string(),
        lhs_spans: diff
            .lhs_positions
            .into_iter()
            .map(|position| Span {
                line: position.pos.line.0,
                start_col: position.pos.start_col,
                end_col: position.pos.end_col,
                novel: position.kind.is_novel(),
            })
            .collect(),
        rhs_spans: diff
            .rhs_positions
            .into_iter()
            .map(|position| Span {
                line: position.pos.line.0,
                start_col: position.pos.start_col,
                end_col: position.pos.end_col,
                novel: position.kind.is_novel(),
            })
            .collect(),
    })
}

fn fallback_language_name(language_extension: &str) -> String {
    match language_extension {
        "rs" => "Rust",
        "hs" => "Haskell",
        "ts" | "tsx" => "TypeScript",
        _ => language_extension,
    }
    .to_string()
}

#[cfg(not(target_os = "wasi"))]
pub fn generate_demo_diffs() -> Result<Vec<StructuredDiff>, String> {
    let rust = generate_structured_diff(
        "rs",
        "before.rs",
        "after.rs",
        "fn sum(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
        "fn sum(a: i32, b: i32) -> i32 {\n    a + b + 1\n}\n",
    )?;

    let haskell = generate_structured_diff(
        "hs",
        "before.hs",
        "after.hs",
        "sumTwo a b = a + b\n",
        "sumTwo a b = a + b + 1\n",
    )?;

    let ts = generate_structured_diff(
        "ts",
        "before.ts",
        "after.ts",
        "export const answer = 41;\n",
        "export const answer = 42;\n",
    )?;

    Ok(vec![rust, haskell, ts])
}

pub fn render_demo_html(diffs: &[StructuredDiff]) -> String {
    let mut html = String::from("<section class=\"diff-demo\">");

    for diff in diffs {
        html.push_str("<article class=\"diff-card\">");
        html.push_str(&format!(
            "<h2>{}</h2><p class=\"diff-files\">{} → {}</p>",
            escape_html(&diff.language),
            escape_html(&diff.lhs_path),
            escape_html(&diff.rhs_path)
        ));
        html.push_str("<div class=\"diff-columns\">");
        html.push_str("<pre class=\"diff-pane\">");
        html.push_str(&render_side(&diff.lhs_source, &diff.lhs_spans));
        html.push_str("</pre>");
        html.push_str("<pre class=\"diff-pane\">");
        html.push_str(&render_side(&diff.rhs_source, &diff.rhs_spans));
        html.push_str("</pre>");
        html.push_str("</div></article>");
    }

    html.push_str("</section>");
    html
}

fn render_side(source: &str, spans: &[Span]) -> String {
    let mut output = String::new();

    for (line_idx, line) in source.split('\n').enumerate() {
        let mut line_spans: Vec<&Span> = spans
            .iter()
            .filter(|span| span.line == line_idx as u32)
            .collect();
        line_spans.sort_by_key(|span| (span.start_col, span.end_col));

        let mut cursor = 0usize;
        let line_len = line.len();

        for span in line_spans {
            let start = span.start_col as usize;
            let end = span.end_col as usize;

            if start > line_len || end > line_len || start >= end || start < cursor {
                continue;
            }

            output.push_str(&escape_html(&line[cursor..start]));
            let class_name = if span.novel {
                "diff-span-novel"
            } else {
                "diff-span-unchanged"
            };
            output.push_str(&format!(
                "<span class=\"{class_name}\">{}</span>",
                escape_html(&line[start..end])
            ));
            cursor = end;
        }

        output.push_str(&escape_html(&line[cursor..]));
        output.push('\n');
    }

    output
}

fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_os = "wasi"))]
    #[test]
    fn generates_structured_diffs_with_novel_spans() {
        let diff = generate_structured_diff(
            "rs",
            "before.rs",
            "after.rs",
            "fn main() { println!(\"hi\"); }\n",
            "fn main() { println!(\"hello\"); }\n",
        )
        .expect("diff should generate");

        assert!(diff.lhs_spans.iter().any(|span| span.novel));
        assert!(diff.rhs_spans.iter().any(|span| span.novel));
        assert_eq!(diff.language.to_lowercase(), "rust");
    }

    #[cfg(not(target_os = "wasi"))]
    #[test]
    fn generates_demo_diffs_for_required_languages() {
        let diffs = generate_demo_diffs().expect("demo diff generation should work");
        let languages: Vec<String> = diffs.into_iter().map(|diff| diff.language).collect();

        assert!(languages.iter().any(|language| language == "Rust"));
        assert!(languages.iter().any(|language| language == "Haskell"));
        assert!(languages.iter().any(|language| language == "TypeScript"));
    }

    #[test]
    fn renders_diff_markup_with_language_and_file_names() {
        let diff = StructuredDiff {
            language: "TypeScript".to_string(),
            lhs_path: "before.ts".to_string(),
            rhs_path: "after.ts".to_string(),
            lhs_source: "const x = 1;".to_string(),
            rhs_source: "const x = 2;".to_string(),
            lhs_spans: vec![Span {
                line: 0,
                start_col: 10,
                end_col: 11,
                novel: true,
            }],
            rhs_spans: vec![Span {
                line: 0,
                start_col: 10,
                end_col: 11,
                novel: true,
            }],
        };

        let html = render_demo_html(&[diff]);
        assert!(html.contains("TypeScript"));
        assert!(html.contains("before.ts"));
        assert!(html.contains("after.ts"));
        assert!(html.contains("diff-span-novel"));
    }
}
