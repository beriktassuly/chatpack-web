use std::str::FromStr;

use chatpack::core::{
    apply_filters, merge_consecutive, FilterConfig, OutputConfig, ProcessingStats,
};
use chatpack::format::{to_format_string, OutputFormat};
use chatpack::parser::{create_parser, Platform};
use chatpack::Message;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct ConvertOptions {
    include_timestamps: bool,
    include_ids: bool,
    include_replies: bool,
    include_edited: bool,
    merge_consecutive: bool,
    filter_sender: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            include_timestamps: false,
            include_ids: false,
            include_replies: false,
            include_edited: false,
            merge_consecutive: true,
            filter_sender: None,
            date_from: None,
            date_to: None,
        }
    }
}

#[derive(Debug)]
struct PreparedMessages {
    messages: Vec<Message>,
    original_count: usize,
    filtered_count: usize,
    filters_active: bool,
    merged: bool,
}

#[derive(Serialize)]
struct ConversionReport {
    output: String,
    stats: ConversionStats,
}

#[derive(Serialize)]
struct ParseReport {
    messages: Vec<Message>,
    stats: ConversionStats,
}

#[derive(Serialize)]
struct ConversionStats {
    original_count: usize,
    filtered_count: usize,
    merged_count: usize,
    messages_saved: usize,
    compression_ratio: f64,
    merge_ratio: f64,
    input_bytes: usize,
    output_bytes: usize,
    filters_active: bool,
    merged: bool,
}

#[derive(Serialize)]
struct SupportedSource {
    id: &'static str,
    label: String,
    default_extension: &'static str,
}

#[derive(Serialize)]
struct SupportedFormat {
    id: &'static str,
    label: String,
    extension: &'static str,
    mime_type: &'static str,
}

/// Backwards-compatible conversion API.
#[wasm_bindgen]
pub fn convert(
    input: &str,
    source: &str,
    format: &str,
    include_timestamps: bool,
    include_replies: bool,
) -> std::result::Result<String, JsValue> {
    let options =
        ConvertOptions { include_timestamps, include_replies, ..ConvertOptions::default() };

    run_conversion(input, source, format, &options).map(|report| report.output).map_err(js_error)
}

/// Convert chat export with the full chatpack 0.7 option surface.
///
/// `options_json` accepts:
/// - `include_timestamps`, `include_ids`, `include_replies`, `include_edited`
/// - `merge_consecutive`
/// - `filter_sender`, `date_from`, `date_to`
#[wasm_bindgen]
pub fn convert_with_options(
    input: &str,
    source: &str,
    format: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    let options = parse_options(options_json).map_err(js_error)?;

    run_conversion(input, source, format, &options).map(|report| report.output).map_err(js_error)
}

/// Convert chat export and return a JSON report with output and processing stats.
#[wasm_bindgen]
pub fn convert_with_report(
    input: &str,
    source: &str,
    format: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    let options = parse_options(options_json).map_err(js_error)?;
    let report = run_conversion(input, source, format, &options).map_err(js_error)?;

    serde_json::to_string(&report).map_err(|e| js_error(e.to_string()))
}

/// Parse chat export and return normalized messages plus stats as JSON.
#[wasm_bindgen]
pub fn parse_chat(
    input: &str,
    source: &str,
    options_json: &str,
) -> std::result::Result<String, JsValue> {
    let options = parse_options(options_json).map_err(js_error)?;
    let prepared = prepare_messages(input, source, &options).map_err(js_error)?;
    let final_count = prepared.messages.len();
    let stats = build_stats(
        input,
        0,
        prepared.original_count,
        prepared.filtered_count,
        final_count,
        prepared.filters_active,
        prepared.merged,
    );
    let report = ParseReport { messages: prepared.messages, stats };

    serde_json::to_string(&report).map_err(|e| js_error(e.to_string()))
}

/// Return supported sources as JSON.
#[wasm_bindgen]
pub fn supported_sources() -> std::result::Result<String, JsValue> {
    let sources: Vec<SupportedSource> = Platform::all()
        .iter()
        .map(|platform| SupportedSource {
            id: platform_id(*platform),
            label: platform.to_string(),
            default_extension: platform.default_extension(),
        })
        .collect();

    serde_json::to_string(&sources).map_err(|e| js_error(e.to_string()))
}

