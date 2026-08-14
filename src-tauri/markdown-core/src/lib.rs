pub mod ast;
pub mod error;
pub mod lexer;
pub mod options;
pub mod parser;
pub mod renderer;
pub mod scanner;

pub use ast::Node;
pub use error::MarkdownError;
pub use options::MarkdownOptions;

/// Public entry point for Markdown conversion.
pub struct Markdown;

impl Markdown {
    /// Markdown -> AST
    pub fn parse(input: &str) -> Node {
        parser::Parser::parse(input)
    }

    /// Markdown -> HTML
    pub fn to_html(input: &str) -> String {
        let ast = Self::parse(input);
        renderer::html::HtmlRenderer::render(&ast)
    }

    /// Markdown -> Plain text
    pub fn to_plain_text(input: &str) -> String {
        let ast = Self::parse(input);
        renderer::plain_text::PlainTextRenderer::render(&ast)
    }

    /// Markdown -> normalized Markdown
    pub fn to_markdown(input: &str) -> String {
        let ast = Self::parse(input);
        renderer::markdown::MarkdownRenderer::render(&ast)
    }
}

#[cfg(test)]
mod tests {
    use super::Markdown;

    #[test]
    fn renders_heading_and_inline_styles() {
        let html = Markdown::to_html("# Hello **World** and *italic*");
        assert!(html.contains("<h1>Hello <strong>World</strong> and <em>italic</em></h1>"));
    }

    #[test]
    fn renders_fenced_code_block_with_language() {
        let html = Markdown::to_html("```rust\nfn main() {}\n```");
        assert!(html.contains(r#"<pre><code class="language-rust">fn main() {}"#));
    }

    #[test]
    fn renders_unordered_and_ordered_lists() {
        let html = Markdown::to_html("- a\n- b\n\n1. x\n2. y");
        assert!(html.contains("<ul>\n<li>a</li>\n<li>b</li>\n</ul>"));
        assert!(html.contains("<ol>\n<li>x</li>\n<li>y</li>\n</ol>"));
    }

    #[test]
    fn renders_links_and_images() {
        let html = Markdown::to_html("[OpenAI](https://openai.com) ![logo](logo.png)");
        assert!(html.contains(r#"<a href="https://openai.com">OpenAI</a>"#));
        assert!(html.contains(r#"<img src="logo.png" alt="logo">"#));
    }

    #[test]
    fn escapes_html_in_text() {
        let html = Markdown::to_html("<script>alert(1)</script>");
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn renders_plain_text() {
        let text = Markdown::to_plain_text("# Title\n\nSome **bold** text.");
        assert!(text.contains("Title"));
        assert!(text.contains("Some bold text."));
    }
}
