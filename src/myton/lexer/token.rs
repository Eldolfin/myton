use regex::Regex;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone)]
pub struct Token{
    pub kind: TokenKind,
    pub value: String,
    pub pos: Option<(usize, usize)>,
    pub indent: usize,
}

#[derive(Debug, EnumIter, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    LeftParen, RightParen, LeftBracket, RightBracket,
    Comma, Dot, Plus, Minus, Slash, Star, Colon,
    Identifier, Stringue, Number, Percent,

    BangEqual, Bang,
    Equal, EqualEqual, EqualEqualEqual,
    GreaterEqual, Greater,
    LessEqual, Less,

    And, Class, Else, False, Def, For, If, Nil, Or, 
    Print, Return, Super, Selph, True, While, Pass,
    In,

    Comment,
    Space,
    Indent,
    Newline,

    Eof,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.value == other.value
    }
}

impl Default for Token {
    fn default() -> Self {
        Token {
            kind: TokenKind::Eof,
            value: "".to_string(),
            pos: None,
            indent: 0,
        }
    }
}

impl TokenKind {
    pub fn regex(&self) -> &str {
        match self {
            TokenKind::Number => r"\d+(\.\d+)?",
            TokenKind::Plus => r"\+",
            TokenKind::Minus => r"-",
            TokenKind::Star => r"\*",
            TokenKind::Slash => r"/",
            TokenKind::LeftParen => r"\(",
            TokenKind::RightParen => r"\)",
            TokenKind::LeftBracket => r"\[",
            TokenKind::RightBracket => r"\]",
            TokenKind::Colon => r":",
            TokenKind::Comment => r"(?m)#.*$",
            TokenKind::Eof => r"^$",
            TokenKind::Bang => r"!",
            TokenKind::BangEqual => r"!=",
            TokenKind::Equal => r"=",
            TokenKind::EqualEqual => r"==",
            TokenKind::EqualEqualEqual => r"===",
            TokenKind::Greater => r">",
            TokenKind::GreaterEqual => r">=",
            TokenKind::Less => r"<",
            TokenKind::LessEqual => r"<=",
            TokenKind::Comma => r",",
            TokenKind::Dot => r"\.",
            TokenKind::Newline => r"\n",
            TokenKind::Identifier => r"[a-zA-Z_][a-zA-Z0-9_]*",
            TokenKind::Stringue => r#""[^"]*""#,
            TokenKind::Space => r"[ \t]+",
            TokenKind::Indent => r"[ ]{2}",
            TokenKind::Percent => r"%",

            TokenKind::And => r"and",
            TokenKind::Class => r"Class",
            TokenKind::Else => r"else",
            TokenKind::False => r"False",
            TokenKind::Def => r"def",
            TokenKind::For => r"for",
            TokenKind::If => r"if",
            TokenKind::Nil => r"None",
            TokenKind::Or => r"or",
            TokenKind::Print => r"print",
            TokenKind::Return => r"return",
            TokenKind::Super => r"super",
            TokenKind::Selph => r"self",
            TokenKind::True => r"True",
            TokenKind::While => r"while",
            TokenKind::Pass => r"pass",
            TokenKind::In => r"in",
        }
    }
}

impl Token {
    pub fn from_token_kind(kind: TokenKind) -> Self {
        Token {
            kind,
            ..Default::default()
        }
    }
}