/// Return supported output formats as JSON.
#[wasm_bindgen]
pub fn supported_formats() -> std::result::Result<String, JsValue> {
    let formats: Vec<SupportedFormat> = OutputFormat::all()
        .iter()
        .map(|format| SupportedFormat {
            id: format_id(*format),
            label: format.to_string(),
            extension: format.extension(),
            mime_type: format.mime_type(),
        })
        .collect();

    serde_json::to_string(&formats).map_err(|e| js_error(e.to_string()))
}

/// Get WASM binding version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn run_conversion(
    input: &str,
    source: &str,
    format: &str,
    options: &ConvertOptions,
) -> std::result::Result<ConversionReport, String> {
    let output_format = parse_format(format)?;
    let prepared = prepare_messages(input, source, options)?;
    let output_config = output_config(options);
    let output = to_format_string(&prepared.messages, output_format, &output_config)
        .map_err(|e| e.to_string())?;
    let stats = build_stats(
        input,
        output.len(),
        prepared.original_count,
        prepared.filtered_count,
        prepared.messages.len(),
        prepared.filters_active,
        prepared.merged,
    );

    Ok(ConversionReport { output, stats })
}

fn prepare_messages(
    input: &str,
    source: &str,
    options: &ConvertOptions,
) -> std::result::Result<PreparedMessages, String> {
    if input.trim().is_empty() {
        return Err("Empty input. File is empty or contains no data".to_string());
    }

    let platform = parse_platform(source)?;
    let parser = create_parser(platform);
    let mut messages = parser.parse_str(input).map_err(|e| e.to_string())?;
    let original_count = messages.len();
    let filter = filter_config(options)?;
    let filters_active = filter.is_active();

    if filters_active {
        messages = apply_filters(messages, &filter);
    }

    let filtered_count = messages.len();

    // IDs must remain one-to-one when the output includes reply references.
    let merged = options.merge_consecutive && !options.include_replies && !options.include_ids;
    if merged {
        messages = merge_consecutive(messages);
    }

    Ok(PreparedMessages { messages, original_count, filtered_count, filters_active, merged })
}

fn build_stats(
    input: &str,
    output_bytes: usize,
    original_count: usize,
    filtered_count: usize,
    merged_count: usize,
    filters_active: bool,
    merged: bool,
) -> ConversionStats {
    let mut stats = ProcessingStats::new(original_count, merged_count);
    if filters_active {
        stats = stats.with_filtered(filtered_count);
    }

    ConversionStats {
        original_count,
        filtered_count,
        merged_count,
        messages_saved: stats.messages_saved(),
        compression_ratio: stats.compression_ratio(),
        merge_ratio: stats.merge_ratio(),
        input_bytes: input.len(),
        output_bytes,
        filters_active,
        merged,
    }
}

fn output_config(options: &ConvertOptions) -> OutputConfig {
    let mut config = OutputConfig::new();

    if options.include_timestamps {
        config = config.with_timestamps();
    }
    if options.include_ids || options.include_replies {
        config = config.with_ids();
    }
    if options.include_replies {
        config = config.with_replies();
    }
    if options.include_edited {
        config = config.with_edited();
    }

    config
}

fn filter_config(options: &ConvertOptions) -> std::result::Result<FilterConfig, String> {
    let mut config = FilterConfig::new();

    if let Some(sender) = non_empty(options.filter_sender.as_deref()) {
        config = config.with_sender(sender);
    }
    if let Some(date_from) = non_empty(options.date_from.as_deref()) {
        config = config.with_date_from(date_from).map_err(|e| e.to_string())?;
    }
    if let Some(date_to) = non_empty(options.date_to.as_deref()) {
        config = config.with_date_to(date_to).map_err(|e| e.to_string())?;
    }

    Ok(config)
}

fn parse_options(options_json: &str) -> std::result::Result<ConvertOptions, String> {
    if options_json.trim().is_empty() {
        return Ok(ConvertOptions::default());
    }

    serde_json::from_str(options_json).map_err(|e| format!("Invalid options JSON: {e}"))
}

fn parse_platform(s: &str) -> std::result::Result<Platform, String> {
    Platform::from_str(s)
}

fn parse_format(s: &str) -> std::result::Result<OutputFormat, String> {
    OutputFormat::from_str(s)
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn platform_id(platform: Platform) -> &'static str {
    match platform {
        Platform::Telegram => "telegram",
        Platform::WhatsApp => "whatsapp",
        Platform::Instagram => "instagram",
        Platform::Discord => "discord",
        _ => "unknown",
    }
}

fn format_id(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Csv => "csv",
        OutputFormat::Json => "json",
        OutputFormat::Jsonl => "jsonl",
        _ => "unknown",
    }
}

