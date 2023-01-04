use super::token::{TokenKind, Token};
mod ast;
use ast::*;

pub fn parse(tokens: Vec<Token>) -> Result<Box<dyn Expression>, String> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
        }
    }

    fn parse(&mut self) -> Result<Box<dyn Expression>, String> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Box<dyn Expression>, String> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Box<dyn Expression>, String> {
        let mut expr = self.comparison()?;
        while self.match_token(vec![TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Box::new(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn match_token(&mut self, token_types: Vec<TokenKind>) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peektype() == token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peektype() == TokenKind::Eof
    }

    fn peektype(&self) -> TokenKind {
        self.tokens[self.current].kind.clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn comparison(&mut self) -> Result<Box<dyn Expression>, String> {
        let mut expr = self.term()?;
        while self.match_token(vec![TokenKind::Greater, TokenKind::GreaterEqual, TokenKind::Less, TokenKind::LessEqual]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Box::new(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<dyn Expression>, String> {
        let mut expr = self.factor()?;
        while self.match_token(vec![TokenKind::Plus, TokenKind::Minus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Box::new(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Box<dyn Expression>, String> {
        let mut expr = self.unary()?;
        while self.match_token(vec![TokenKind::Star, TokenKind::Slash]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Box::new(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<dyn Expression>, String> {
        if self.match_token(vec![TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Box::new(Unary::new(operator, right)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Box<dyn Expression>, String> {
        if self.match_token(vec![TokenKind::False]) {
            return Ok(Box::new(Literal{token: self.previous()}));
        }
        if self.match_token(vec![TokenKind::True]) {
            return Ok(Box::new(Literal{token: self.previous()}));
        }
        if self.match_token(vec![TokenKind::Nil]) {
            return Ok(Box::new(Literal{token: self.previous()}));
        }
        if self.match_token(vec![TokenKind::Number, TokenKind::Stringue]) {
            return Ok(Box::new(Literal{token: self.previous()}));
        }
        if self.match_token(vec![TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RightParen, "Expect ')' after expression.")?;
            return Ok(Box::new(Grouping{expression: expr}));
        }
        Err(format!("Expect expression."))
    }

    fn consume(&mut self, token_type: TokenKind, message: &str) -> Result<Token, String> {
        if self.check(token_type) {
            return Ok(self.advance());
        }
        Err(format!("{}", message))
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Newline {
                return;
            }
            match self.peektype() {
                TokenKind::Class => return,
                TokenKind::Def => return,
                TokenKind::For => return,
                TokenKind::If => return,
                TokenKind::While => return,
                TokenKind::Print => return,
                TokenKind::Return => return,
                _ => (),
            }
            self.advance();
        }
    }
}
