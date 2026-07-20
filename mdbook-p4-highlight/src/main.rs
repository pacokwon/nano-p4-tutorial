use mdbook_core::book::{Book, BookItem};
use mdbook_core::errors::Result;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use regex::Regex;
use serde_json::Value;
use std::io::{self, Read};
use std::process;
use tree_sitter::Language;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

unsafe extern "C" {
    fn tree_sitter_p4() -> *const ();
}

fn p4_language() -> Language {
    unsafe { Language::from_raw(tree_sitter_p4() as _) }
}

const HIGHLIGHTS_QUERY: &str = include_str!("../highlights.scm");

const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "comment",
    "constant.builtin",
    "function",
    "function.call",
    "keyword",
    "keyword.conditional",
    "keyword.directive",
    "keyword.import",
    "keyword.repeat",
    "keyword.type",
    "number",
    "operator",
    "punctuation.bracket",
    "string",
    "type",
    "type.builtin",
    "variable.member",
    "variable.parameter",
];

struct P4Highlight;

impl Preprocessor for P4Highlight {
    fn name(&self) -> &str {
        "p4-highlight"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let mut highlighter = Highlighter::new();
        let language = p4_language();
        let mut config =
            HighlightConfiguration::new(language, "p4", HIGHLIGHTS_QUERY, "", "")
                .expect("failed to create highlight config");
        config.configure(HIGHLIGHT_NAMES);

        let fence_re = Regex::new(r"(?s)```p4\n(.*?)```").unwrap();

        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                ch.content = fence_re
                    .replace_all(&ch.content, |caps: &regex::Captures| {
                        let code = &caps[1];
                        match highlight_code(&mut highlighter, &config, code) {
                            Ok(html) => {
                                format!("<pre class=\"p4\"><code>{}</code></pre>", html)
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
    let preprocessor = P4Highlight;

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
