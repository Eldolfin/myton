use regex::Regex;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token{
    pub kind: TokenKind,
    pub value: String,
}

#[derive(Debug, EnumIter, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    LeftParen, RightParen,
    Comma, Dot, Plus, Minus, Slash, Star, Colon,
    Identifier, Stringue, Number,

    BangEqual, Bang,
    EqualEqual, Equal,
    GreaterEqual, Greater,
    LessEqual, Less,

    And, Class, Else, False, Def, For, If, Nil, Or, 
    Print, Return, Super, Selph, True, While,

    Comment,
    Space,
    Newline,

    Eof,
}


impl TokenKind {
    fn regex(&self) -> &str {
        match self {
            TokenKind::Number => r"\d+(\.\d+)?",
            TokenKind::Plus => r"\+",
            TokenKind::Minus => r"-",
            TokenKind::Star => r"\*",
            TokenKind::Slash => r"/",
            TokenKind::LeftParen => r"\(",
            TokenKind::RightParen => r"\)",
            TokenKind::Space => r"\s+",
            TokenKind::Colon => r":",
            TokenKind::Comment => r"#.*$",
            TokenKind::Eof => r"(?m)$",
            TokenKind::Bang => r"!",
            TokenKind::BangEqual => r"!=",
            TokenKind::Equal => r"=",
            TokenKind::EqualEqual => r"==",
            TokenKind::Greater => r">",
            TokenKind::GreaterEqual => r">=",
            TokenKind::Less => r"<",
            TokenKind::LessEqual => r"<=",
            TokenKind::Comma => r",",
            TokenKind::Dot => r"\.",
            TokenKind::Newline => r"$",
            TokenKind::Identifier => r"[a-zA-Z_][a-zA-Z0-9_]*",
            TokenKind::Stringue => r#""[^"]*""#,

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

        }
    }

    pub fn matcher(&self) -> Regex {
        static mut MATCHERS: Vec<Regex> = Vec::new();
        if unsafe { MATCHERS.len() } == 0 {
            for kind in TokenKind::iter() {
                let final_regex = format!(r"^({})", kind.regex());
                unsafe { MATCHERS.push(Regex::new(&final_regex).unwrap()) }
            }
        }

        unsafe { MATCHERS[*self as usize].clone() }
    }
}

