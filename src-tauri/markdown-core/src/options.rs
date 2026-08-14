/// Parser options. Reserved for CommonMark/GFM compatibility work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkdownOptions {
    pub gfm: bool,
    pub breaks: bool,
    pub html: bool,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            gfm: true,
            breaks: false,
            html: false,
        }
    }
}
