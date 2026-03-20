use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub struct StatusInput {
    pub model: Option<ModelData>,
    pub context_window: Option<ContextData>,
    pub workspace: Option<WorkspaceData>,
    pub version: Option<String>,
}

#[derive(Deserialize, Default, Debug)]
pub struct ModelData {
    pub display_name: Option<String>,
}

#[derive(Deserialize, Default, Debug)]
pub struct ContextData {
    pub used_percentage: Option<f64>,
}

#[derive(Deserialize, Default, Debug)]
pub struct WorkspaceData {
    pub current_dir: Option<String>,
}

impl StatusInput {
    pub fn model_name(&self) -> &str {
        self.model
            .as_ref()
            .and_then(|m| m.display_name.as_deref())
            .unwrap_or("?")
    }

    pub fn context_pct(&self) -> u32 {
        self.context_window
            .as_ref()
            .and_then(|c| c.used_percentage)
            .map(|p| p as u32)
            .unwrap_or(0)
    }

    pub fn current_dir(&self) -> &str {
        self.workspace
            .as_ref()
            .and_then(|w| w.current_dir.as_deref())
            .unwrap_or("")
    }

    pub fn version(&self) -> &str {
        self.version.as_deref().unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse() {
        let json = r#"{"model":{"display_name":"Opus"},"context_window":{"used_percentage":42.5},"workspace":{"current_dir":"/tmp"},"version":"2.1.29"}"#;
        let input: StatusInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.model_name(), "Opus");
        assert_eq!(input.context_pct(), 42);
        assert_eq!(input.current_dir(), "/tmp");
        assert_eq!(input.version(), "2.1.29");
    }

    #[test]
    fn test_missing_model() {
        let input: StatusInput = serde_json::from_str("{}").unwrap();
        assert_eq!(input.model_name(), "?");
    }

    #[test]
    fn test_missing_display_name() {
        let input: StatusInput = serde_json::from_str(r#"{"model":{}}"#).unwrap();
        assert_eq!(input.model_name(), "?");
    }

    #[test]
    fn test_missing_context_window() {
        let input: StatusInput = serde_json::from_str("{}").unwrap();
        assert_eq!(input.context_pct(), 0);
    }

    #[test]
    fn test_missing_used_percentage() {
        let input: StatusInput = serde_json::from_str(r#"{"context_window":{}}"#).unwrap();
        assert_eq!(input.context_pct(), 0);
    }

    #[test]
    fn test_float_percentage_truncated() {
        let input: StatusInput =
            serde_json::from_str(r#"{"context_window":{"used_percentage":42.9}}"#).unwrap();
        assert_eq!(input.context_pct(), 42);
    }

    #[test]
    fn test_null_values() {
        let input: StatusInput = serde_json::from_str(
            r#"{"model":null,"context_window":null,"workspace":null,"version":null}"#,
        )
        .unwrap();
        assert_eq!(input.model_name(), "?");
        assert_eq!(input.context_pct(), 0);
        assert_eq!(input.current_dir(), "");
        assert_eq!(input.version(), "");
    }

    #[test]
    fn test_empty_json() {
        let input: StatusInput = serde_json::from_str("{}").unwrap();
        assert_eq!(input.model_name(), "?");
        assert_eq!(input.context_pct(), 0);
        assert_eq!(input.current_dir(), "");
        assert_eq!(input.version(), "");
    }
}
