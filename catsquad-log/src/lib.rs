use std::env;

use tracing::span;
use tracing_subscriber::fmt::format::PrettyVisitor;
use tracing_subscriber::fmt::format::Writer;
use wasm_bindgen::prelude::*;

pub mod prelude {
    pub use crate::init_log;
    pub use tracing::debug;
    pub use tracing::error;
    pub use tracing::info;
    pub use tracing::trace;
}

pub use tracing::Level;

#[derive(Debug, Clone)]
struct SpanBody(pub String);

// RUST_LOG="catsuad=trace"

// #[cfg(target_arch = "wasm32")]
pub fn init_log() {
    let rust_log = env::var("RUST_LOG");
    let mut config = LogConfig::default();
    if let Ok(rust_log) = rust_log
        && !rust_log.is_empty()
    {
        config.max_level = MaxLevelType::from(&rust_log);
    }
    init_log_with_config(config);
}

pub fn init_log_with_config(config: LogConfig) {
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt::with(
            tracing_subscriber::Registry::default(),
            LogTracingLayer::new(config),
        ),
    );
}

// #[cfg(not(target_arch = "wasm32"))]
// pub fn init_log() {
//     simple_shell_logger_init(MaxLevelType::Global(Level::TRACE));
// }

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogConfig {
    pub target: bool,
    pub line: bool,
    pub max_level: MaxLevelType,
    pub colors: ColorKind,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaxLevelType {
    List(Vec<(String, Level)>),
    Global(Level),
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorKind {
    Web,
    Ascii,
}

struct LogTracingLayer {
    pub config: LogConfig,
}

impl<T> From<T> for MaxLevelType
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        let value = value.as_ref();
        let value = value.trim();
        match value {
            "trace" => return Self::Global(Level::TRACE),
            "debug" => return Self::Global(Level::DEBUG),
            "info" => return Self::Global(Level::INFO),
            "warn" | "warning" | "warnings" => return Self::Global(Level::WARN),
            "err" | "error" | "errors" => return Self::Global(Level::ERROR),
            _ => {}
        }
        let mut list = Vec::<(String, Level)>::new();
        let values = value.split(",");
        for value in values {
            let value = value.to_lowercase();
            let mut value = value.split("=").take(2);
            let value_target = value.next();
            let value_level = value.next();

            let (value_target, value_level) = match (value_target, value_level) {
                (Some(v1), Some(v2)) if !v1.is_empty() && !v2.is_empty() => (v1, v2),
                _ => continue,
            };

            let level = match value_level {
                "trace" => Level::TRACE,
                "debug" => Level::DEBUG,
                "info" => Level::INFO,
                "warn" | "warning" | "warnings" => Level::WARN,
                "err" | "error" | "errors" => Level::ERROR,
                _ => {
                    continue;
                }
            };

            let target = value_target.to_string();

            list.push((target, level));
        }

        if list.is_empty() {
            Self::Global(Level::TRACE)
        } else {
            Self::List(list)
        }
    }
}

#[test]
fn test_max_level_from_str() {
    let lvl = MaxLevelType::from("trace");
    assert_eq!(lvl, MaxLevelType::Global(Level::TRACE));
    let lvl = MaxLevelType::from("debug");
    assert_eq!(lvl, MaxLevelType::Global(Level::DEBUG));
    let lvl = MaxLevelType::from("info");
    assert_eq!(lvl, MaxLevelType::Global(Level::INFO));
    let lvl = MaxLevelType::from("warn");
    assert_eq!(lvl, MaxLevelType::Global(Level::WARN));
    let lvl = MaxLevelType::from("warning");
    assert_eq!(lvl, MaxLevelType::Global(Level::WARN));
    let lvl = MaxLevelType::from("warnings");
    assert_eq!(lvl, MaxLevelType::Global(Level::WARN));
    let lvl = MaxLevelType::from("err");
    assert_eq!(lvl, MaxLevelType::Global(Level::ERROR));
    let lvl = MaxLevelType::from("error");
    assert_eq!(lvl, MaxLevelType::Global(Level::ERROR));
    let lvl = MaxLevelType::from("errors");
    assert_eq!(lvl, MaxLevelType::Global(Level::ERROR));

    assert_eq!(
        MaxLevelType::from("a=trace"),
        MaxLevelType::List(vec![("a".to_string(), Level::TRACE)])
    );
    assert_eq!(
        MaxLevelType::from("a=debug"),
        MaxLevelType::List(vec![("a".to_string(), Level::DEBUG)])
    );
    assert_eq!(
        MaxLevelType::from("a=info"),
        MaxLevelType::List(vec![("a".to_string(), Level::INFO)])
    );
    assert_eq!(
        MaxLevelType::from("a=warn"),
        MaxLevelType::List(vec![("a".to_string(), Level::WARN)])
    );
    assert_eq!(
        MaxLevelType::from("a=ERR"),
        MaxLevelType::List(vec![("a".to_string(), Level::ERROR)])
    );

    assert_eq!(
        MaxLevelType::from("a=trace,b=debug,c=info,d=warn,e=err"),
        MaxLevelType::List(vec![
            ("a".to_string(), Level::TRACE),
            ("b".to_string(), Level::DEBUG),
            ("c".to_string(), Level::INFO),
            ("d".to_string(), Level::WARN),
            ("e".to_string(), Level::ERROR),
        ])
    );
}

