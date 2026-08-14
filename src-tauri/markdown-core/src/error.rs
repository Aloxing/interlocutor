use std::fmt;

/// Error type used by the Markdown core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownError(pub String);

impl fmt::Display for MarkdownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MarkdownError {}

impl From<String> for MarkdownError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for MarkdownError {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
