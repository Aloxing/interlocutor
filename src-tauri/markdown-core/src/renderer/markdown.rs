use std::fmt::Write;

use crate::ast::Node;

/// Renders the AST back to normalized Markdown.
pub struct MarkdownRenderer;

impl MarkdownRenderer {
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
                output.push_str(&"#".repeat(*level as usize));
                output.push(' ');
                Self::render_children(children, output);
                output.push_str("\n\n");
            }
            Node::Paragraph(children) => {
                Self::render_children(children, output);
                output.push_str("\n\n");
            }
            Node::BlockQuote(children) => {
                output.push_str("> ");
                Self::render_children(children, output);
                output.push_str("\n\n");
            }
            Node::CodeBlock { language, code } => {
                output.push_str("```");
                if let Some(lang) = language {
                    output.push_str(lang);
                }
                output.push('\n');
                output.push_str(code);
                output.push_str("```\n\n");
            }
            Node::List { ordered, items } => {
                for (index, item) in items.iter().enumerate() {
                    if *ordered {
                        let _ = write!(output, "{}. ", index + 1);
                    } else {
                        output.push_str("- ");
                    }
                    Self::render_node(item, output);
                }
                output.push('\n');
            }
            Node::ListItem(children) => {
                Self::render_children(children, output);
                output.push('\n');
            }
            Node::ThematicBreak => {
                output.push_str("---\n\n");
            }
            Node::Text(text) => {
                output.push_str(&Self::escape(text));
            }
            Node::Bold(children) => {
                output.push_str("**");
                Self::render_children(children, output);
                output.push_str("**");
            }
            Node::Italic(children) => {
                output.push('*');
                Self::render_children(children, output);
                output.push('*');
            }
            Node::Strikethrough(children) => {
                output.push_str("~~");
                Self::render_children(children, output);
                output.push_str("~~");
            }
            Node::InlineCode(code) => {
                output.push('`');
                output.push_str(code);
                output.push('`');
            }
            Node::Link { url, children, .. } => {
                output.push('[');
                Self::render_children(children, output);
                output.push_str("](");
                output.push_str(url);
                output.push(')');
            }
            Node::Image { url, alt, .. } => {
                output.push_str("![");
                output.push_str(alt);
                output.push_str("](");
                output.push_str(url);
                output.push(')');
            }
            Node::SoftBreak | Node::HardBreak => {
                output.push('\n');
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
            .replace('\\', "\\\\")
            .replace('*', "\\*")
            .replace('`', "\\`")
            .replace('[', "\\[")
            .replace(']', "\\]")
    }
}
