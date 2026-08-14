/// Abstract syntax tree for Markdown.
#[derive(Debug, Clone)]
pub enum Node {
    Document(Vec<Node>),

    Heading {
        level: u8,
        children: Vec<Node>,
    },

    Paragraph(Vec<Node>),

    BlockQuote(Vec<Node>),

    CodeBlock {
        language: Option<String>,
        code: String,
    },

    List {
        ordered: bool,
        items: Vec<Node>,
    },

    ListItem(Vec<Node>),

    ThematicBreak,

    Text(String),

    Bold(Vec<Node>),

    Italic(Vec<Node>),

    Strikethrough(Vec<Node>),

    InlineCode(String),

    Link {
        url: String,
        title: Option<String>,
        children: Vec<Node>,
    },

    Image {
        url: String,
        title: Option<String>,
        alt: String,
    },

    SoftBreak,

    HardBreak,
}
