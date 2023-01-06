mod ast;

use super::token::{TokenKind, Token};
use super::traceback::Traceback;
use super::types::DynValue;
use ast::*;

pub fn parse(tokens: Vec<Token>) -> ParseResult {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

type ParseResult = Result<Vec<Box<dyn Statement>>, Traceback>;

impl Parser {
    fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
        }
    }

    fn parse(&mut self) -> ParseResult {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Box<dyn Statement>, Traceback> {
        if self.match_token(vec![TokenKind::Identifier]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Box<dyn Statement>, Traceback> {
        let name = self.previous().value;

        let mut initializer = None;

        if self.match_token(vec![TokenKind::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(TokenKind::Newline, "Expect newline after variable declaration.")?;
        Ok(Box::new(VarStatement {
            name,
            initializer,
        }))
    }

    fn statement(&mut self) -> Result<Box<dyn Statement>, Traceback> {
        if self.match_token(vec![TokenKind::If]) {
            self.if_statement()
        } else if self.match_token(vec![TokenKind::While]) {
            self.while_statement()
        } else if self.match_token(vec![TokenKind::For]) {
            self.for_statement()
        } else if self.match_token(vec![TokenKind::Print]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn while_statement(&mut self) -> Result<Box<dyn Statement>, Traceback> {
        let condition = self.expression()?;
        self.consume(TokenKind::Colon, "Expect ':' after while condition.")?;
        self.consume(TokenKind::Newline, "Expect newline after while condition.")?;
        let body = self.block_statement()?;

        Ok(Box::new(WhileStatement {
            condition,
            body,
        }))
    }

    fn for_statement(&mut self) -> Result<Box<dyn Statement>, Traceback> {
        let variable = self.consume(TokenKind::Identifier, "Expect variable name.")?.value;
        self.consume(TokenKind::In, "Expect 'in' after variable name.")?;
        let collection = self.expression()?;
        self.consume(TokenKind::Colon, "Expect ':' after for collection.")?;
        self.consume(TokenKind::Newline, "Expect newline after for collection.")?;
        let body = self.block_statement()?;

        Ok(Box::new(ForeachStatement {
            variable,
            collection,
            body,
        }))
    }

    fn if_statement(&mut self) -> Result<Box<dyn Statement>, Traceback> {
        let condition = self.expression()?;
        self.consume(TokenKind::Colon, "Expect ':' after if condition.")?;
        self.consume(TokenKind::Newline, "Expect newline after if condition.")?;
        let then_branch = self.block_statement()?;

        let else_branch = if self.match_token(vec![TokenKind::Else]) {
            self.consume(TokenKind::Colon, "Expect ':' after else.")?;
            self.consume(TokenKind::Newline, "Expect newline after else.")?;
            Some(self.block_statement()?)
        } else { None };

        Ok(Box::new(IfStatement {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn block_statement(&mut self) -> Result<Box<dyn Statement>, Traceback> {
        let indent_level = self.previous().indent;
        let mut statements = Vec::new();
        while !self.is_at_end() && self.peek().indent > indent_level {
            statements.push(self.declaration()?);
        }
        Ok(Box::new(BlockStatement {
            statements,
        }))
    }

    fn print_statement(&mut self) -> Result<Box<dyn Statement>, Traceback> {
        let expression = self.expression()?;
        self.consume(TokenKind::Newline, "Expect newline after expression.")?;
        Ok(Box::new(PrintStatement { expression }))
    }

    fn expression_statement(&mut self) -> Result<Box<dyn Statement>, Traceback> {
        let expression = self.expression()?;
        self.consume(TokenKind::Newline, "Expect newline after expression.")?;
        Ok(Box::new(ExpressionStatement { expression }))
    }

    fn expression(&mut self) -> Result<Box<dyn Expression>, Traceback> {
        self.or()
    }

    fn or(&mut self) -> Result<Box<dyn Expression>, Traceback> {
        let mut expr = self.and()?;

        while self.match_token(vec![TokenKind::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Box::new(Logical::new(expr, operator, right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Box<dyn Expression>, Traceback> {
        let mut expr = self.equality()?;

        while self.match_token(vec![TokenKind::And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Box::new(Logical::new(expr, operator, right));
        }

        Ok(expr)
    }


    fn equality(&mut self) -> Result<Box<dyn Expression>, Traceback> {
        let mut expr = self.comparison()?;
        while self.match_token(vec![TokenKind::BangEqual, TokenKind::EqualEqual, TokenKind::EqualEqualEqual]) {
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

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        if self.current == 0 {
            self.tokens[0].clone()
        } else {
            self.tokens[self.current - 1].clone()
        }
    }

    fn comparison(&mut self) -> Result<Box<dyn Expression>, Traceback> {
        let mut expr = self.term()?;
        while self.match_token(vec![TokenKind::Greater, TokenKind::GreaterEqual, TokenKind::Less, TokenKind::LessEqual]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Box::new(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<dyn Expression>, Traceback> {
        let mut expr = self.factor()?;
        while self.match_token(vec![TokenKind::Plus, TokenKind::Minus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Box::new(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Box<dyn Expression>, Traceback> {
        let mut expr = self.unary()?;
        while self.match_token(vec![TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Box::new(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<dyn Expression>, Traceback> {
        if self.match_token(vec![TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Box::new(Unary::new(operator, right)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Box<dyn Expression>, Traceback> {
        if self.match_token(vec![TokenKind::Number, TokenKind::Stringue, TokenKind::False, TokenKind::True, TokenKind::Nil]) {
            return Ok(Box::new(Literal::new(self.previous())));
        }
        if self.match_token(vec![TokenKind::Pass]) {
            let mut token = self.previous();
            token.kind = TokenKind::Nil;
            return Ok(Box::new(Literal::new(token)));
        }
        if self.match_token(vec![TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RightParen, "Expect ')' after expression.")?;
            return Ok(Box::new(Grouping{expression: expr}));
        }
        if self.match_token(vec![TokenKind::Identifier]) {
            return Ok(Box::new(Variable{token: self.previous()}));
        }
        if self.match_token(vec![TokenKind::LeftBracket]) {
            let mut elements = Vec::new();
            while {
                elements.push(self.expression()?);
                self.consume(TokenKind::Comma, "Expect ',' after expression.").is_ok()
            } {}

            self.consume(TokenKind::RightBracket, "Expect ']' after expression.")?;
            return Ok(Box::new(List{elements}));
        }

        Err(Traceback{pos: self.peek().pos.unwrap_or_default(), message: Some("Expect expression.".to_string()), ..Default::default()})
    }

    fn consume(&mut self, token_type: TokenKind, message: &str) -> Result<Token, Traceback> {
        if self.check(token_type) {
            return Ok(self.advance());
        }
        Err(Traceback {
            message: Some(format!("{}", message)),
            pos: self.previous().pos.unwrap(),
            ..Default::default()
        })
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
