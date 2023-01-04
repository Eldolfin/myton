pub mod token;


use token::*;
use super::traceback::Traceback;
use strum::IntoEnumIterator;

pub struct Lexer {
    input: String,
    pub position: (usize,usize),
    pub idx: usize,
    ignored_tokens: Vec<TokenKind>,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        let lexer = Lexer{
            input,
            position: (0, 1),
            idx: 0,
            ignored_tokens: vec![TokenKind::Space, TokenKind::Comment],
        };
        lexer
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Traceback> {
        let mut tokens: Vec<Token> = Vec::new();

        while tokens.last().map(|t| t.kind) != Some(TokenKind::Eof) {
            let res = self.step();
            if let Some(token) = res {
                if !self.ignored_tokens.contains(&token.kind){
                    tokens.push(token);
                }
            } else {
                return Err(Traceback {
                    pos: self.position,
                    message: Some("invalid syntax".to_string()),
                    code: Some(self.input.clone()),
                    ..Default::default()
                });
            }
        }

        if tokens.len() > 1 && tokens[tokens.len()-2].kind != TokenKind::Newline {
            tokens.insert(tokens.len()-1, 
                Token {
                    kind: TokenKind::Newline,
                    value: "".to_string(),
                    pos: None,
                });
        }

        return Ok(tokens);
    }

    fn step(&mut self) -> Option<Token>{
        let mut matches: Vec<(TokenKind, String)> = Vec::new();
        
        for kind in TokenKind::iter(){
            
            let re = kind.matcher();
            
            if re.is_match(&self.input[self.idx..]){
                let value = re.captures(&self.input[self.idx..])
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

        let (kind, mut value) = matches.iter()
            .max_by_key(|(_, value)| value.len())
            .unwrap()
            .clone();

        let token_pos = self.position;

        self.idx += value.len();
        self.position.0 += value.len();
        if kind == TokenKind::Newline {
            self.position.1 += 1;
            self.position.0 = 0;
        }

        if kind == TokenKind::Stringue {
            value.remove(0);
            value.pop();
        }

        return Some(Token {kind, value, pos: Some(token_pos)});
    }
}

#[cfg(test)]
mod tests {
    use crate::myton::errors::report_trace;

    use super::*;

    fn vec_compare<T: PartialEq>(a: Vec<T>, b: Vec<T>) -> bool {
        (a.len() == b.len()) && a.iter().zip(b.iter()).all(|(x, y)| x == y)
    }

    fn test_lexer_case(input: &str, expected: Vec<Token>) {
        let mut lexer = Lexer::new(input.to_string());
        let lex_res = lexer.tokenize();
        assert!(lex_res.is_ok(), "Lexer failed to tokenize, error: {:?}", report_trace(lex_res.err().unwrap()));
        let tokens = lex_res.ok().unwrap();
        let diff = tokens.iter().zip(expected.iter()).filter(|(a, b)| a != b).collect::<Vec<_>>();
        let message = format!("Lexer failed to tokenize correctly, diff: {:?}", diff);
        assert!(vec_compare(tokens.clone(), expected.clone()), "{}", message);
    }

    #[test]
    fn test_lexer() {
        test_lexer_case(
            "1+2",
            vec![
                Token{kind: TokenKind::Number, value: "1".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Plus, value: "+".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Number, value: "2".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Newline, value: "".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Eof, value: "".to_string(), pos: Some((0,0))},
            ]
        );

        test_lexer_case(
            "(1*2) + 3 - 4/1.2",
            vec![
                Token{kind: TokenKind::LeftParen, value: "(".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Number, value: "1".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Star, value: "*".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Number, value: "2".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::RightParen, value: ")".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Plus, value: "+".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Number, value: "3".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Minus, value: "-".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Number, value: "4".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Slash, value: "/".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Number, value: "1.2".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Newline, value: "".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Eof, value: "".to_string(), pos: Some((0,0))},
            ]
        );

        test_lexer_case(
            "var.1=(b!=c<>>=d)",
            vec![
                Token{kind: TokenKind::Identifier, value: "var".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Dot, value: ".".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Number, value: "1".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Equal, value: "=".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::LeftParen, value: "(".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Identifier, value: "b".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::BangEqual, value: "!=".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Identifier, value: "c".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Less, value: "<".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Greater, value: ">".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::GreaterEqual, value: ">=".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Identifier, value: "d".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::RightParen, value: ")".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Newline, value: "".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Eof, value: "".to_string(), pos: Some((0,0))},
            ]
        );

        test_lexer_case(
            "\"hello world\" # this is a comment",
            vec![
                Token{kind: TokenKind::Stringue, value: "hello world".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Newline, value: "".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Eof, value: "".to_string(), pos: Some((0,0))},
            ]
        );

        test_lexer_case(
            "def main() -> int<>:",
            vec![
                Token{kind: TokenKind::Def, value: "def".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Identifier, value: "main".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::LeftParen, value: "(".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::RightParen, value: ")".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Minus, value: "-".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Greater, value: ">".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Identifier, value: "int".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Less, value: "<".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Greater, value: ">".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Colon, value: ":".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Newline, value: "".to_string(), pos: Some((0,0))},
                Token{kind: TokenKind::Eof, value: "".to_string(), pos: Some((0,0))},
            ]
        );
    }
}
