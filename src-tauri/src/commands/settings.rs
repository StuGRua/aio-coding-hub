//! Usage: Settings-related Tauri commands.

use crate::{blocking, resident, settings};
use tauri::Manager;

#[cfg(desktop)]
fn desired_autostart_args(tray_enabled: bool, silent_startup: bool) -> Vec<&'static str> {
    if tray_enabled && silent_startup {
        vec!["--autostart", "--silent-startup"]
    } else {
        vec!["--autostart"]
    }
}

#[cfg(desktop)]
fn sync_autostart(
    app: &tauri::AppHandle,
    auto_start: bool,
    tray_enabled: bool,
    silent_startup: bool,
) -> Result<(), String> {
    use auto_launch::AutoLaunchBuilder;

    let package_name = &app.package_info().name;
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("failed to resolve current executable for autostart: {e}"))?;
    let args = desired_autostart_args(tray_enabled, silent_startup);

    let mut builder = AutoLaunchBuilder::new();
    builder.set_app_name(package_name);
    builder.set_args(&args);

    #[cfg(windows)]
    builder.set_app_path(&current_exe.display().to_string());

    #[cfg(target_os = "macos")]
    {
        let exe_path = current_exe
            .canonicalize()
            .map_err(|e| format!("failed to canonicalize executable for autostart: {e}"))?
            .display()
            .to_string();
        let parts: Vec<&str> = exe_path.split(".app/").collect();
        let app_path = if parts.len() == 2 {
            format!("{}.app", parts.first().unwrap_or(&""))
        } else {
            exe_path
        };
        builder.set_use_launch_agent(true);
        builder.set_app_path(&app_path);
    }

    #[cfg(target_os = "linux")]
    if let Some(appimage) = app
        .env()
        .appimage
        .and_then(|p| p.to_str().map(|s| s.to_string()))
    {
        builder.set_app_path(&appimage);
    } else {
        builder.set_app_path(&current_exe.display().to_string());
    }

    let autolaunch = builder
        .build()
        .map_err(|e| format!("failed to build autostart entry: {e}"))?;

    if auto_start {
        autolaunch.disable().ok();
        autolaunch
            .enable()
            .map_err(|e| format!("failed to enable autostart: {e}"))
    } else {
        autolaunch
            .disable()
            .map_err(|e| format!("failed to disable autostart: {e}"))
    }
}

/// Encapsulates all fields for the `settings_set` command.
#[derive(serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SettingsUpdate {
    pub preferred_port: u16,
    pub gateway_listen_mode: Option<settings::GatewayListenMode>,
    pub gateway_custom_listen_address: Option<String>,
    pub auto_start: bool,
    pub tray_enabled: Option<bool>,
    pub silent_startup: Option<bool>,
    pub enable_cli_proxy_startup_recovery: Option<bool>,
    pub log_retention_days: u32,
    pub provider_cooldown_seconds: Option<u32>,
    pub provider_base_url_ping_cache_ttl_seconds: Option<u32>,
    pub upstream_first_byte_timeout_seconds: Option<u32>,
    pub upstream_stream_idle_timeout_seconds: Option<u32>,
    pub upstream_request_timeout_non_streaming_seconds: Option<u32>,
    pub intercept_anthropic_warmup_requests: Option<bool>,
    pub enable_thinking_signature_rectifier: Option<bool>,
    pub enable_thinking_budget_rectifier: Option<bool>,
    pub enable_claude_metadata_user_id_injection: Option<bool>,
    pub enable_cache_anomaly_monitor: Option<bool>,
    pub enable_task_complete_notify: Option<bool>,
    pub enable_response_fixer: Option<bool>,
    pub response_fixer_fix_encoding: Option<bool>,
    pub response_fixer_fix_sse_format: Option<bool>,
    pub response_fixer_fix_truncated_json: Option<bool>,
    pub verbose_provider_error: Option<bool>,
    pub failover_max_attempts_per_provider: u32,
    pub failover_max_providers_to_try: u32,
    pub circuit_breaker_failure_threshold: Option<u32>,
    pub circuit_breaker_open_duration_minutes: Option<u32>,
    pub update_releases_url: Option<String>,
    pub wsl_auto_config: Option<bool>,
    pub wsl_target_cli: Option<settings::WslTargetCli>,
    pub wsl_host_address_mode: Option<settings::WslHostAddressMode>,
    pub wsl_custom_host_address: Option<String>,
}

