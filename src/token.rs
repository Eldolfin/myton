use regex::Regex;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use std::str::Chars;

#[derive(Debug)]
pub struct Token{
    kind: TokenKind,
    value: String,
}

#[derive(Debug, EnumIter)]
pub enum TokenKind{
    Int,
    Float,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

impl TokenKind {
    fn regex(&self) -> String {
        match self {
            TokenKind::Int => r"\d+".to_string(),
            TokenKind::Float => r"\d+\.\d+".to_string(),
            TokenKind::Plus => r"\+".to_string(),
            TokenKind::Minus => r"-".to_string(),
            TokenKind::Star => r"\*".to_string(),
            TokenKind::Slash => r"/".to_string(),
            TokenKind::LParen => r"\(".to_string(),
            TokenKind::RParen => r"\)".to_string(),
        }
    }
}

pub struct Lexer {
    input: String,
    position: usize,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        let lexer = Lexer{
            input,
            position: 0,
        };
        lexer
    }
    
    pub fn tokenize(&mut self) -> Vec<Token>{
        let mut tokens = Vec::new();

        while let Some(token) = self.step(){
            tokens.push(token);
        }

        tokens
    }

    fn step(&mut self) -> Option<Token>{
        for kind in TokenKind::iter(){
            let re = Regex::new(&kind.regex()).unwrap();
            if re.is_match(&self.input[self.position..]){
                let value = re.captures(&self.input[self.position..]).unwrap().get(0).unwrap().as_str().to_string();
                self.position += value.len();
                print!("{:?}", self.position);
                return Some(Token{
                    kind,
                    value,
                });
            }
        }
        None
    }
}
