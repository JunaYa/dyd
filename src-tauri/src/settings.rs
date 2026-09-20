use serde_json::Value;

pub fn screenshot_path(value: &Value) -> Result<&str, String> {
    value
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| "settings.json: screenshot_path.value must be a string".to_string())
}

pub fn needs_startup(
    first_run: Option<&Value>,
    saved_path: Option<&Value>,
) -> Result<bool, String> {
    if let Some(path) = saved_path {
        screenshot_path(path)?;
    }
    match first_run {
        // Older releases saved a path without writing the first_run marker.
        None => Ok(saved_path.is_none()),
        Some(Value::Null) => Ok(true),
        Some(value) => value
            .get("value")
            .and_then(Value::as_bool)
            .map(|has_started| !has_started)
            .ok_or_else(|| "settings.json: first_run.value must be a boolean".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fresh_install_needs_startup() {
        assert_eq!(needs_startup(None, None), Ok(true));
    }

    #[test]
    fn legacy_install_keeps_its_path_and_skips_startup() {
        for path in ["/Users/test/截图", ""] {
            let saved = json!({ "value": path });
            assert_eq!(needs_startup(None, Some(&saved)), Ok(false));
            assert_eq!(screenshot_path(&saved), Ok(path));
        }
    }

    #[test]
    fn unfinished_startup_is_retried() {
        for marker in [Value::Null, json!({ "value": false })] {
            assert_eq!(needs_startup(Some(&marker), None), Ok(true));
        }
    }

    #[test]
    fn completed_startup_is_not_repeated_even_if_path_is_missing() {
        let marker = json!({ "value": true });
        assert_eq!(needs_startup(Some(&marker), None), Ok(false));
    }

    #[test]
    fn invalid_settings_are_rejected_instead_of_reset() {
        for path in [Value::Null, json!("/tmp"), json!({ "value": 123 })] {
            assert!(needs_startup(None, Some(&path)).is_err());
        }
        for marker in [json!(true), json!({ "value": "false" }), json!({})] {
            assert!(needs_startup(Some(&marker), None).is_err());
        }
    }
}
