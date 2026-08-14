/// Block-level tokens produced by the Lexer.
#[derive(Debug, Clone)]
pub enum BlockToken {
    Heading(u8, String),
    Paragraph(String),
    Quote(String),
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    List {
        ordered: bool,
        items: Vec<String>,
    },
    ThematicBreak,
}

/// Splits Markdown text into block-level tokens.
pub struct Lexer;

impl Lexer {
    pub fn tokenize(input: &str) -> Vec<BlockToken> {
        let mut tokens = Vec::new();
        let lines: Vec<&str> = input.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            if line.trim().is_empty() {
                i += 1;
                continue;
            }

            // Fenced code block
            if line.starts_with("```") {
                let language = {
                    let lang = line.trim_start_matches("```").trim();
                    if lang.is_empty() {
                        None
                    } else {
                        Some(lang.to_string())
                    }
                };

                i += 1;
                let mut code = String::new();
                while i < lines.len() && !lines[i].starts_with("```") {
                    code.push_str(lines[i]);
                    code.push('\n');
                    i += 1;
                }
                if i < lines.len() {
                    i += 1;
                }
                tokens.push(BlockToken::CodeBlock { language, code });
                continue;
            }

            // Heading
            if let Some((level, text)) = Self::parse_heading(line) {
                tokens.push(BlockToken::Heading(level, text));
                i += 1;
                continue;
            }

            // Thematic break
            if Self::is_thematic_break(line) {
                tokens.push(BlockToken::ThematicBreak);
                i += 1;
                continue;
            }

            // Block quote
            if line.starts_with('>') {
                let text = line.trim_start_matches('>').trim_start().to_string();
                tokens.push(BlockToken::Quote(text));
                i += 1;
                continue;
            }

            // List
            if Self::is_list(line) {
                let ordered = Self::is_ordered_list(line);
                let mut items = Vec::new();
                while i < lines.len() && Self::is_list(lines[i]) {
                    items.push(Self::strip_list_marker(lines[i]));
                    i += 1;
                }
                tokens.push(BlockToken::List { ordered, items });
                continue;
            }

            // Paragraph
            let mut paragraph = String::new();
            while i < lines.len() {
                let current = lines[i];
                if current.trim().is_empty() {
                    break;
                }
                if !paragraph.is_empty() {
                    paragraph.push('\n');
                }
                paragraph.push_str(current);
                i += 1;
            }
            tokens.push(BlockToken::Paragraph(paragraph));
        }

        tokens
    }

    fn parse_heading(line: &str) -> Option<(u8, String)> {
        let trimmed = line.trim_start();
        let mut level = 0;
        for c in trimmed.chars() {
            if c == '#' {
                level += 1;
            } else {
                break;
            }
        }
        if level == 0 || level > 6 {
            return None;
        }
        let rest = trimmed[level..].trim_start();
        if rest.is_empty() {
            return None;
        }
        Some((level as u8, rest.to_string()))
    }

    fn is_thematic_break(line: &str) -> bool {
        let s = line.trim();
        s == "---" || s == "***" || s == "___"
    }

    fn is_list(line: &str) -> bool {
        let s = line.trim_start();
        s.starts_with("- ")
            || s.starts_with("* ")
            || s.starts_with("+ ")
            || Self::is_ordered_list(line)
    }

    fn is_ordered_list(line: &str) -> bool {
        let s = line.trim_start();
        let mut chars = s.chars();
        let mut found_number = false;

        while let Some(c) = chars.next() {
            if c.is_ascii_digit() {
                found_number = true;
                continue;
            }
            if found_number && c == '.' {
                return chars.next() == Some(' ');
            }
            return false;
        }
        false
    }

    fn strip_list_marker(line: &str) -> String {
        let s = line.trim_start();
        if s.starts_with("- ") || s.starts_with("* ") || s.starts_with("+ ") {
            return s[2..].to_string();
        }

        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'.' {
            i += 1;
            if i < bytes.len() && bytes[i] == b' ' {
                i += 1;
            }
            return s[i..].to_string();
        }
        s.to_string()
    }
}
