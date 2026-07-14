use mdbook_core::book::{Book, BookItem};
use mdbook_core::errors::Result;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use regex::Regex;
use serde_json::Value;
use std::io::{self, Read};
use std::process;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

const HIGHLIGHTS_QUERY: &str = include_str!("../highlights.scm");

const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "comment",
    "constant.builtin",
    "constructor",
    "function",
    "function.call",
    "keyword",
    "label",
    "number",
    "operator",
    "punctuation.bracket",
    "punctuation.bracket.open",
    "punctuation.bracket.close",
    "punctuation.delimiter",
    "string",
    "string.special",
    "type",
    "type.builtin",
    "type.definition",
    "type.parameter",
    "variable",
    "variable.member",
    "variable.parameter",
];

struct SpectecHighlight;

impl Preprocessor for SpectecHighlight {
    fn name(&self) -> &str {
        "spectec-highlight"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let mut highlighter = Highlighter::new();
        let language = tree_sitter_spectec::LANGUAGE.into();
        let mut config =
            HighlightConfiguration::new(language, "spectec", HIGHLIGHTS_QUERY, "", "")
                .expect("failed to create highlight config");
        config.configure(HIGHLIGHT_NAMES);

        let fence_re = Regex::new(r"(?s)```spectec\n(.*?)```").unwrap();

        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                ch.content = fence_re
                    .replace_all(&ch.content, |caps: &regex::Captures| {
                        let code = &caps[1];
                        match highlight_code(&mut highlighter, &config, code) {
                            Ok(html) => {
                                format!("<pre class=\"spectec\"><code>{}</code></pre>", html)
                            }
                            Err(_) => format!("```\n{}```", code),
                        }
                    })
                    .into_owned();
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool> {
        Ok(renderer == "html")
    }
}

fn highlight_code(
    highlighter: &mut Highlighter,
    config: &HighlightConfiguration,
    code: &str,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let events = highlighter.highlight(config, code.as_bytes(), None, |_| None)?;

    let mut html = String::new();
    for event in events {
        match event? {
            HighlightEvent::Source { start, end } => {
                html.push_str(&escape_html(&code[start..end]));
            }
            HighlightEvent::HighlightStart(h) => {
                let class = HIGHLIGHT_NAMES[h.0].replace('.', "-");
                html.push_str(&format!("<span class=\"hl-{}\">", class));
            }
            HighlightEvent::HighlightEnd => {
                html.push_str("</span>");
            }
        }
    }

    Ok(html)
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// mdBook sends [ctx, book] where ctx.config may contain JSON null values
// that can't deserialize into TOML Values. Strip nulls from ctx only.
fn strip_nulls(v: Value) -> Value {
    match v {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k, strip_nulls(v)))
                .collect(),
        ),
        Value::Array(arr) => Value::Array(arr.into_iter().map(strip_nulls).collect()),
        other => other,
    }
}

fn main() {
    let preprocessor = SpectecHighlight;

    if let Some(arg) = std::env::args().nth(1) {
        if arg == "supports" {
            let renderer = std::env::args().nth(2).unwrap_or_default();
            process::exit(
                match preprocessor.supports_renderer(&renderer) {
                    Ok(true) => 0,
                    _ => 1,
                },
            );
        }
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("failed to read stdin");

    let mut arr: Value = serde_json::from_str(&input).expect("failed to parse input JSON");
    if let Value::Array(ref mut items) = arr {
        if let Some(ctx_val) = items.get_mut(0) {
            *ctx_val = strip_nulls(ctx_val.take());
        }
    }
    let cleaned = serde_json::to_string(&arr).expect("failed to re-serialize input");

    let (ctx, book) = mdbook_preprocessor::parse_input(cleaned.as_bytes())
        .expect("failed to parse preprocessor input");

    let result = preprocessor.run(&ctx, book).expect("preprocessor failed");
    serde_json::to_writer(io::stdout(), &result).expect("failed to write output");
}