#[cfg(target_arch = "wasm32")]
impl Default for LogConfig {
    fn default() -> Self {
        Self {
            line: true,
            target: true,
            max_level: MaxLevelType::Global(Level::TRACE),
            colors: ColorKind::Web,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for LogConfig {
    fn default() -> Self {
        Self {
            line: true,
            target: true,
            max_level: MaxLevelType::Global(Level::TRACE),
            colors: ColorKind::Ascii,
        }
    }
}

impl LogTracingLayer {
    pub fn new(config: LogConfig) -> Self {
        Self { config }
    }
}

impl<S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>>
    tracing_subscriber::Layer<S> for LogTracingLayer
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let max_level = &self.config.max_level;
        let meta = event.metadata();
        let target = meta.target();
        let file = meta.file();
        let line = meta.line();
        let level = *meta.level();
        // println!("{meta:?}");
        match max_level {
            MaxLevelType::List(list) => {
                let pass = list.iter().any(|(item_target, item_max_lvl)| {
                    target.contains(item_target) && level <= *item_max_lvl
                });
                if !pass {
                    return;
                }
            }
            MaxLevelType::Global(max_level) => {
                if level > *max_level {
                    return;
                }
            }
        }
        let colors = self.config.colors;

        let mut spans_combined = String::new();
        {
            let mut span_text: Vec<String> = Vec::new();
            let mut current_span = ctx.current_span().id().and_then(|id| ctx.span(id));

            while let Some(span) = current_span {
                let name = span.metadata().name();
                let extensions = span.extensions();
                let span_body = extensions.get::<SpanBody>();

                if let Some(span_body) = span_body {
                    span_text.push(format!("{}({})", &name, span_body.0));
                } else {
                    span_text.push(name.to_string());
                }

                current_span = span.parent();
            }

            if !span_text.is_empty() {
                spans_combined = span_text.iter().rev().fold(String::from(" "), |mut a, b| {
                    a += b;
                    a += " ";
                    a
                });
            }
        }

        let mut value = String::new();
        {
            let writer = Writer::new(&mut value);
            let mut visitor = PrettyVisitor::new(writer, true);
            event.record(&mut visitor);
        }

        let target = if self.config.target {
            format!(" {}", target)
        } else {
            "".to_string()
        };
        let origin = if self.config.line
            || level == tracing::Level::ERROR
            || level == tracing::Level::WARN
        {
            file.and_then(|file| line.map(|ln| format!(" {}:{}", file, ln)))
                .unwrap_or_default()
        } else {
            String::new()
        };

        match colors {
            ColorKind::Web => log5(
                format!("%c{level}%c{spans_combined}%c{target}{origin}%c: {value}"),
                match level {
                    tracing::Level::TRACE => "color: dodgerblue; background: #444",
                    tracing::Level::DEBUG => "color: lawngreen; background: #444",
                    tracing::Level::INFO => "color: whitesmoke; background: #444",
                    tracing::Level::WARN => "color: orange; background: #444",
                    tracing::Level::ERROR => "color: red; background: #444",
                },
                "color: inherit; font-weight: bold",
                "color: gray; font-style: italic",
                "color: inherit",
            ),
            ColorKind::Ascii => {
                let msg = format!(
                    "{}\x1b[1m{}\x1b[0m{}{}: {}",
                    match level {
                        tracing::Level::TRACE => "\x1b[96mTRACE\x1b[0m",
                        tracing::Level::DEBUG => "\x1b[95mDEBUG\x1b[0m",
                        tracing::Level::INFO => "\x1b[92mINFO\x1b[0m",
                        tracing::Level::WARN => "\x1b[93mWARN\x1b[0m",
                        tracing::Level::ERROR => "\x1b[91mERROR\x1b[0m",
                    },
                    spans_combined,
                    target,
                    origin,
                    value
                );
                println!("{}", msg);
            }
        }
    }

    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut span_body = String::new();
        let writer = Writer::new(&mut span_body);
        let mut visitor = PrettyVisitor::new(writer, true);
        attrs.record(&mut visitor);
        if !span_body.is_empty() {
            ctx.span(id)
                .unwrap()
                .extensions_mut()
                .insert(SpanBody(span_body));
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log1(message1: String);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log5(message1: String, message2: &str, message3: &str, message4: &str, message5: &str);
}

#[cfg(test)]
mod tests {
    use tracing::{trace, trace_span};

    use crate::prelude::*;

    #[test]
    fn test_logger() {
        init_log();

        let _span = trace_span!("one").entered();

        trace!("hello");
    }
}
