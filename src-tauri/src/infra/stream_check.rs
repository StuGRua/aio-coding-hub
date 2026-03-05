//! Usage: Provider stream check — validates Provider availability via minimal streaming API request.

use crate::shared::error::{AppError, AppResult};
use futures_core::Stream;
use reqwest::header::{HeaderMap, HeaderValue};
use std::task::Context;
use std::time::{Duration, Instant};

// ── IPC Contract ────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct ProviderStreamCheckInput {
    pub cli_key: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub provider_id: Option<i64>,
    pub model: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(serde::Serialize)]
pub struct ProviderStreamCheckResult {
    pub ok: bool,
    pub grade: String,
    pub duration_ms: u64,
    pub http_status: Option<u16>,
    pub target_url: String,
    pub used_model: String,
    pub failure_kind: Option<String>,
    pub message: Option<String>,
    pub attempts: u8,
}

// ── Defaults ────────────────────────────────────────────────────────

const FALLBACK_MODELS: &[(&str, &str)] = &[
    ("claude", "claude-haiku-4-5-latest"),
    ("codex", "gpt-4.1-mini"),
    ("gemini", "gemini-2.0-flash"),
];

fn fallback_model(cli_key: &str) -> &'static str {
    FALLBACK_MODELS
        .iter()
        .find(|(k, _)| *k == cli_key)
        .map(|(_, v)| *v)
        .unwrap_or("gpt-4.1-mini")
}

