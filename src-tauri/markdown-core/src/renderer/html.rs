use std::fmt::Write;

use crate::ast::Node;

/// Renders the AST to HTML.
pub struct HtmlRenderer;

impl HtmlRenderer {
    pub fn render(node: &Node) -> String {
        let mut output = String::new();
        Self::render_node(node, &mut output);
        output
    }

    fn render_node(node: &Node, output: &mut String) {
        match node {
            Node::Document(children) => {
                for child in children {
                    Self::render_node(child, output);
                }
            }
            Node::Heading { level, children } => {
                let _ = write!(output, "<h{}>", level);
                Self::render_children(children, output);
                let _ = write!(output, "</h{}>\n", level);
            }
            Node::Paragraph(children) => {
                output.push_str("<p>");
                Self::render_children(children, output);
                output.push_str("</p>\n");
            }
            Node::BlockQuote(children) => {
                output.push_str("<blockquote>\n");
                Self::render_children(children, output);
                output.push_str("</blockquote>\n");
            }
            Node::CodeBlock { language, code } => {
                output.push_str("<pre><code");
                if let Some(lang) = language {
                    let _ = write!(output, " class=\"language-{}\"", Self::escape(lang));
                }
                output.push('>');
                output.push_str(&Self::escape(code));
                output.push_str("</code></pre>\n");
            }
            Node::List { ordered, items } => {
                if *ordered {
                    output.push_str("<ol>\n");
                } else {
                    output.push_str("<ul>\n");
                }
                for item in items {
                    Self::render_node(item, output);
                }
                if *ordered {
                    output.push_str("</ol>\n");
                } else {
                    output.push_str("</ul>\n");
                }
            }
            Node::ListItem(children) => {
                output.push_str("<li>");
                Self::render_children(children, output);
                output.push_str("</li>\n");
            }
            Node::ThematicBreak => {
                output.push_str("<hr>\n");
            }
            Node::Text(text) => {
                output.push_str(&Self::escape(text));
            }
            Node::Bold(children) => {
                output.push_str("<strong>");
                Self::render_children(children, output);
                output.push_str("</strong>");
            }
            Node::Italic(children) => {
                output.push_str("<em>");
                Self::render_children(children, output);
                output.push_str("</em>");
            }
            Node::Strikethrough(children) => {
                output.push_str("<del>");
                Self::render_children(children, output);
                output.push_str("</del>");
            }
            Node::InlineCode(code) => {
                output.push_str("<code>");
                output.push_str(&Self::escape(code));
                output.push_str("</code>");
            }
            Node::Link { url, children, .. } => {
                let _ = write!(output, "<a href=\"{}\">", Self::escape(url));
                Self::render_children(children, output);
                output.push_str("</a>");
            }
            Node::Image { url, alt, .. } => {
                let _ = write!(
                    output,
                    "<img src=\"{}\" alt=\"{}\">",
                    Self::escape(url),
                    Self::escape(alt)
                );
            }
            Node::SoftBreak => {
                output.push('\n');
            }
            Node::HardBreak => {
                output.push_str("<br>\n");
            }
        }
    }

    fn render_children(children: &[Node], output: &mut String) {
        for child in children {
            Self::render_node(child, output);
        }
    }

    fn escape(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}
