use super::Parser;
use crate::ast::Node;
use crate::scanner::{find_char, find_double};

impl Parser {
    pub(crate) fn parse_inline(input: &str) -> Vec<Node> {
        let mut nodes = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Image ![alt](url)
            if chars[i] == '!' && i + 1 < chars.len() && chars[i + 1] == '[' {
                if let Some(end_alt) = find_char(&chars, i + 2, ']') {
                    if end_alt + 1 < chars.len() && chars[end_alt + 1] == '(' {
                        if let Some(end_url) = find_char(&chars, end_alt + 2, ')') {
                            let alt: String = chars[i + 2..end_alt].iter().collect();
                            let url: String = chars[end_alt + 2..end_url].iter().collect();
                            nodes.push(Node::Image {
                                url,
                                title: None,
                                alt,
                            });
                            i = end_url + 1;
                            continue;
                        }
                    }
                }
            }

            // Link [text](url)
            if chars[i] == '[' {
                if let Some(end_text) = find_char(&chars, i + 1, ']') {
                    if end_text + 1 < chars.len() && chars[end_text + 1] == '(' {
                        if let Some(end_url) = find_char(&chars, end_text + 2, ')') {
                            let text: String = chars[i + 1..end_text].iter().collect();
                            let url: String = chars[end_text + 2..end_url].iter().collect();
                            nodes.push(Node::Link {
                                url,
                                title: None,
                                children: Self::parse_inline(&text),
                            });
                            i = end_url + 1;
                            continue;
                        }
                    }
                }
            }

            // Inline code `code`
            if chars[i] == '`' {
                if let Some(end) = find_char(&chars, i + 1, '`') {
                    let code: String = chars[i + 1..end].iter().collect();
                    nodes.push(Node::InlineCode(code));
                    i = end + 1;
                    continue;
                }
            }

            // Bold **text**
            if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
                if let Some(end) = find_double(&chars, i + 2, '*') {
                    let text: String = chars[i + 2..end].iter().collect();
                    nodes.push(Node::Bold(Self::parse_inline(&text)));
                    i = end + 2;
                    continue;
                }
            }

            // Italic *text*
            if chars[i] == '*' {
                if let Some(end) = find_char(&chars, i + 1, '*') {
                    let text: String = chars[i + 1..end].iter().collect();
                    nodes.push(Node::Italic(Self::parse_inline(&text)));
                    i = end + 1;
                    continue;
                }
            }

            // Strikethrough ~~text~~
            if i + 1 < chars.len() && chars[i] == '~' && chars[i + 1] == '~' {
                if let Some(end) = find_double(&chars, i + 2, '~') {
                    let text: String = chars[i + 2..end].iter().collect();
                    nodes.push(Node::Strikethrough(Self::parse_inline(&text)));
                    i = end + 2;
                    continue;
                }
            }

            // Newline
            if chars[i] == '\n' {
                nodes.push(Node::SoftBreak);
                i += 1;
                continue;
            }

            // Plain text run
            let mut text = String::new();
            while i < chars.len() {
                let c = chars[i];
                if c == '*' || c == '`' || c == '~' || c == '[' || c == '!' || c == '\n' {
                    break;
                }
                text.push(c);
                i += 1;
            }

            if !text.is_empty() {
                nodes.push(Node::Text(text));
            } else {
                nodes.push(Node::Text(chars[i].to_string()));
                i += 1;
            }
        }

        nodes
    }
}