fn js_error(message: String) -> JsValue {
    JsValue::from_str(&message)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TELEGRAM: &str = r#"{
  "messages": [
    {
      "id": 1,
      "type": "message",
      "date_unixtime": "1705314600",
      "from": "Alice",
      "text": "Hello"
    },
    {
      "id": 2,
      "type": "message",
      "date_unixtime": "1705314660",
      "from": "Bob",
      "text": "Reply",
      "reply_to_message_id": 1,
      "edited_unixtime": "1705314700"
    }
  ]
}"#;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_parse_platform() {
        assert!(parse_platform("tg").is_ok());
        assert!(parse_platform("whatsapp").is_ok());
        assert!(parse_platform("unknown").is_err());
    }

    #[test]
    fn test_parse_format() {
        assert!(parse_format("csv").is_ok());
        assert!(parse_format("json").is_ok());
        assert!(parse_format("ndjson").is_ok());
        assert!(parse_format("invalid").is_err());
    }

    #[test]
    fn test_convert_with_options_uses_core_output_config() {
        let options = r#"{
            "include_timestamps": true,
            "include_ids": true,
            "include_replies": true,
            "include_edited": true,
            "merge_consecutive": false
        }"#;
        let output = convert_with_options(TELEGRAM, "telegram", "csv", options)
            .map_err(|e| e.as_string().unwrap_or_default())
            .expect("conversion should succeed");

        assert!(output
            .contains("ID;Timestamp;Sender;Content;ReplyTo;ReplyToSender;ReplyToTopic;Edited"));
        assert!(output.contains("1;"));
        assert!(output.contains("2;"));
        assert!(output.contains("Bob;Reply;1;Alice;;"));
    }

    #[test]
    fn test_replies_keep_message_ids_even_when_merge_requested() {
        let input = r#"{"messages":[
            {"id":-999994266,"type":"service"},
            {"id":1,"type":"message","from":"Alice","text":"First"},
            {"id":2,"type":"message","from":"Alice","text":"Second"},
            {"id":3,"type":"message","from":"Bob","text":"Answer","reply_to_message_id":2}
        ]}"#;
        let options = ConvertOptions {
            include_replies: true,
            merge_consecutive: true,
            ..ConvertOptions::default()
        };
        let report = run_conversion(input, "telegram", "json", &options).unwrap();
        let messages: serde_json::Value = serde_json::from_str(&report.output).unwrap();

        assert_eq!(messages.as_array().unwrap().len(), 3);
        assert_eq!(messages[1]["id"], 2);
        assert_eq!(messages[2]["reply_to"], 2);
        assert_eq!(messages[2]["reply_to_sender"], "Alice");
        assert!(!report.stats.merged);
    }

    #[test]
    fn test_convert_with_report_returns_stats() {
        let report_json = convert_with_report(
            TELEGRAM,
            "telegram",
            "json",
            r#"{"filter_sender":"Alice","merge_consecutive":true}"#,
        )
        .map_err(|e| e.as_string().unwrap_or_default())
        .expect("report should serialize");
        let report: serde_json::Value =
            serde_json::from_str(&report_json).expect("report should be valid JSON");

        assert_eq!(report["stats"]["original_count"], 2);
        assert_eq!(report["stats"]["filtered_count"], 1);
        assert_eq!(report["stats"]["merged_count"], 1);
        assert!(report["stats"]["filters_active"].as_bool().unwrap());
        assert!(report["output"].as_str().unwrap().contains("Alice"));
    }

    #[test]
    fn test_parse_chat_returns_messages() {
        let report_json = parse_chat(TELEGRAM, "telegram", r#"{"merge_consecutive":false}"#)
            .map_err(|e| e.as_string().unwrap_or_default())
            .expect("parse should succeed");
        let report: serde_json::Value =
            serde_json::from_str(&report_json).expect("report should be valid JSON");

        assert_eq!(report["messages"].as_array().unwrap().len(), 2);
        assert_eq!(report["stats"]["merged_count"], 2);
    }

    #[test]
    fn test_supported_sources_and_formats() {
        let sources = supported_sources()
            .map_err(|e| e.as_string().unwrap_or_default())
            .expect("sources should serialize");
        let formats = supported_formats()
            .map_err(|e| e.as_string().unwrap_or_default())
            .expect("formats should serialize");

        assert!(sources.contains("telegram"));
        assert!(formats.contains("jsonl"));
        assert!(formats.contains("application/x-ndjson"));
    }
}