fn parse_cost_f64(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn dynamic_default_model(db: &crate::db::Db, cli_key: &str) -> Option<String> {
    use rusqlite::params;

    let conn = db.open_connection().ok()?;
    let mut stmt = conn
        .prepare_cached(
            r#"
SELECT model, price_json, updated_at
FROM model_prices
WHERE cli_key = ?1
"#,
        )
        .ok()?;

    let mut rows = stmt.query(params![cli_key]).ok()?;

    let mut best: Option<(f64, i64, String)> = None;
    while let Ok(Some(row)) = rows.next() {
        let model: String = match row.get(0) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let price_json: String = match row.get(1) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let updated_at: i64 = match row.get(2) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let parsed: serde_json::Value = match serde_json::from_str(&price_json) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let obj = match parsed.as_object() {
            Some(v) => v,
            None => continue,
        };

        let cost = obj
            .get("input_cost_per_token")
            .or_else(|| obj.get("input_cost_per_cached_token"))
            .and_then(parse_cost_f64);
        let Some(cost) = cost else {
            continue;
        };
        if !cost.is_finite() || cost <= 0.0 {
            continue;
        }

        match &best {
            None => best = Some((cost, updated_at, model)),
            Some((best_cost, best_updated_at, _)) => {
                let cheaper = cost < (*best_cost - 1e-18);
                let same_price = (cost - *best_cost).abs() <= 1e-18;
                let newer = updated_at > *best_updated_at;
                if cheaper || (same_price && newer) {
                    best = Some((cost, updated_at, model));
                }
            }
        }
    }

    best.map(|(_, _, model)| model)
}

fn default_model(db: Option<&crate::db::Db>, cli_key: &str) -> String {
    if let Some(db) = db {
        if let Some(model) = dynamic_default_model(db, cli_key) {
            return model;
        }
    }
    fallback_model(cli_key).to_string()
}

// ── URL Construction ────────────────────────────────────────────────

fn build_stream_check_url(
    cli_key: &str,
    base_url: &str,
    model: &str,
) -> Result<reqwest::Url, String> {
    let mut url = reqwest::Url::parse(base_url).map_err(|e| format!("INVALID_BASE_URL: {e}"))?;

    let forwarded_path = match cli_key {
        "claude" => "/v1/messages".to_string(),
        "codex" => "/v1/chat/completions".to_string(),
        "gemini" => format!("/v1beta/models/{model}:streamGenerateContent"),
        _ => "/v1/chat/completions".to_string(),
    };

    let base_path = url.path().trim_end_matches('/').to_string();
    let adjusted = if base_path.ends_with("/v1")
        && (forwarded_path == "/v1" || forwarded_path.starts_with("/v1/"))
    {
        forwarded_path
            .strip_prefix("/v1")
            .unwrap_or(&forwarded_path)
            .to_string()
    } else if base_path.ends_with("/v1beta")
        && (forwarded_path == "/v1beta" || forwarded_path.starts_with("/v1beta/"))
    {
        forwarded_path
            .strip_prefix("/v1beta")
            .unwrap_or(&forwarded_path)
            .to_string()
    } else {
        forwarded_path
    };

    let mut combined = String::new();
    combined.push_str(&base_path);
    combined.push_str(&adjusted);
    if combined.is_empty() {
        combined.push('/');
    }
    if !combined.starts_with('/') {
        combined.insert(0, '/');
    }

    url.set_path(&combined);

    if cli_key == "gemini" {
        url.set_query(Some("alt=sse"));
    }

    Ok(url)
}

// ── Auth Injection ──────────────────────────────────────────────────

fn inject_auth(cli_key: &str, api_key: &str, headers: &mut HeaderMap) {
    match cli_key {
        "codex" => {
            let value = format!("Bearer {api_key}");
            if let Ok(hv) = HeaderValue::from_str(&value) {
                headers.insert(reqwest::header::AUTHORIZATION, hv);
            }
        }
        "claude" => {
            let value = format!("Bearer {api_key}");
            if let Ok(hv) = HeaderValue::from_str(&value) {
                headers.insert(reqwest::header::AUTHORIZATION, hv);
            }
            if let Ok(hv) = HeaderValue::from_str(api_key) {
                headers.insert("x-api-key", hv);
            }
            headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        }
        "gemini" => {
            let trimmed = api_key.trim();
            let oauth_token = if trimmed.starts_with("ya29.") {
                Some(trimmed.to_string())
            } else if trimmed.starts_with('{') {
                serde_json::from_str::<serde_json::Value>(trimmed)
                    .ok()
                    .and_then(|v| v.get("access_token")?.as_str().map(str::to_string))
            } else {
                None
            };

            if let Some(token) = oauth_token {
                let value = format!("Bearer {token}");
                if let Ok(hv) = HeaderValue::from_str(&value) {
                    headers.insert(reqwest::header::AUTHORIZATION, hv);
                }
            } else if let Ok(hv) = HeaderValue::from_str(trimmed) {
                headers.insert("x-goog-api-key", hv);
            }
        }
        _ => {}
    }
}

// ── Request Body ────────────────────────────────────────────────────

fn build_request_body(cli_key: &str, model: &str, prompt: &str) -> serde_json::Value {
    match cli_key {
        "gemini" => serde_json::json!({
            "contents": [{"role": "user", "parts": [{"text": prompt}]}],
            "generationConfig": {"maxOutputTokens": 1}
        }),
        "claude" => serde_json::json!({
            "model": model,
            "max_tokens": 1,
            "stream": true,
            "messages": [{"role": "user", "content": [{"type": "text", "text": prompt}]}]
        }),
        _ => serde_json::json!({
            "model": model,
            "max_tokens": 1,
            "stream": true,
            "messages": [{"role": "user", "content": prompt}]
        }),
    }
}

// ── Error Classification ────────────────────────────────────────────

fn classify_error(reqwest_err: Option<&reqwest::Error>, status: Option<u16>, body: &str) -> String {
    // 1. Transport layer
    if let Some(err) = reqwest_err {
        if err.is_timeout() {
            return "timeout".to_string();
        }
        if err.is_connect() {
            return "network".to_string();
        }
    }

    // 2. HTTP status layer
    if let Some(code) = status {
        match code {
            401 | 403 => return "auth".to_string(),
            404 => return "model".to_string(),
            429 => return "rate_limit".to_string(),
            408 | 504 => return "timeout".to_string(),
            400..=599 => {
                // Fall through to body inspection for 4xx/5xx
            }
            _ => {}
        }
    }

    // 3. Response body keywords
    let lower = body.to_ascii_lowercase();
    if lower.contains("authentication")
        || lower.contains("invalid") && lower.contains("key")
        || lower.contains("permission")
    {
        return "auth".to_string();
    }
    if lower.contains("model")
        && (lower.contains("not found")
            || lower.contains("not available")
            || lower.contains("does not exist"))
    {
        return "model".to_string();
    }
    if lower.contains("rate") || lower.contains("quota") || lower.contains("limit") {
        return "rate_limit".to_string();
    }

    // 4. Remaining 4xx/5xx without body match
    if let Some(code) = status {
        if (400..600).contains(&code) {
            return "server".to_string();
        }
    }

    "unknown".to_string()
}

fn truncate_message(msg: &str, max_bytes: usize) -> String {
    if msg.len() <= max_bytes {
        return msg.to_string();
    }
    // Manual floor_char_boundary: walk back from max_bytes to find a valid UTF-8 boundary
    let mut end = max_bytes;
    while end > 0 && !msg.is_char_boundary(end) {
        end -= 1;
    }
    msg[..end].to_string()
}

// ── Core Stream Check ───────────────────────────────────────────────

pub async fn stream_check(
    client: &reqwest::Client,
    db: Option<&crate::db::Db>,
    input: &ProviderStreamCheckInput,
    api_key: &str,
) -> AppResult<ProviderStreamCheckResult> {
    let model_raw = input.model.as_deref().unwrap_or("").trim();
    let model = if model_raw.is_empty() || model_raw.len() > 200 {
        default_model(db, &input.cli_key)
    } else {
        model_raw.to_string()
    };

    let timeout_ms = input.timeout_ms.unwrap_or(10_000).clamp(1_000, 60_000);
    let timeout = Duration::from_millis(timeout_ms);

    let target_url = build_stream_check_url(&input.cli_key, &input.base_url, &model)
        .map_err(|e| AppError::new("STREAM_CHECK", e))?;
    let body = build_request_body(&input.cli_key, &model, "ping");

    let mut headers = HeaderMap::new();
    inject_auth(&input.cli_key, api_key, &mut headers);
    headers.insert("content-type", HeaderValue::from_static("application/json"));

    let start = Instant::now();
    let max_attempts: u8 = 3;
    let retry_delay = Duration::from_millis(500);

    let mut last_status: Option<u16> = None;
    let mut last_message = String::new();
    let mut last_failure_kind = "unknown".to_string();
    let mut attempts: u8 = 0;

    for attempt in 0..max_attempts {
        attempts = attempt + 1;

        let result = client
            .post(target_url.as_str())
            .headers(headers.clone())
            .json(&body)
            .timeout(timeout)
            .send()
            .await;

        match result {
            Ok(resp) => {
                let status_code = resp.status().as_u16();
                last_status = Some(status_code);

                if !resp.status().is_success() {
                    let resp_body = resp.text().await.unwrap_or_default();
                    last_failure_kind = classify_error(None, Some(status_code), &resp_body);
                    last_message = truncate_message(&resp_body, 512);
                    // Non-2xx: do not retry
                    break;
                }

                // 2xx — try to read first chunk
                let stream = resp.bytes_stream();
                tokio::pin!(stream);
                let next_chunk =
                    std::future::poll_fn(|cx: &mut Context<'_>| stream.as_mut().poll_next(cx));
                match tokio::time::timeout(timeout, next_chunk).await {
                    Ok(Some(Ok(bytes))) if !bytes.is_empty() => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        let grade = if duration_ms <= 6000 {
                            "operational"
                        } else {
                            "degraded"
                        };
                        return Ok(ProviderStreamCheckResult {
                            ok: true,
                            grade: grade.to_string(),
                            duration_ms,
                            http_status: Some(status_code),
                            target_url: target_url.to_string(),
                            used_model: model.clone(),
                            failure_kind: None,
                            message: None,
                            attempts,
                        });
                    }
                    Ok(Some(Ok(_))) | Ok(None) => {
                        // Empty stream or immediate EOF
                        last_failure_kind = "server".to_string();
                        last_message = "empty stream response".to_string();
                        break;
                    }
                    Ok(Some(Err(e))) => {
                        last_failure_kind = classify_error(Some(&e), Some(status_code), "");
                        last_message = truncate_message(&e.to_string(), 512);
                        break;
                    }
                    Err(_) => {
                        // Timeout reading first chunk
                        last_failure_kind = "timeout".to_string();
                        last_message = "timeout waiting for first chunk".to_string();
                        if attempt + 1 < max_attempts {
                            tokio::time::sleep(retry_delay).await;
                            continue;
                        }
                        break;
                    }
                }
            }
            Err(e) => {
                let retryable = e.is_timeout() || e.is_connect();
                last_failure_kind = classify_error(Some(&e), None, "");
                last_message = truncate_message(&e.to_string(), 512);

                if retryable && attempt + 1 < max_attempts {
                    tokio::time::sleep(retry_delay).await;
                    continue;
                }
                break;
            }
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    Ok(ProviderStreamCheckResult {
        ok: false,
        grade: "failed".to_string(),
        duration_ms,
        http_status: last_status,
        target_url: target_url.to_string(),
        used_model: model.to_string(),
        failure_kind: Some(last_failure_kind),
        message: Some(last_message),
        attempts,
    })
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── build_request_body ──

    #[test]
    fn build_request_body_claude_uses_content_blocks() {
        let body = build_request_body("claude", "claude-haiku-4-5-latest", "ping");
        let messages = body
            .get("messages")
            .and_then(|v| v.as_array())
            .expect("messages should be an array");
        let first = messages
            .first()
            .and_then(|v| v.as_object())
            .expect("first message should be an object");
        let content = first
            .get("content")
            .and_then(|v| v.as_array())
            .expect("claude message content should be an array of blocks");
        let first_block = content
            .first()
            .and_then(|v| v.as_object())
            .expect("first content block should be an object");
        assert_eq!(
            first_block.get("type"),
            Some(&serde_json::Value::String("text".to_string()))
        );
        assert_eq!(
            first_block.get("text"),
            Some(&serde_json::Value::String("ping".to_string()))
        );
    }

    #[test]
    fn build_request_body_chat_completions_uses_string_content() {
        let body = build_request_body("codex", "gpt-4.1-mini", "ping");
        let messages = body
            .get("messages")
            .and_then(|v| v.as_array())
            .expect("messages should be an array");
        let first = messages
            .first()
            .and_then(|v| v.as_object())
            .expect("first message should be an object");
        let content = first
            .get("content")
            .and_then(|v| v.as_str())
            .expect("chat-completions content should be a string");
        assert_eq!(content, "ping");
    }

    // ── classify_error ──

    #[test]
    fn classify_401_as_auth() {
        assert_eq!(classify_error(None, Some(401), ""), "auth");
    }

    #[test]
    fn classify_403_as_auth() {
        assert_eq!(classify_error(None, Some(403), ""), "auth");
    }

    #[test]
    fn classify_404_as_model() {
        assert_eq!(classify_error(None, Some(404), ""), "model");
    }

    #[test]
    fn classify_429_as_rate_limit() {
        assert_eq!(classify_error(None, Some(429), ""), "rate_limit");
    }

    #[test]
    fn classify_500_as_server() {
        assert_eq!(classify_error(None, Some(500), ""), "server");
    }

    #[test]
    fn classify_200_with_authentication_body_as_auth() {
        assert_eq!(
            classify_error(None, Some(200), "authentication_error: invalid key"),
            "auth"
        );
    }

    #[test]
    fn classify_400_with_model_not_found_body() {
        assert_eq!(
            classify_error(None, Some(400), "the model was not found in registry"),
            "model"
        );
    }

    #[test]
    fn classify_400_with_rate_body() {
        assert_eq!(
            classify_error(None, Some(400), "rate limit exceeded"),
            "rate_limit"
        );
    }

    // ── build_stream_check_url ──

    #[test]
    fn url_claude_no_double_v1() {
        let url = build_stream_check_url("claude", "https://a.com/v1", "m").unwrap();
        assert!(!url.path().contains("/v1/v1"));
        assert!(url.path().ends_with("/v1/messages"));
    }

    #[test]
    fn url_claude_without_v1() {
        let url = build_stream_check_url("claude", "https://a.com", "m").unwrap();
        assert!(url.path().contains("/v1/messages"));
    }

    #[test]
    fn url_gemini_no_double_v1beta() {
        let url = build_stream_check_url("gemini", "https://a.com/v1beta", "flash").unwrap();
        assert!(!url.path().contains("/v1beta/v1beta"));
        assert!(url.path().contains("flash:streamGenerateContent"));
        assert_eq!(url.query(), Some("alt=sse"));
    }

    #[test]
    fn url_codex_appends_chat_completions() {
        let url = build_stream_check_url("codex", "https://api.openai.com", "gpt-4").unwrap();
        assert!(url.path().contains("/v1/chat/completions"));
    }

    #[test]
    fn url_codex_with_v1_no_double() {
        let url = build_stream_check_url("codex", "https://api.openai.com/v1", "gpt-4").unwrap();
        assert!(!url.path().contains("/v1/v1"));
        assert!(url.path().ends_with("/v1/chat/completions"));
    }

    // ── model validation ──

    #[test]
    fn empty_model_falls_back_to_default() {
        assert_eq!(
            default_model(None, "claude"),
            "claude-haiku-4-5-latest".to_string()
        );
        assert_eq!(default_model(None, "codex"), "gpt-4.1-mini".to_string());
        assert_eq!(
            default_model(None, "gemini"),
            "gemini-2.0-flash".to_string()
        );
    }

    // ── truncate_message ──

    #[test]
    fn truncate_within_limit() {
        let msg = "short";
        assert_eq!(truncate_message(msg, 512), "short");
    }

    #[test]
    fn truncate_exceeds_limit() {
        let msg = "a".repeat(600);
        let result = truncate_message(&msg, 512);
        assert!(result.len() <= 512);
    }
}