#[cfg(all(test, desktop))]
mod autostart_arg_tests {
    use super::desired_autostart_args;

    #[test]
    fn adds_silent_flag_only_when_tray_and_silent_are_enabled() {
        assert_eq!(
            desired_autostart_args(true, true),
            vec!["--autostart", "--silent-startup"]
        );
        assert_eq!(desired_autostart_args(true, false), vec!["--autostart"]);
        assert_eq!(desired_autostart_args(false, true), vec!["--autostart"]);
    }
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn settings_get(app: tauri::AppHandle) -> Result<settings::AppSettings, String> {
    blocking::run("settings_get", move || settings::read(&app))
        .await
        .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn settings_set(
    app: tauri::AppHandle,
    update: SettingsUpdate,
) -> Result<settings::AppSettings, String> {
    let SettingsUpdate {
        preferred_port,
        gateway_listen_mode,
        gateway_custom_listen_address,
        auto_start,
        tray_enabled,
        silent_startup,
        enable_cli_proxy_startup_recovery,
        log_retention_days,
        provider_cooldown_seconds,
        provider_base_url_ping_cache_ttl_seconds,
        upstream_first_byte_timeout_seconds,
        upstream_stream_idle_timeout_seconds,
        upstream_request_timeout_non_streaming_seconds,
        intercept_anthropic_warmup_requests,
        enable_thinking_signature_rectifier,
        enable_thinking_budget_rectifier,
        enable_claude_metadata_user_id_injection,
        enable_cache_anomaly_monitor,
        enable_task_complete_notify,
        enable_response_fixer,
        response_fixer_fix_encoding,
        response_fixer_fix_sse_format,
        response_fixer_fix_truncated_json,
        verbose_provider_error,
        failover_max_attempts_per_provider,
        failover_max_providers_to_try,
        circuit_breaker_failure_threshold,
        circuit_breaker_open_duration_minutes,
        update_releases_url,
        wsl_auto_config,
        wsl_target_cli,
        wsl_host_address_mode,
        wsl_custom_host_address,
    } = update;

    let app_for_work = app.clone();
    let next_settings = blocking::run(
        "settings_set",
        move || -> crate::shared::error::AppResult<settings::AppSettings> {
            let previous = settings::read(&app_for_work).unwrap_or_default();
            let update_releases_url = update_releases_url.unwrap_or(previous.update_releases_url);
            let tray_enabled = tray_enabled.unwrap_or(previous.tray_enabled);
            let silent_startup = silent_startup.unwrap_or(previous.silent_startup);
            let enable_cli_proxy_startup_recovery = enable_cli_proxy_startup_recovery
                .unwrap_or(previous.enable_cli_proxy_startup_recovery);
            let provider_cooldown_seconds =
                provider_cooldown_seconds.unwrap_or(previous.provider_cooldown_seconds);
            let gateway_listen_mode = gateway_listen_mode.unwrap_or(previous.gateway_listen_mode);
            let gateway_custom_listen_address = gateway_custom_listen_address
                .unwrap_or(previous.gateway_custom_listen_address)
                .trim()
                .to_string();
            let wsl_auto_config = wsl_auto_config.unwrap_or(previous.wsl_auto_config);
            let wsl_target_cli = wsl_target_cli.unwrap_or(previous.wsl_target_cli);
            let wsl_host_address_mode =
                wsl_host_address_mode.unwrap_or(previous.wsl_host_address_mode);
            let wsl_custom_host_address = wsl_custom_host_address
                .unwrap_or(previous.wsl_custom_host_address)
                .trim()
                .to_string();
            let provider_base_url_ping_cache_ttl_seconds = provider_base_url_ping_cache_ttl_seconds
                .unwrap_or(previous.provider_base_url_ping_cache_ttl_seconds);
            let upstream_first_byte_timeout_seconds = upstream_first_byte_timeout_seconds
                .unwrap_or(previous.upstream_first_byte_timeout_seconds);
            let upstream_stream_idle_timeout_seconds = upstream_stream_idle_timeout_seconds
                .unwrap_or(previous.upstream_stream_idle_timeout_seconds);
            let upstream_request_timeout_non_streaming_seconds =
                upstream_request_timeout_non_streaming_seconds
                    .unwrap_or(previous.upstream_request_timeout_non_streaming_seconds);
            let intercept_anthropic_warmup_requests = intercept_anthropic_warmup_requests
                .unwrap_or(previous.intercept_anthropic_warmup_requests);
            let enable_thinking_signature_rectifier = enable_thinking_signature_rectifier
                .unwrap_or(previous.enable_thinking_signature_rectifier);
            let enable_thinking_budget_rectifier = enable_thinking_budget_rectifier
                .unwrap_or(previous.enable_thinking_budget_rectifier);
            let enable_claude_metadata_user_id_injection = enable_claude_metadata_user_id_injection
                .unwrap_or(previous.enable_claude_metadata_user_id_injection);
            let enable_cache_anomaly_monitor =
                enable_cache_anomaly_monitor.unwrap_or(previous.enable_cache_anomaly_monitor);
            let enable_task_complete_notify =
                enable_task_complete_notify.unwrap_or(previous.enable_task_complete_notify);
            let enable_response_fixer =
                enable_response_fixer.unwrap_or(previous.enable_response_fixer);
            let response_fixer_fix_encoding =
                response_fixer_fix_encoding.unwrap_or(previous.response_fixer_fix_encoding);
            let response_fixer_fix_sse_format =
                response_fixer_fix_sse_format.unwrap_or(previous.response_fixer_fix_sse_format);
            let response_fixer_fix_truncated_json = response_fixer_fix_truncated_json
                .unwrap_or(previous.response_fixer_fix_truncated_json);
            let verbose_provider_error =
                verbose_provider_error.unwrap_or(previous.verbose_provider_error);
            let circuit_breaker_failure_threshold = circuit_breaker_failure_threshold
                .unwrap_or(previous.circuit_breaker_failure_threshold);
            let circuit_breaker_open_duration_minutes = circuit_breaker_open_duration_minutes
                .unwrap_or(previous.circuit_breaker_open_duration_minutes);
            let mut next_auto_start = auto_start;

            #[cfg(desktop)]
            {
                let autostart_args_changed = previous.tray_enabled != tray_enabled
                    || previous.silent_startup != silent_startup;
                if auto_start != previous.auto_start || (auto_start && autostart_args_changed) {
                    let result =
                        sync_autostart(&app_for_work, auto_start, tray_enabled, silent_startup);

                    if let Err(err) = result {
                        tracing::warn!("auto-start sync failed: {}", err);
                        next_auto_start = previous.auto_start;
                    }
                }
            }

            let settings = settings::AppSettings {
                schema_version: settings::SCHEMA_VERSION,
                preferred_port,
                gateway_listen_mode,
                gateway_custom_listen_address,
                wsl_auto_config,
                wsl_target_cli,
                wsl_host_address_mode,
                wsl_custom_host_address,
                auto_start: next_auto_start,
                tray_enabled,
                silent_startup,
                enable_cli_proxy_startup_recovery,
                log_retention_days,
                provider_cooldown_seconds,
                provider_base_url_ping_cache_ttl_seconds,
                upstream_first_byte_timeout_seconds,
                upstream_stream_idle_timeout_seconds,
                upstream_request_timeout_non_streaming_seconds,
                update_releases_url,
                failover_max_attempts_per_provider,
                failover_max_providers_to_try,
                circuit_breaker_failure_threshold,
                circuit_breaker_open_duration_minutes,
                enable_circuit_breaker_notice: previous.enable_circuit_breaker_notice,
                verbose_provider_error,
                intercept_anthropic_warmup_requests,
                enable_thinking_signature_rectifier,
                enable_thinking_budget_rectifier,
                enable_codex_session_id_completion: previous.enable_codex_session_id_completion,
                enable_claude_metadata_user_id_injection,
                enable_cache_anomaly_monitor,
                enable_task_complete_notify,
                enable_response_fixer,
                response_fixer_fix_encoding,
                response_fixer_fix_sse_format,
                response_fixer_fix_truncated_json,
                response_fixer_max_json_depth: previous.response_fixer_max_json_depth,
                response_fixer_max_fix_size: previous.response_fixer_max_fix_size,
            };

            let next_settings = settings::write(&app_for_work, &settings)?;
            Ok(next_settings)
        },
    )
    .await?;

    app.state::<resident::ResidentState>()
        .set_tray_enabled(next_settings.tray_enabled);

    tracing::info!(
        preferred_port = next_settings.preferred_port,
        auto_start = next_settings.auto_start,
        tray_enabled = next_settings.tray_enabled,
        log_retention_days = next_settings.log_retention_days,
        failover_max_attempts_per_provider = next_settings.failover_max_attempts_per_provider,
        failover_max_providers_to_try = next_settings.failover_max_providers_to_try,
        "settings updated"
    );

    Ok(next_settings)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn settings_gateway_rectifier_set(
    app: tauri::AppHandle,
    verbose_provider_error: bool,
    intercept_anthropic_warmup_requests: bool,
    enable_thinking_signature_rectifier: bool,
    enable_thinking_budget_rectifier: bool,
    enable_claude_metadata_user_id_injection: bool,
    enable_response_fixer: bool,
    response_fixer_fix_encoding: bool,
    response_fixer_fix_sse_format: bool,
    response_fixer_fix_truncated_json: bool,
    response_fixer_max_json_depth: u32,
    response_fixer_max_fix_size: u32,
) -> Result<settings::AppSettings, String> {
    let app_for_work = app.clone();
    let result = blocking::run("settings_gateway_rectifier_set", move || {
        let mut settings = settings::read(&app_for_work).unwrap_or_default();
        settings.schema_version = settings::SCHEMA_VERSION;

        settings.verbose_provider_error = verbose_provider_error;
        settings.intercept_anthropic_warmup_requests = intercept_anthropic_warmup_requests;
        settings.enable_thinking_signature_rectifier = enable_thinking_signature_rectifier;
        settings.enable_thinking_budget_rectifier = enable_thinking_budget_rectifier;
        settings.enable_claude_metadata_user_id_injection =
            enable_claude_metadata_user_id_injection;
        settings.enable_response_fixer = enable_response_fixer;
        settings.response_fixer_fix_encoding = response_fixer_fix_encoding;
        settings.response_fixer_fix_sse_format = response_fixer_fix_sse_format;
        settings.response_fixer_fix_truncated_json = response_fixer_fix_truncated_json;
        settings.response_fixer_max_json_depth = response_fixer_max_json_depth;
        settings.response_fixer_max_fix_size = response_fixer_max_fix_size;

        settings::write(&app_for_work, &settings)
    })
    .await
    .map_err(Into::into);

    if let Ok(ref settings) = result {
        tracing::info!(
            verbose_provider_error = settings.verbose_provider_error,
            intercept_anthropic_warmup_requests = settings.intercept_anthropic_warmup_requests,
            enable_thinking_signature_rectifier = settings.enable_thinking_signature_rectifier,
            enable_thinking_budget_rectifier = settings.enable_thinking_budget_rectifier,
            enable_claude_metadata_user_id_injection =
                settings.enable_claude_metadata_user_id_injection,
            enable_response_fixer = settings.enable_response_fixer,
            "gateway rectifier settings updated"
        );
    }

    result
}

#[tauri::command]
pub(crate) async fn settings_circuit_breaker_notice_set(
    app: tauri::AppHandle,
    enable_circuit_breaker_notice: bool,
) -> Result<settings::AppSettings, String> {
    let app_for_work = app.clone();
    blocking::run("settings_circuit_breaker_notice_set", move || {
        let mut settings = settings::read(&app_for_work).unwrap_or_default();
        settings.schema_version = settings::SCHEMA_VERSION;
        settings.enable_circuit_breaker_notice = enable_circuit_breaker_notice;
        settings::write(&app_for_work, &settings)
    })
    .await
    .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn settings_codex_session_id_completion_set(
    app: tauri::AppHandle,
    enable_codex_session_id_completion: bool,
) -> Result<settings::AppSettings, String> {
    let app_for_work = app.clone();
    blocking::run("settings_codex_session_id_completion_set", move || {
        let mut settings = settings::read(&app_for_work).unwrap_or_default();
        settings.schema_version = settings::SCHEMA_VERSION;
        settings.enable_codex_session_id_completion = enable_codex_session_id_completion;
        settings::write(&app_for_work, &settings)
    })
    .await
    .map_err(Into::into)
}
