pub mod token;


use token::*;
use super::traceback::Traceback;
use strum::IntoEnumIterator;
use regex::Regex;

pub struct Lexer {
    input: String,
    pub position: (usize,usize),
    pub idx: usize,
    ignored_tokens: Vec<TokenKind>,
    regexs: Vec<Regex>,
    tokens: Vec<Token>,
    cur_indent: usize,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        let lexer = Lexer{
            input,
            position: (0, 1),
            idx: 0,
            ignored_tokens: vec![TokenKind::Space, TokenKind::Comment, TokenKind::Indent],
            regexs: TokenKind::iter().map(|kind| {Regex::new(format!(r"^{}", kind.regex()).as_str()).unwrap()}).collect(),
            tokens: Vec::new(),
            cur_indent: 0,
        };
        lexer
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Traceback> {
        while self.tokens.last().map(|t| t.kind) != Some(TokenKind::Eof) {
            let res = self.step();
            if let Some(mut token) = res {
                token.pos = Some(self.position);
                token.indent = self.cur_indent;
                if !self.ignored_tokens.contains(&token.kind){
                    self.tokens.push(token.clone());
                }
                match token.kind {
                    TokenKind::Newline => {
                        self.cur_indent = 0;
                    },
                    TokenKind::Indent => {
                        self.cur_indent += 1;
                    },
                    _ => {},
                }
            } else {
                return Err(Traceback {
                    pos: self.position,
                    message: Some("invalid syntax".to_string()),
                    ..Default::default()
                });
            }
        }

        if self.tokens.len() > 1 && self.tokens[self.tokens.len()-2].kind != TokenKind::Newline {
            self.tokens.insert(self.tokens.len()-1, 
                Token::from_token_kind(TokenKind::Newline))
        }

        return Ok(self.tokens.clone());
    }

    fn step(&mut self) -> Option<Token>{
        let mut matches: Vec<(TokenKind, String)> = Vec::new();
        
        for (kind,re) in TokenKind::iter().zip(self.regexs.iter()) {
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

        if matches.len() > 1 {
            matches.retain(|(kind, _)| *kind != TokenKind::Space);
        }

        if matches.len() == 0 {
            if self.idx == self.input.len() {
                return Some(Token::from_token_kind(TokenKind::Eof));
            } else {
                return None;
            }
        }

        let max_match = matches.iter().map(|(_,v)| v.len()).max().unwrap();

        matches = matches.iter()
            .cloned()
            .filter(|(_, value)| value.len() == max_match)
            .collect::<Vec<_>>();

        if matches.len() > 1 {
            // If there is an identifier, it should be the only match
            matches.retain(|(kind, _)| kind != &TokenKind::Identifier);
        }

        let (kind, mut value) = matches.iter()
            .max_by_key(|(_, value)| value.len())
            .unwrap()
            .clone();

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

        return Some(Token {kind, value, ..Default::default()});
    }
}

#[cfg(test)]
mod tests {
    use crate::myton::errors::report_trace;

    use super::*;
    use TokenKind::*;

    fn test_lexer_case(input: &str, expected: Vec<TokenKind>) {
        let mut lexer = Lexer::new(input.to_string());
        let lex_res = lexer.tokenize();
        assert!(lex_res.is_ok(), "Lexer failed to tokenize {}, \nerror: {:?}", input, report_trace(lex_res.err().unwrap()));
        let tokens = lex_res.ok().unwrap();

        let diff = tokens.iter().zip(expected.iter()).filter(|(a, b)| a.kind != **b).collect::<Vec<_>>();
        let mut message = format!("Lexer failed to tokenize correctly \"{}\", diff: {:?}\n", input, diff);
        message.push_str("got:expected\n");
        for (token, kind) in tokens.iter().zip(expected.iter()) {
            message.push_str(&format!("{:?}:{:?}\n", token.kind, kind));
        }


        assert!(diff.len()==0, "{}", message);
    }

    #[test]
    fn test_lexer() {
        test_lexer_case(
            "1+2",
            vec![
                Number,
                Plus,
                Number,
                Newline,
                Eof
            ]
        );

        test_lexer_case(
            "(1*2) + 3 - 4/1.2",
            vec![
                LeftParen,
                Number,
                Star,
                Number,
                RightParen,
                Plus,
                Number,
                Minus,
                Number,
                Slash,
                Number,
                Newline,
                Eof
            ]
        );

        test_lexer_case(
            "var.1=(b!=c<>>=d)",
            vec![
                Identifier,
                Dot,
                Number,
                Equal,
                LeftParen,
                Identifier,
                BangEqual,
                Identifier,
                Less,
                Greater,
                GreaterEqual,
                Identifier,
                RightParen,
                Newline,
                Eof
            ]
        );

        test_lexer_case(
            "\"hello world\" # this is a comment\n# this is another comment\n print # this is a comment",
            vec![
                Stringue,
                Newline,
                Newline,
                Print,
                Newline,
                Eof
            ]
        );

        test_lexer_case(
            "def main() -> int<>:",
            vec![
                Def,
                Identifier,
                LeftParen,
                RightParen,
                Minus,
                Greater,
                Identifier,
                Less,
                Greater,
                Colon,
                Newline,
                Eof
            ]
        );

        test_lexer_case(
            "a=1\nb=2",
            vec![
                Identifier,
                Equal,
                Number,
                Newline,
                Identifier,
                Equal,
                Number,
                Newline,
                Eof
            ]
        );

        test_lexer_case(
            "if 1:\n    print 1\nelse:\n    print \"lol\"",
            vec![
                If,
                Number,
                Colon,
                Newline,
                Print,
                Number,
                Newline,
                Else,
                Colon,
                Newline,
                Print,
                Stringue,
                Newline,
                Eof
            ]
        );

        test_lexer_case(
            "for i in [1,2,3]:", 
            vec![
                For,
                Identifier,
                In,
                LeftBracket,
                Number,
                Comma,
                Number,
                Comma,
                Number,
                RightBracket,
                Colon,
                Newline,
                Eof
            ]
        )
    }
}
