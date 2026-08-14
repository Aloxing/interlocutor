use std::fmt::Write;

use crate::ast::Node;

/// Renders the AST to plain text.
pub struct PlainTextRenderer;

impl PlainTextRenderer {
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
                output.push('\n');
            }
            Node::Paragraph(children) => {
                Self::render_children(children, output);
                output.push('\n');
            }
            Node::BlockQuote(children) => {
                output.push_str("> ");
                Self::render_children(children, output);
                output.push('\n');
            }
            Node::CodeBlock { code, .. } => {
                output.push_str(code);
                output.push('\n');
            }
            Node::List { ordered, items } => {
                for (index, item) in items.iter().enumerate() {
                    if *ordered {
                        let _ = write!(output, "{}", index + 1);
                        output.push_str(". ");
                    } else {
                        output.push_str("- ");
                    }
                    Self::render_node(item, output);
                }
            }
            Node::ListItem(children) => {
                Self::render_children(children, output);
                output.push('\n');
            }
            Node::ThematicBreak => {
                output.push_str("---\n");
            }
            Node::Text(text) => {
                output.push_str(text);
            }
            Node::Bold(children) | Node::Italic(children) | Node::Strikethrough(children) => {
                Self::render_children(children, output);
            }
            Node::InlineCode(code) => {
                output.push_str(code);
            }
            Node::Link { children, .. } => {
                Self::render_children(children, output);
            }
            Node::Image { alt, .. } => {
                output.push_str(alt);
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
}
