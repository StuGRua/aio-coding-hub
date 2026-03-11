//! Usage: MCP server management related Tauri commands.

use crate::app_state::{ensure_db_ready, DbInitState};
use crate::{blocking, mcp};

#[tauri::command]
pub(crate) async fn mcp_servers_list(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    workspace_id: i64,
) -> Result<Vec<mcp::McpServerSummary>, String> {
    let db = ensure_db_ready(app, db_state.inner()).await?;
    blocking::run("mcp_servers_list", move || {
        mcp::list_for_workspace(&db, workspace_id)
    })
    .await
    .map_err(Into::into)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn mcp_server_upsert(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    server_id: Option<i64>,
    server_key: String,
    name: String,
    transport: String,
    command: Option<String>,
    args: Vec<String>,
    env: std::collections::BTreeMap<String, String>,
    cwd: Option<String>,
    url: Option<String>,
    headers: std::collections::BTreeMap<String, String>,
) -> Result<mcp::McpServerSummary, String> {
    #[cfg(windows)]
    let app_for_wsl = app.clone();
    let db = ensure_db_ready(app.clone(), db_state.inner()).await?;
    let result = blocking::run("mcp_server_upsert", move || {
        mcp::upsert(
            &app,
            &db,
            server_id,
            &server_key,
            &name,
            &transport,
            command.as_deref(),
            args,
            env,
            cwd.as_deref(),
            url.as_deref(),
            headers,
        )
    })
    .await
    .map_err(Into::into);
    #[cfg(windows)]
    if result.is_ok() {
        super::wsl::wsl_sync_trigger::trigger(app_for_wsl);
    }
    result
}

#[tauri::command]
pub(crate) async fn mcp_server_set_enabled(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    workspace_id: i64,
    server_id: i64,
    enabled: bool,
) -> Result<mcp::McpServerSummary, String> {
    #[cfg(windows)]
    let app_for_wsl = app.clone();
    let db = ensure_db_ready(app.clone(), db_state.inner()).await?;
    let result = blocking::run("mcp_server_set_enabled", move || {
        mcp::set_enabled(&app, &db, workspace_id, server_id, enabled)
    })
    .await
    .map_err(Into::into);
    #[cfg(windows)]
    if result.is_ok() {
        super::wsl::wsl_sync_trigger::trigger(app_for_wsl);
    }
    result
}

#[tauri::command]
pub(crate) async fn mcp_server_delete(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    server_id: i64,
) -> Result<bool, String> {
    #[cfg(windows)]
    let app_for_wsl = app.clone();
    let db = ensure_db_ready(app.clone(), db_state.inner()).await?;
    let result = blocking::run(
        "mcp_server_delete",
        move || -> crate::shared::error::AppResult<bool> {
            mcp::delete(&app, &db, server_id)?;
            Ok(true)
        },
    )
    .await
    .map_err(Into::into);
    #[cfg(windows)]
    if result.is_ok() {
        super::wsl::wsl_sync_trigger::trigger(app_for_wsl);
    }
    result
}

#[tauri::command]
pub(crate) fn mcp_parse_json(json_text: String) -> Result<mcp::McpParseResult, String> {
    mcp::parse_json(&json_text).map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn mcp_import_servers(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    workspace_id: i64,
    servers: Vec<mcp::McpImportServer>,
) -> Result<mcp::McpImportReport, String> {
    #[cfg(windows)]
    let app_for_wsl = app.clone();
    let db = ensure_db_ready(app.clone(), db_state.inner()).await?;
    let result = blocking::run("mcp_import_servers", move || {
        mcp::import_servers(&app, &db, workspace_id, servers)
    })
    .await
    .map_err(Into::into);
    #[cfg(windows)]
    if result.is_ok() {
        super::wsl::wsl_sync_trigger::trigger(app_for_wsl);
    }
    result
}

#[tauri::command]
pub(crate) async fn mcp_import_from_workspace_cli(
    app: tauri::AppHandle,
    db_state: tauri::State<'_, DbInitState>,
    workspace_id: i64,
) -> Result<mcp::McpImportReport, String> {
    #[cfg(windows)]
    let app_for_wsl = app.clone();
    let db = ensure_db_ready(app.clone(), db_state.inner()).await?;
    let result = blocking::run("mcp_import_from_workspace_cli", move || {
        mcp::import_servers_from_workspace_cli(&app, &db, workspace_id)
    })
    .await
    .map_err(Into::into);
    #[cfg(windows)]
    if result.is_ok() {
        super::wsl::wsl_sync_trigger::trigger(app_for_wsl);
    }
    result
}
