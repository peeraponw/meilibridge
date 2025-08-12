use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,

    #[serde(default = "default_log_format")]
    pub format: LogFormat,

    #[serde(default)]
    pub outputs: Vec<LogOutput>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            outputs: vec![LogOutput::Stdout {
                format: None,
                filter: None,
            }],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
    Text,
    Json,
    Pretty,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum LogOutput {
    #[serde(rename = "stdout")]
    Stdout {
        #[serde(default)]
        format: Option<LogFormat>,
        #[serde(default)]
        filter: Option<String>,
    },

    #[serde(rename = "stderr")]
    Stderr {
        #[serde(default)]
        format: Option<LogFormat>,
        #[serde(default)]
        filter: Option<String>,
    },

    #[serde(rename = "file")]
    File {
        path: String,
        #[serde(default)]
        format: Option<LogFormat>,
        #[serde(default)]
        rotation: LogRotation,
        #[serde(default)]
        filter: Option<String>,
    },
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogRotation {
    Never,
    #[default]
    Daily,
    Hourly,
    Size(u64),
}

fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> LogFormat {
    LogFormat::Text
}
