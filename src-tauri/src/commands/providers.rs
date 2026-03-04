//! Usage: Provider configuration related Tauri commands.

use crate::app_state::{ensure_db_ready, DbInitState, GatewayState};
use crate::shared::mutex_ext::MutexExt;
use crate::{base_url_probe, blocking, providers};
use serde_json::json;
use std::path::{Path, PathBuf};
use tauri::Emitter;
use tauri::Manager;

const ENV_CLAUDE_DISABLE_NONESSENTIAL_TRAFFIC: &str = "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC";
const ENV_DISABLE_ERROR_REPORTING: &str = "DISABLE_ERROR_REPORTING";
const ENV_DISABLE_TELEMETRY: &str = "DISABLE_TELEMETRY";
const ENV_MCP_TIMEOUT: &str = "MCP_TIMEOUT";
const ENV_ANTHROPIC_BASE_URL: &str = "ANTHROPIC_BASE_URL";
const ENV_ANTHROPIC_AUTH_TOKEN: &str = "ANTHROPIC_AUTH_TOKEN";

#[tauri::command]
pub(crate) async fn providers_list(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    cli_key: String,
) -> Result<Vec<providers::ProviderSummary>, String> {
    let db = ensure_db_ready(app, db_state.inner()).await?;
    blocking::run("providers_list", move || {
        providers::list_by_cli(&db, &cli_key)
    })
    .await
    .map_err(Into::into)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn provider_upsert(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    provider_id: Option<i64>,
    cli_key: String,
    name: String,
    base_urls: Vec<String>,
    base_url_mode: String,
    api_key: Option<String>,
    enabled: bool,
    cost_multiplier: f64,
    priority: Option<i64>,
    claude_models: Option<providers::ClaudeModels>,
    limit_5h_usd: Option<f64>,
    limit_daily_usd: Option<f64>,
    daily_reset_mode: Option<String>,
    daily_reset_time: Option<String>,
    limit_weekly_usd: Option<f64>,
    limit_monthly_usd: Option<f64>,
    limit_total_usd: Option<f64>,
    tags: Option<Vec<String>>,
    note: Option<String>,
) -> Result<providers::ProviderSummary, String> {
    let is_create = provider_id.is_none();
    let name_for_log = name.clone();
    let cli_key_for_log = cli_key.clone();
    let db = ensure_db_ready(app, db_state.inner()).await?;
    let result = blocking::run("provider_upsert", move || {
        providers::upsert(
            &db,
            provider_id,
            &cli_key,
            &name,
            base_urls,
            &base_url_mode,
            api_key.as_deref(),
            enabled,
            cost_multiplier,
            priority,
            claude_models,
            limit_5h_usd,
            limit_daily_usd,
            daily_reset_mode.as_deref(),
            daily_reset_time.as_deref(),
            limit_weekly_usd,
            limit_monthly_usd,
            limit_total_usd,
            tags,
            note.as_deref(),
        )
    })
    .await
    .map_err(Into::into);

    if let Ok(ref provider) = result {
        if is_create {
            tracing::info!(
                provider_id = provider.id,
                provider_name = %name_for_log,
                cli_key = %cli_key_for_log,
                "provider created"
            );
        } else {
            tracing::info!(
                provider_id = provider.id,
                provider_name = %name_for_log,
                cli_key = %cli_key_for_log,
                "provider updated"
            );
        }
    }

    result
}

#[tauri::command]
pub(crate) async fn provider_set_enabled(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    provider_id: i64,
    enabled: bool,
) -> Result<providers::ProviderSummary, String> {
    let db = ensure_db_ready(app, db_state.inner()).await?;
    let result = blocking::run("provider_set_enabled", move || {
        providers::set_enabled(&db, provider_id, enabled)
    })
    .await
    .map_err(Into::into);

    if let Ok(ref provider) = result {
        tracing::info!(
            provider_id = provider.id,
            enabled = provider.enabled,
            "provider enabled state changed"
        );
    }

    result
}

#[tauri::command]
pub(crate) async fn provider_delete(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    provider_id: i64,
) -> Result<bool, String> {
    let db = ensure_db_ready(app, db_state.inner()).await?;
    let result = blocking::run(
        "provider_delete",
        move || -> crate::shared::error::AppResult<bool> {
            providers::delete(&db, provider_id)?;
            Ok(true)
        },
    )
    .await
    .map_err(Into::into);

    if let Ok(true) = result {
        tracing::info!(provider_id = provider_id, "provider deleted");
    }

    result
}

#[tauri::command]
pub(crate) async fn providers_reorder(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    gateway_state: tauri::State<'_, GatewayState>,
    cli_key: String,
    ordered_provider_ids: Vec<i64>,
) -> Result<Vec<providers::ProviderSummary>, String> {
    let cli_key_for_log = cli_key.clone();
    let db = ensure_db_ready(app, db_state.inner()).await?;
    let result = blocking::run("providers_reorder", move || {
        providers::reorder(&db, &cli_key, ordered_provider_ids)
    })
    .await
    .map_err(Into::into);

    if let Ok(ref providers) = result {
        // Provider order changes must invalidate session-bound provider_order (default TTL=300s).
        let cleared = {
            let manager = gateway_state.0.lock_or_recover();
            manager.clear_cli_session_bindings(&cli_key_for_log)
        };
        tracing::info!(
            cli_key = %cli_key_for_log,
            count = providers.len(),
            cleared_sessions = cleared,
            "providers reordered"
        );
    }

    result
}

#[tauri::command]
pub(crate) async fn provider_claude_terminal_launch_command(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    provider_id: i64,
) -> Result<String, String> {
    let db = ensure_db_ready(app.clone(), db_state.inner()).await?;
    let gateway_base_origin = blocking::run("provider_claude_terminal_launch_gateway_origin", {
        let app = app.clone();
        let db = db.clone();
        move || ensure_gateway_base_origin(&app, &db)
    })
    .await?;

    blocking::run("provider_claude_terminal_launch_command", move || {
        let launch = providers::claude_terminal_launch_context(&db, provider_id)?;
        let claude_base_url = build_claude_gateway_base_url(&gateway_base_origin, provider_id);
        create_claude_terminal_launch_command(
            provider_id,
            &claude_base_url,
            &launch.api_key_plaintext,
        )
    })
    .await
    .map_err(Into::into)
}

fn ensure_gateway_base_origin(
    app: &tauri::AppHandle,
    db: &crate::db::Db,
) -> crate::shared::error::AppResult<String> {
    let state = app.state::<GatewayState>();
    let mut manager = state.0.lock_or_recover();

    let mut status = manager.status();
    if !status.running {
        status = manager.start(app, db.clone(), None)?;
    }

    drop(manager);

    let _ = app.emit("gateway:status", status.clone());

    status
        .base_url
        .ok_or_else(|| "SYSTEM_ERROR: gateway base_url missing".to_string().into())
}

fn build_claude_gateway_base_url(gateway_base_origin: &str, provider_id: i64) -> String {
    format!(
        "{}/claude/_aio/provider/{provider_id}",
        gateway_base_origin.trim_end_matches('/')
    )
}

fn create_claude_terminal_launch_command(
    provider_id: i64,
    base_url: &str,
    api_key_plaintext: &str,
) -> crate::shared::error::AppResult<String> {
    let temp_dir = std::env::temp_dir();
    let now = crate::shared::time::now_unix_seconds();
    let pid = std::process::id();

    let config_path = temp_dir.join(format!("claude_{provider_id}_{pid}_{now}.json"));

    let settings_json = build_claude_settings_json(base_url, api_key_plaintext)?;
    std::fs::write(&config_path, settings_json)
        .map_err(|e| format!("SYSTEM_ERROR: write claude settings failed: {e}"))?;

    let (script_path, script_content, launch_command) =
        build_claude_launch_assets(provider_id, pid, now, &temp_dir, &config_path);
    if let Err(err) = std::fs::write(&script_path, script_content) {
        let _ = std::fs::remove_file(&config_path);
        return Err(format!("SYSTEM_ERROR: write launch script failed: {err}").into());
    }

    Ok(launch_command)
}

fn build_claude_launch_assets(
    provider_id: i64,
    pid: u32,
    now: i64,
    temp_dir: &Path,
    config_path: &Path,
) -> (PathBuf, String, String) {
    if cfg!(target_os = "windows") {
        let script_path =
            temp_dir.join(format!("aio_claude_launcher_{provider_id}_{pid}_{now}.ps1"));
        let script_content = build_claude_launcher_powershell_script(config_path, &script_path);
        let launch_command = build_powershell_launch_command(&script_path);
        (script_path, script_content, launch_command)
    } else {
        let script_path =
            temp_dir.join(format!("aio_claude_launcher_{provider_id}_{pid}_{now}.sh"));
        let script_content = build_claude_launcher_bash_script(config_path, &script_path);
        let launch_command = build_bash_launch_command(&script_path);
        (script_path, script_content, launch_command)
    }
}

fn build_claude_settings_json(
    base_url: &str,
    api_key_plaintext: &str,
) -> crate::shared::error::AppResult<String> {
    let value = json!({
        "env": {
            ENV_CLAUDE_DISABLE_NONESSENTIAL_TRAFFIC: "1",
            ENV_DISABLE_ERROR_REPORTING: "1",
            ENV_DISABLE_TELEMETRY: "1",
            ENV_MCP_TIMEOUT: "60000",
            ENV_ANTHROPIC_BASE_URL: base_url,
            ENV_ANTHROPIC_AUTH_TOKEN: api_key_plaintext,
        }
    });

    serde_json::to_string_pretty(&value)
        .map_err(|e| format!("SYSTEM_ERROR: serialize claude settings failed: {e}").into())
}

fn build_claude_launcher_bash_script(config_path: &Path, script_path: &Path) -> String {
    let config_var = bash_single_quote(&config_path.to_string_lossy());
    let script_var = bash_single_quote(&script_path.to_string_lossy());

    format!(
        "#!/bin/bash\n\
config_path={config_var}\n\
script_path={script_var}\n\
trap 'rm -f \"$config_path\" \"$script_path\"' EXIT\n\
echo \"Using provider-specific claude config:\"\n\
echo \"$config_path\"\n\
claude --settings \"$config_path\"\n\
exec bash --norc --noprofile\n"
    )
}

fn build_claude_launcher_powershell_script(config_path: &Path, script_path: &Path) -> String {
    let config_var = powershell_single_quote(&config_path.to_string_lossy());
    let script_var = powershell_single_quote(&script_path.to_string_lossy());

    format!(
        "$configPath = {config_var}\n\
$scriptPath = {script_var}\n\
try {{\n\
  Write-Output \"Using provider-specific claude config:\"\n\
  Write-Output $configPath\n\
  claude --settings $configPath\n\
}} finally {{\n\
  Remove-Item -LiteralPath $configPath -ErrorAction SilentlyContinue\n\
  Remove-Item -LiteralPath $scriptPath -ErrorAction SilentlyContinue\n\
}}\n"
    )
}

fn build_bash_launch_command(script_path: &Path) -> String {
    format!("bash {}", bash_single_quote(&script_path.to_string_lossy()))
}

fn build_powershell_launch_command(script_path: &Path) -> String {
    format!(
        "powershell -NoLogo -NoExit -ExecutionPolicy Bypass -File {}",
        windows_double_quote(&script_path.to_string_lossy())
    )
}

fn bash_single_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    format!("'{}'", value.replace('\'', r#"'"'"'"#))
}

fn powershell_single_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    format!("'{}'", value.replace('\'', "''"))
}

fn windows_double_quote(value: &str) -> String {
    format!("\"{value}\"")
}

#[tauri::command]
pub(crate) async fn provider_get_api_key(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    provider_id: i64,
) -> Result<String, String> {
    let db = ensure_db_ready(app, db_state.inner()).await?;
    blocking::run("provider_get_api_key", move || {
        providers::get_api_key_plaintext(&db, provider_id)
    })
    .await
    .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn base_url_ping_ms(base_url: String) -> Result<u64, String> {
    let client = reqwest::Client::builder()
        .user_agent(format!("aio-coding-hub-ping/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| format!("PING_HTTP_CLIENT_INIT: {e}"))?;
    base_url_probe::probe_base_url_ms(&client, &base_url, std::time::Duration::from_secs(3)).await
}

#[tauri::command]
pub(crate) async fn provider_stream_check(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    input: crate::infra::stream_check::ProviderStreamCheckInput,
) -> Result<crate::infra::stream_check::ProviderStreamCheckResult, String> {
    use crate::infra::stream_check;

    let model_raw = input.model.as_deref().unwrap_or("").trim();
    let needs_default_model = model_raw.is_empty() || model_raw.len() > 200;

    // Best-effort DB for dynamic default model selection.
    // Stream check should still work even if DB isn't ready (fallback models).
    let mut db_for_stream_check: Option<crate::db::Db> = if needs_default_model {
        ensure_db_ready(app.clone(), db_state.inner()).await.ok()
    } else {
        None
    };

    // Resolve API key: explicit > DB lookup > error
    let explicit_key = input
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let api_key = if let Some(key) = explicit_key {
        key
    } else if let Some(provider_id) = input.provider_id {
        let db = match db_for_stream_check.clone() {
            Some(db) => db,
            None => ensure_db_ready(app, db_state.inner()).await?,
        };
        db_for_stream_check = Some(db.clone());

        let cli_key_for_check = input.cli_key.clone();
        let (stored_cli_key, stored_key) = blocking::run(
            "provider_stream_check_get_key",
            move || -> crate::shared::error::AppResult<(String, String)> {
                use rusqlite::OptionalExtension;
                let conn = db.open_connection()?;
                let row: Option<(String, Option<String>)> = conn
                    .query_row(
                        "SELECT cli_key, api_key_plaintext FROM providers WHERE id = ?1",
                        rusqlite::params![provider_id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()
                    .map_err(|e| crate::shared::error::db_err!("failed to query provider: {e}"))?;

                let (cli_key, api_key) = row.ok_or_else(|| {
                    crate::shared::error::AppError::new("DB_NOT_FOUND", "provider not found")
                })?;

                if cli_key != cli_key_for_check {
                    return Err(crate::shared::error::AppError::new(
                        "SEC_INVALID_INPUT",
                        "cli_key mismatch with stored provider",
                    ));
                }

                Ok((cli_key, api_key.unwrap_or_default()))
            },
        )
        .await
        .map_err(|e| e.to_string())?;

        let _ = stored_cli_key;
        if stored_key.trim().is_empty() {
            return Ok(stream_check::ProviderStreamCheckResult {
                ok: false,
                grade: "failed".to_string(),
                duration_ms: 0,
                http_status: None,
                target_url: String::new(),
                used_model: String::new(),
                failure_kind: Some("auth".to_string()),
                message: Some("该 Provider 未配置 API Key".to_string()),
                attempts: 0,
            });
        }
        stored_key
    } else {
        return Ok(stream_check::ProviderStreamCheckResult {
            ok: false,
            grade: "failed".to_string(),
            duration_ms: 0,
            http_status: None,
            target_url: String::new(),
            used_model: String::new(),
            failure_kind: Some("auth".to_string()),
            message: Some("API Key 未提供".to_string()),
            attempts: 0,
        });
    };

    let client = reqwest::Client::builder()
        .user_agent(format!(
            "aio-coding-hub-stream-check/{}",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .map_err(|e| format!("STREAM_CHECK_HTTP_CLIENT_INIT: {e}"))?;

    stream_check::stream_check(&client, db_for_stream_check.as_ref(), &input, &api_key)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_single_quote_escapes_single_quote() {
        assert_eq!(bash_single_quote("a'b"), "'a'\"'\"'b'");
    }

    #[test]
    fn powershell_single_quote_escapes_single_quote() {
        assert_eq!(powershell_single_quote("a'b"), "'a''b'");
    }

    #[test]
    fn build_settings_contains_required_envs() {
        let json_text = build_claude_settings_json("https://example.com", "sk-test").unwrap();
        let value: serde_json::Value = serde_json::from_str(&json_text).unwrap();
        let env = value
            .get("env")
            .and_then(|v| v.as_object())
            .expect("env object");

        assert_eq!(
            env.get(ENV_CLAUDE_DISABLE_NONESSENTIAL_TRAFFIC)
                .and_then(|v| v.as_str()),
            Some("1")
        );
        assert_eq!(
            env.get(ENV_DISABLE_ERROR_REPORTING)
                .and_then(|v| v.as_str()),
            Some("1")
        );
        assert_eq!(
            env.get(ENV_DISABLE_TELEMETRY).and_then(|v| v.as_str()),
            Some("1")
        );
        assert_eq!(
            env.get(ENV_MCP_TIMEOUT).and_then(|v| v.as_str()),
            Some("60000")
        );
        assert_eq!(
            env.get(ENV_ANTHROPIC_BASE_URL).and_then(|v| v.as_str()),
            Some("https://example.com")
        );
        assert_eq!(
            env.get(ENV_ANTHROPIC_AUTH_TOKEN).and_then(|v| v.as_str()),
            Some("sk-test")
        );
    }

    #[test]
    fn build_claude_gateway_base_url_trims_trailing_slash() {
        let url = build_claude_gateway_base_url("http://127.0.0.1:18080/", 12);
        assert_eq!(url, "http://127.0.0.1:18080/claude/_aio/provider/12");
    }

    #[test]
    fn bash_launch_script_includes_cleanup_and_claude_settings() {
        let config_path = Path::new("/tmp/claude_x.json");
        let script_path = Path::new("/tmp/aio_launcher.sh");
        let script = build_claude_launcher_bash_script(config_path, script_path);

        assert!(script.contains("trap 'rm -f \"$config_path\" \"$script_path\"' EXIT"));
        assert!(script.contains("claude --settings \"$config_path\""));
        assert!(script.contains("exec bash --norc --noprofile"));
    }

    #[test]
    fn powershell_launch_script_includes_cleanup_and_claude_settings() {
        let config_path = Path::new(r"C:\\Temp\\claude_x.json");
        let script_path = Path::new(r"C:\\Temp\\aio_launcher.ps1");
        let script = build_claude_launcher_powershell_script(config_path, script_path);

        assert!(script.contains("Write-Output \"Using provider-specific claude config:\""));
        assert!(script.contains("claude --settings $configPath"));
        assert!(
            script.contains("Remove-Item -LiteralPath $configPath -ErrorAction SilentlyContinue")
        );
        assert!(
            script.contains("Remove-Item -LiteralPath $scriptPath -ErrorAction SilentlyContinue")
        );
    }

    #[test]
    fn powershell_launch_command_uses_expected_flags() {
        let script_path = Path::new(r"C:\\Temp\\aio_launcher.ps1");
        let command = build_powershell_launch_command(script_path);

        assert!(command.starts_with("powershell -NoLogo -NoExit -ExecutionPolicy Bypass -File"));
        assert!(command.contains("\"C:\\\\Temp\\\\aio_launcher.ps1\""));
    }
}
