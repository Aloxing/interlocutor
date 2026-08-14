use super::Parser;
use crate::ast::Node;
use crate::lexer::block::{BlockToken, Lexer};

impl Parser {
    pub fn parse(input: &str) -> Node {
        let tokens = Lexer::tokenize(input);
        let mut nodes = Vec::new();

        for token in tokens {
            match token {
                BlockToken::Heading(level, text) => {
                    nodes.push(Node::Heading {
                        level,
                        children: Self::parse_inline(&text),
                    });
                }
                BlockToken::Paragraph(text) => {
                    nodes.push(Node::Paragraph(Self::parse_inline(&text)));
                }
                BlockToken::Quote(text) => {
                    nodes.push(Node::BlockQuote(vec![Node::Paragraph(Self::parse_inline(
                        &text,
                    ))]));
                }
                BlockToken::CodeBlock { language, code } => {
                    nodes.push(Node::CodeBlock { language, code });
                }
                BlockToken::List { ordered, items } => {
                    let children = items
                        .into_iter()
                        .map(|item| Node::ListItem(Self::parse_inline(&item)))
                        .collect();
                    nodes.push(Node::List {
                        ordered,
                        items: children,
                    });
                }
                BlockToken::ThematicBreak => {
                    nodes.push(Node::ThematicBreak);
                }
            }
        }

        Node::Document(nodes)
    }
}
