pub mod token;
use token::*;
use strum::IntoEnumIterator;

pub struct Lexer {
    input: String,
    pub position: usize,
    pub line_number: usize,
    pub line_position: usize,
    ignored_tokens: Vec<TokenKind>,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        let lexer = Lexer{
            input,
            position: 0,
            line_position: 0,
            line_number: 1,
            ignored_tokens: vec![TokenKind::Space, TokenKind::Comment],
        };
        lexer
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens: Vec<Token> = Vec::new();

        while tokens.last().map(|t| t.kind) != Some(TokenKind::Eof) {
            let res = self.step();
            if let Some(token) = res {
                if !self.ignored_tokens.contains(&token.kind){
                    tokens.push(token);
                }
            } else {
                //TODO better error handling, ex:
                //   --> /path/to/file.my:75:17
                //     |
                // 75  | def main() -> int<>:
                //     |                  ^
                // help: unexpected token '<'
                let err = format!("Unexpected token at position {}", self.position);
                return Err(err);
            }
        }

        return Ok(tokens);
    }

    fn step(&mut self) -> Option<Token>{
        let mut matches: Vec<(TokenKind, String)> = Vec::new();
        
        for kind in TokenKind::iter(){
            
            let re = kind.matcher();
            
            if re.is_match(&self.input[self.position..]){
                let value = re.captures(&self.input[self.position..])
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .as_str()
                    .to_string();
                
                matches.push((kind, value));
            }
        }

        if matches.len() == 0 {
            return None;
        }

        if matches.len() > 1 {
            // If there is an identifier, it should be the only match
            matches.retain(|(kind, _)| kind != &TokenKind::Identifier);
        }

        let (kind, value) = matches.iter()
            .max_by_key(|(_, value)| value.len())
            .unwrap()
            .clone();

        self.position += value.len();
        self.line_position += value.len();
        if kind == TokenKind::Newline {
            self.line_number += 1;
            self.line_position = 0;
        }

        return Some(Token {kind, value});
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec_compare<T: PartialEq>(a: Vec<T>, b: Vec<T>) -> bool {
        (a.len() == b.len()) && a.iter().zip(b.iter()).all(|(x, y)| x == y)
    }

    fn test_lexer_case(input: &str, expected: Vec<Token>) {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        assert!(vec_compare(tokens.clone(), expected.clone()), "input: {}\nexpected: {:?}\nactual: {:?}", input, expected, tokens);
    }

    #[test]
    fn test_lexer() {
        test_lexer_case(
            "1 + 2",
            vec![
                Token{kind: TokenKind::Number, value: "1".to_string()},
                Token{kind: TokenKind::Plus, value: "+".to_string()},
                Token{kind: TokenKind::Number, value: "2".to_string()},
                Token{kind: TokenKind::Eof, value: "".to_string()},
            ]
        );

        test_lexer_case(
            "(1*2) + 3 - 4/1.2",
            vec![
                Token{kind: TokenKind::LeftParen, value: "(".to_string()},
                Token{kind: TokenKind::Number, value: "1".to_string()},
                Token{kind: TokenKind::Star, value: "*".to_string()},
                Token{kind: TokenKind::Number, value: "2".to_string()},
                Token{kind: TokenKind::RightParen, value: ")".to_string()},
                Token{kind: TokenKind::Plus, value: "+".to_string()},
                Token{kind: TokenKind::Number, value: "3".to_string()},
                Token{kind: TokenKind::Minus, value: "-".to_string()},
                Token{kind: TokenKind::Number, value: "4".to_string()},
                Token{kind: TokenKind::Slash, value: "/".to_string()},
                Token{kind: TokenKind::Number, value: "1.2".to_string()},
                Token{kind: TokenKind::Eof, value: "".to_string()},
            ]
        );

        test_lexer_case(
            "var.1=(b!=c<>>=d)",
            vec![
                Token{kind: TokenKind::Identifier, value: "var".to_string()},
                Token{kind: TokenKind::Dot, value: ".".to_string()},
                Token{kind: TokenKind::Number, value: "1".to_string()},
                Token{kind: TokenKind::Equal, value: "=".to_string()},
                Token{kind: TokenKind::LeftParen, value: "(".to_string()},
                Token{kind: TokenKind::Identifier, value: "b".to_string()},
                Token{kind: TokenKind::BangEqual, value: "!=".to_string()},
                Token{kind: TokenKind::Identifier, value: "c".to_string()},
                Token{kind: TokenKind::Less, value: "<".to_string()},
                Token{kind: TokenKind::Greater, value: ">".to_string()},
                Token{kind: TokenKind::GreaterEqual, value: ">=".to_string()},
                Token{kind: TokenKind::Identifier, value: "d".to_string()},
                Token{kind: TokenKind::RightParen, value: ")".to_string()},
                Token{kind: TokenKind::Eof, value: "".to_string()},
            ]
        );

        test_lexer_case(
            "\"hello world\" # this is a comment",
            vec![
                Token{kind: TokenKind::Stringue, value: "\"hello world\"".to_string()},
                Token{kind: TokenKind::Eof, value: "".to_string()},
            ]
        );

        test_lexer_case(
            "def main() -> int<>:",
            vec![
                Token{kind: TokenKind::Def, value: "def".to_string()},
                Token{kind: TokenKind::Identifier, value: "main".to_string()},
                Token{kind: TokenKind::LeftParen, value: "(".to_string()},
                Token{kind: TokenKind::RightParen, value: ")".to_string()},
                Token{kind: TokenKind::Minus, value: "-".to_string()},
                Token{kind: TokenKind::Greater, value: ">".to_string()},
                Token{kind: TokenKind::Identifier, value: "int".to_string()},
                Token{kind: TokenKind::Less, value: "<".to_string()},
                Token{kind: TokenKind::Greater, value: ">".to_string()},
                Token{kind: TokenKind::Colon, value: ":".to_string()},
                Token{kind: TokenKind::Eof, value: "".to_string()},
            ]
        );
    }
}
