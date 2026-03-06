use super::*;

fn empty_patch() -> CodexConfigPatch {
    CodexConfigPatch {
        model: None,
        approval_policy: None,
        sandbox_mode: None,
        model_reasoning_effort: None,
        plan_mode_reasoning_effort: None,
        web_search: None,
        sandbox_workspace_write_network_access: None,
        features_unified_exec: None,
        features_shell_snapshot: None,
        features_apply_patch_freeform: None,
        features_shell_tool: None,
        features_exec_policy: None,
        features_remote_compaction: None,
        features_remote_models: None,
        features_multi_agent: None,
        features_fast_mode: None,
    }
}

#[test]
fn patch_creates_features_table_and_preserves_other_tables() {
    let input = r#"# header

[mcp_servers.exa]
type = "stdio"

"#;

    let out = patch_config_toml(
        Some(input.as_bytes().to_vec()),
        CodexConfigPatch {
            features_shell_snapshot: Some(true),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let s = String::from_utf8(out).expect("utf8");
    assert!(s.contains("[mcp_servers.exa]"), "{s}");
    assert!(s.contains("[features]"), "{s}");
    assert!(s.contains("shell_snapshot = true"), "{s}");
}

#[test]
fn patch_deletes_sandbox_workspace_write_network_access_and_table_when_false() {
    let input = r#"[sandbox_workspace_write]
network_access = false
"#;

    let out = patch_config_toml(
        Some(input.as_bytes().to_vec()),
        CodexConfigPatch {
            sandbox_workspace_write_network_access: Some(false),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let s = String::from_utf8(out).expect("utf8");
    assert!(!s.contains("[sandbox_workspace_write]"), "{s}");
    assert!(!s.contains("network_access"), "{s}");
}

#[test]
fn patch_deletes_sandbox_workspace_write_network_access_but_preserves_other_keys() {
    let input = r#"[sandbox_workspace_write]
network_access = true
other = "keep"
"#;

    let out = patch_config_toml(
        Some(input.as_bytes().to_vec()),
        CodexConfigPatch {
            sandbox_workspace_write_network_access: Some(false),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let s = String::from_utf8(out).expect("utf8");
    assert!(s.contains("[sandbox_workspace_write]"), "{s}");
    assert!(s.contains("other = \"keep\""), "{s}");
    assert!(!s.contains("network_access ="), "{s}");
}

#[test]
fn patch_preserves_existing_features_when_setting_another() {
    let input = r#"[features]
shell_tool = true
"#;

    let out = patch_config_toml(
        Some(input.as_bytes().to_vec()),
        CodexConfigPatch {
            features_shell_snapshot: Some(true),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let s = String::from_utf8(out).expect("utf8");
    assert!(s.contains("shell_tool = true"), "{s}");
    assert!(s.contains("shell_snapshot = true"), "{s}");
}

#[test]
fn patch_deletes_default_false_feature_when_disabled() {
    let input = r#"[features]
shell_snapshot = true
"#;

    let out = patch_config_toml(
        Some(input.as_bytes().to_vec()),
        CodexConfigPatch {
            features_shell_snapshot: Some(false),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let s = String::from_utf8(out).expect("utf8");
    assert!(!s.contains("shell_snapshot ="), "{s}");
}

#[test]
fn patch_writes_true_when_feature_enabled() {
    let input = r#"[features]
shell_tool = false
"#;

    let out = patch_config_toml(
        Some(input.as_bytes().to_vec()),
        CodexConfigPatch {
            features_shell_tool: Some(true),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let s = String::from_utf8(out).expect("utf8");
    assert!(!s.contains("shell_tool = false"), "{s}");
    assert!(s.contains("shell_tool = true"), "{s}");
}

#[test]
fn patch_deletes_feature_when_disabled() {
    let input = r#"[features]
shell_tool = true
"#;

    let out = patch_config_toml(
        Some(input.as_bytes().to_vec()),
        CodexConfigPatch {
            features_shell_tool: Some(false),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let s = String::from_utf8(out).expect("utf8");
    assert!(!s.contains("shell_tool ="), "{s}");
}

#[test]
fn patch_compacts_blank_lines_in_features_table() {
    let input = r#"[features]

shell_snapshot = true



[other]
foo = "bar"
"#;

    let out1 = patch_config_toml(
        Some(input.as_bytes().to_vec()),
        CodexConfigPatch {
            features_shell_tool: Some(true),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let out2 = patch_config_toml(
        Some(out1),
        CodexConfigPatch {
            features_unified_exec: Some(true),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let s = String::from_utf8(out2).expect("utf8");
    assert!(
        s.contains(
            "[features]\n\
shell_snapshot = true\n\
unified_exec = true\n\
shell_tool = true\n\n\
[other]\n"
        ),
        "{s}"
    );
    assert!(!s.contains("[features]\n\n"), "{s}");
    assert!(!s.contains("true\n\nshell_tool"), "{s}");
    assert!(!s.contains("true\n\nunified_exec"), "{s}");
}

#[test]
fn patch_compacts_blank_lines_across_entire_file() {
    let input = r#"approval_policy = "never"


preferred_auth_method = "apikey"


[features]


shell_snapshot = true


[mcp_servers.exa]
type = "stdio"
"#;

    let out = patch_config_toml(
        Some(input.as_bytes().to_vec()),
        CodexConfigPatch {
            features_shell_tool: Some(true),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let s = String::from_utf8(out).expect("utf8");
    assert!(
        s.contains(
            "approval_policy = \"never\"\n\
preferred_auth_method = \"apikey\"\n\n\
[features]\n\
shell_snapshot = true\n\
shell_tool = true\n\n\
[mcp_servers.exa]\n\
type = \"stdio\"\n"
        ),
        "{s}"
    );
    assert!(!s.contains("\n\n\n"), "{s}");
}

#[test]
fn validate_raw_allows_empty() {
    let out = validate_codex_config_toml_raw("");
    assert!(out.ok, "{out:?}");
    assert!(out.error.is_none(), "{out:?}");
}

#[test]
fn validate_raw_rejects_invalid_toml_with_location_when_available() {
    let out = validate_codex_config_toml_raw("approval_policy =");
    assert!(!out.ok, "{out:?}");
    let err = out.error.expect("error");
    assert!(!err.message.trim().is_empty(), "{err:?}");
    assert!(
        err.line.is_some() || err.column.is_some(),
        "expected line/column when available: {err:?}"
    );
}

#[test]
fn validate_raw_rejects_invalid_enum_values() {
    let out = validate_codex_config_toml_raw("approval_policy = \"nope\"");
    assert!(!out.ok, "{out:?}");
    let err = out.error.expect("error");
    assert!(err.message.contains("approval_policy"), "{err:?}");
    assert!(err.message.contains("allowed:"), "{err:?}");
}

#[test]
fn parse_reads_sandbox_mode_from_sandbox_table() {
    let input = r#"[sandbox]
mode = "read-only"
"#;

    let state = make_state_from_bytes(
        "dir".to_string(),
        "path".to_string(),
        true,
        Some(input.as_bytes().to_vec()),
    )
    .expect("make_state_from_bytes");

    assert_eq!(state.sandbox_mode.as_deref(), Some("read-only"));
}

#[test]
fn parse_prefers_root_sandbox_mode_over_sandbox_table() {
    let input = r#"sandbox_mode = "workspace-write"

[sandbox]
mode = "read-only"
"#;

    let state = make_state_from_bytes(
        "dir".to_string(),
        "path".to_string(),
        true,
        Some(input.as_bytes().to_vec()),
    )
    .expect("make_state_from_bytes");

    assert_eq!(state.sandbox_mode.as_deref(), Some("workspace-write"));
}

#[test]
fn parse_ignores_table_headers_inside_multiline_strings() {
    let input = r#"prompt = """
[not_a_table]
foo = "bar"
"""

sandbox_mode = "read-only"
"#;

    let state = make_state_from_bytes(
        "dir".to_string(),
        "path".to_string(),
        true,
        Some(input.as_bytes().to_vec()),
    )
    .expect("make_state_from_bytes");

    assert_eq!(state.sandbox_mode.as_deref(), Some("read-only"));
}

#[test]
fn patch_updates_sandbox_table_mode_when_present() {
    let input = r#"[sandbox]
mode = "read-only"

[other]
foo = "bar"
"#;

    let out = patch_config_toml(
        Some(input.as_bytes().to_vec()),
        CodexConfigPatch {
            sandbox_mode: Some("workspace-write".to_string()),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let s = String::from_utf8(out).expect("utf8");
    assert!(s.contains("[sandbox]\nmode = \"workspace-write\""), "{s}");
    assert!(!s.contains("sandbox_mode ="), "{s}");
}

#[test]
fn patch_updates_sandbox_dotted_mode_when_present() {
    let input = r#"sandbox.mode = "read-only"
"#;

    let out = patch_config_toml(
        Some(input.as_bytes().to_vec()),
        CodexConfigPatch {
            sandbox_mode: Some("danger-full-access".to_string()),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let s = String::from_utf8(out).expect("utf8");
    assert!(s.contains("sandbox.mode = \"danger-full-access\""), "{s}");
    assert!(!s.contains("[sandbox]"), "{s}");
    assert!(!s.contains("sandbox_mode ="), "{s}");
}

#[test]
fn patch_writes_fast_mode_when_enabled() {
    let input = r#"[features]
shell_tool = true
"#;

    let out = patch_config_toml(
        Some(input.as_bytes().to_vec()),
        CodexConfigPatch {
            features_fast_mode: Some(true),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let s = String::from_utf8(out).expect("utf8");
    assert!(s.contains("fast_mode = true"), "{s}");
    assert!(s.contains("shell_tool = true"), "{s}");
}

#[test]
fn patch_deletes_fast_mode_when_disabled() {
    let input = r#"[features]
fast_mode = true
shell_tool = true
"#;

    let out = patch_config_toml(
        Some(input.as_bytes().to_vec()),
        CodexConfigPatch {
            features_fast_mode: Some(false),
            ..empty_patch()
        },
    )
    .expect("patch_config_toml");

    let s = String::from_utf8(out).expect("utf8");
    assert!(!s.contains("fast_mode ="), "{s}");
    assert!(s.contains("shell_tool = true"), "{s}");
}

#[test]
fn get_reads_fast_mode_from_features_table() {
    let state = make_state_from_bytes(
        "/home/user/.codex".to_string(),
        "/home/user/.codex/config.toml".to_string(),
        true,
        Some(
            b"[features]
fast_mode = true
"
            .to_vec(),
        ),
    )
    .expect("make_state_from_bytes");

    assert_eq!(state.features_fast_mode, Some(true));
}
