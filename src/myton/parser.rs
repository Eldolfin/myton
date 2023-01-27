use super::expression::*;
use super::statement::*;
use super::token::{Token, TokenKind};
use super::traceback::Traceback;
use super::MyWrite;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    output: Rc<RefCell<Box<dyn MyWrite>>>,
}

type ParseResult = Result<Vec<STMT>, Traceback>;

impl Parser {
    pub fn new(tokens: Vec<Token>, output: Rc<RefCell<Box<dyn MyWrite>>>) -> Parser {
        Parser {
            tokens,
            current: 0,
            output,
        }
    }

    pub fn parse(&mut self) -> ParseResult {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<STMT, Traceback> {
        if self.match_token(vec![TokenKind::Def]) {
            self.function()
        } else if self.check_sequence(vec![TokenKind::Identifier, TokenKind::Equal]) {
            self.var_declaration()
        } else if self.match_token(vec![TokenKind::Class]) {
            self.class()
        } else {
            self.statement()
        }
    }

    fn class(&mut self) -> Result<STMT, Traceback> {
        let indent_level = self.previous().indent; // should be 0 right?
        let name = self.consume(TokenKind::Identifier, "Expect class name.")?;

        let superclass = if self.match_token(vec![TokenKind::LeftParen]) {
            self.consume(TokenKind::Identifier, "Expect superclass name.")?;
            let name = self.previous().clone();
            self.consume(TokenKind::RightParen, "Expect ')' after superclass name.")?;
            Some(Variable::new(name, self.current))
        } else {
            None
        };

        self.consume(TokenKind::Colon, "Expect ':' after class name.")?;
        self.consume(TokenKind::Newline, "Expect newline after class name.")?;

        let mut methods = Vec::new();
        while !self.is_at_end() && self.peek().indent > indent_level {
            self.consume(TokenKind::Def, "Expect 'def' before class method.")?;
            methods.push(self.function_inner()?);
        }
        Ok(Box::new(ClassStatement::new(name, methods, superclass)))
    }

    fn function(&mut self) -> Result<STMT, Traceback> {
        Ok(Box::new(self.function_inner()?))
    }

    fn function_inner(&mut self) -> Result<FunctionStatement, Traceback> {
        let name = self.consume(TokenKind::Identifier, "Expect function name.")?;
        self.consume(TokenKind::LeftParen, "Expect '(' after function name.")?;
        let mut parameters = Vec::new();
        if !self.check(TokenKind::RightParen) {
            while {
                parameters.push(self.consume(TokenKind::Identifier, "Expect parameter name.")?);
                self.match_token(vec![TokenKind::Comma])
            } {}
        }
        self.consume(TokenKind::RightParen, "Expect ')' after parameters.")?;
        self.consume(TokenKind::Colon, "Expect ':' before function body.")?;
        let body = self.block_statement()?;
        Ok(FunctionStatement::new(name, parameters, body))
    }

    fn var_declaration(&mut self) -> Result<STMT, Traceback> {
        let name = self.consume(TokenKind::Identifier, "Expect variable name.")?;
        self.consume(TokenKind::Equal, "Expect '=' after variable name.")?;
        let initializer = self.expression()?;

        self.consume(
            TokenKind::Newline,
            "Expect newline after variable declaration.",
        )?;
        Ok(Box::new(VarStatement { name, initializer }))
    }

    fn statement(&mut self) -> Result<STMT, Traceback> {
        if self.match_token(vec![TokenKind::If]) {
            self.if_statement()
        } else if self.match_token(vec![TokenKind::While]) {
            self.while_statement()
        } else if self.match_token(vec![TokenKind::For]) {
            self.for_statement()
        } else if self.match_token(vec![TokenKind::Print]) {
            self.print_statement()
        } else if self.match_token(vec![TokenKind::Return]) {
            self.return_statement()
        } else if self.match_token(vec![TokenKind::Global]) {
            self.global_statement()
        } else if self.match_token(vec![TokenKind::Nonlocal]) {
            self.nonlocal_statement()
        } else if self.match_token(vec![TokenKind::Newline]) {
            self.empty_statement()
        } else {
            self.expression_statement()
        }
    }

    fn return_statement(&mut self) -> Result<STMT, Traceback> {
        let keyword = self.previous();
        let value = if self.check(TokenKind::Newline) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenKind::Newline, "Expect newline after return value.")?;
        Ok(Box::new(ReturnStatement { keyword, value }))
    }

    fn while_statement(&mut self) -> Result<STMT, Traceback> {
        let condition = self.expression()?;
        self.consume(TokenKind::Colon, "Expect ':' after while condition.")?;
        let body = self.block_statement()?;

        Ok(Box::new(WhileStatement { condition, body }))
    }

    fn for_statement(&mut self) -> Result<STMT, Traceback> {
        let variable = self.consume(TokenKind::Identifier, "Expect variable name.")?;
        self.consume(TokenKind::In, "Expect 'in' after variable name.")?;
        let collection = self.expression()?;
        self.consume(TokenKind::Colon, "Expect ':' after for collection.")?;
        let body = self.block_statement()?;

        Ok(Box::new(ForeachStatement {
            variable,
            collection,
            body,
        }))
    }

    fn if_statement(&mut self) -> Result<STMT, Traceback> {
        let condition = self.expression()?;
        self.consume(TokenKind::Colon, "Expect ':' after if condition.")?;
        let then_branch = self.block_statement()?;

        let else_branch = if self.match_token(vec![TokenKind::Else]) {
            self.consume(TokenKind::Colon, "Expect ':' after else.")?;
            Some(self.block_statement()?)
        } else {
            None
        };

        Ok(Box::new(IfStatement {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn block_statement(&mut self) -> Result<STMT, Traceback> {
        self.consume(TokenKind::Newline, "Expect newline before code block")?;
        let indent_level = self.previous().indent;
        let mut statements = Vec::new();
        while !self.is_at_end() && self.peek().indent > indent_level {
            statements.push(self.declaration()?);
        }
        Ok(Box::new(BlockStatement { statements }))
    }

    fn print_statement(&mut self) -> Result<STMT, Traceback> {
        let expression = self.expression()?;
        self.consume(TokenKind::Newline, "Expect newline after expression.")?;
        Ok(Box::new(PrintStatement {
            expression,
            output: self.output.clone(),
        }))
    }

    fn expression_statement(&mut self) -> Result<STMT, Traceback> {
        let expression = self.expression()?;
        self.consume(TokenKind::Newline, "Expect newline after expression.")?;
        Ok(Box::new(ExpressionStatement { expression }))
    }

    fn expression(&mut self) -> Result<EXPR, Traceback> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<EXPR, Traceback> {
        let expr = self.or()?;

        if self.match_token(vec![TokenKind::Equal]) {
            if let Some(get) = expr.as_any().downcast_ref::<Get>() {
                let value = self.assignment()?;

                return if let Some(var) = get.object.as_any().downcast_ref::<Variable>().cloned() {
                    Ok(Box::new(Set::new(
                        Box::new(var),
                        get.name.clone(),
                        value,
                        self.current,
                    )))
                } else if let Some(this) = get.object.as_any().downcast_ref::<This>().cloned() {
                    Ok(Box::new(Set::new(
                        Box::new(this.clone()),
                        this.keyword.clone(),
                        value,
                        self.current,
                    )))
                } else {
                    Err(Traceback {
                        message: Some("Only instances have fields".to_string()),
                        pos: self.previous().pos.unwrap(),
                        ..Default::default()
                    })
                };
            }
        }

        Ok(expr)
    }

    fn global_statement(&mut self) -> Result<STMT, Traceback> {
        let mut names = Vec::new();
        while {
            names.push(self.consume(TokenKind::Identifier, "Expect identifier after 'global'")?);
            self.match_token(vec![TokenKind::Comma])
        } {}
        self.consume(TokenKind::Newline, "Expect newline after global statement.")?;
        Ok(Box::new(GlobalStatement { names }))
    }

    fn nonlocal_statement(&mut self) -> Result<STMT, Traceback> {
        let mut names = Vec::new();
        while {
            names.push(self.consume(TokenKind::Identifier, "Expect identifier after 'nonlocal'")?);
            self.match_token(vec![TokenKind::Comma])
        } {}
        self.consume(
            TokenKind::Newline,
            "Expect newline after nonlocal statement.",
        )?;
        Ok(Box::new(NonlocalStatement { names }))
    }

    fn or(&mut self) -> Result<EXPR, Traceback> {
        let mut expr = self.and()?;

        while self.match_token(vec![TokenKind::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Box::new(Logical::new(expr, operator, right, self.current));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<EXPR, Traceback> {
        let mut expr = self.equality()?;

        while self.match_token(vec![TokenKind::And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Box::new(Logical::new(expr, operator, right, self.current));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<EXPR, Traceback> {
        let mut expr = self.comparison()?;
        while self.match_token(vec![
            TokenKind::BangEqual,
            TokenKind::EqualEqual,
            TokenKind::EqualEqualEqual,
        ]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Box::new(Binary::new(expr, operator, right, self.current));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<EXPR, Traceback> {
        let mut expr = self.term()?;
        while self.match_token(vec![
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Box::new(Binary::new(expr, operator, right, self.current));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<EXPR, Traceback> {
        let mut expr = self.factor()?;
        while self.match_token(vec![TokenKind::Plus, TokenKind::Minus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Box::new(Binary::new(expr, operator, right, self.current));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<EXPR, Traceback> {
        let mut expr = self.unary()?;
        while self.match_token(vec![TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Box::new(Binary::new(expr, operator, right, self.current));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<EXPR, Traceback> {
        if self.match_token(vec![TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Box::new(Unary::new(operator, right, self.current)));
        }
        self.call()
    }

    fn call(&mut self) -> Result<EXPR, Traceback> {
        let mut expr = self.primary()?;
        loop {
            if self.match_token(vec![TokenKind::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(vec![TokenKind::Dot]) {
                let name =
                    self.consume(TokenKind::Identifier, "Expect property name after '.'.")?;
                expr = Box::new(Get::new(expr, name, self.current));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: EXPR) -> Result<EXPR, Traceback> {
        let mut arguments = Vec::new();
        if !self.check(TokenKind::RightParen) {
            while {
                arguments.push(self.expression()?);
                self.match_token(vec![TokenKind::Comma])
            } {}
        }
        let paren = self.consume(TokenKind::RightParen, "Expect ')' after arguments.")?;
        Ok(Box::new(Call::new(callee, paren, arguments, self.current)))
    }

    fn primary(&mut self) -> Result<EXPR, Traceback> {
        if self.match_token(vec![
            TokenKind::Number,
            TokenKind::Stringue,
            TokenKind::False,
            TokenKind::True,
            TokenKind::Nil,
        ]) {
            return Ok(Box::new(Literal::new(self.previous(), self.current)));
        }
        if self.match_token(vec![TokenKind::Pass]) {
            return self.empty_expression();
        }
        if self.match_token(vec![TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RightParen, "Expect ')' after expression.")?;
            return Ok(Box::new(Grouping::new(expr, self.current)));
        }
        if self.match_token(vec![TokenKind::Identifier]) {
            return Ok(Box::new(Variable::new(self.previous(), self.current)));
        }
        if self.match_token(vec![TokenKind::LeftBracket]) {
            let mut elements = Vec::new();
            if !self.check(TokenKind::RightBracket) {
                while {
                    elements.push(self.expression()?);
                    self.match_token(vec![TokenKind::Comma])
                } {}
            }

            self.consume(TokenKind::RightBracket, "Expect ']' after expression.")?;
            return Ok(Box::new(List::new(elements, self.current)));
        }
        if self.match_token(vec![TokenKind::Selph]) {
            return Ok(Box::new(This::new(self.previous(), self.current)));
        }
        if self.match_token(vec![TokenKind::Super]) {
            let keyword = self.previous();
            self.consume(TokenKind::Dot, "Expect '.' after 'super'.")?;
            let method = self.consume(TokenKind::Identifier, "Expect superclass method name.")?;
            return Ok(Box::new(Super::new(keyword, method, self.current)));
        }

        Err(Traceback {
            pos: self.peek().pos.unwrap_or_default(),
            message: Some("Expect expression.".to_string()),
            ..Default::default()
        })
    }

    fn empty_expression(&mut self) -> Result<EXPR, Traceback> {
        let mut token = self.previous();
        token.kind = TokenKind::Nil;
        Ok(Box::new(Literal::new(token, self.current)))
    }

    fn empty_statement(&mut self) -> Result<STMT, Traceback> {
        Ok(Box::new(ExpressionStatement {
            expression: self.empty_expression()?,
        }))
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

    fn check_sequence(&self, token_types: Vec<TokenKind>) -> bool {
        if self.is_at_end() {
            return false;
        }
        token_types
            .iter()
            .zip(self.tokens.iter().skip(self.current))
            .all(|(a, b)| a == &b.kind)
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

    fn consume(&mut self, token_type: TokenKind, message: &str) -> Result<Token, Traceback> {
        // special case to allow multiple newlines
        if token_type == TokenKind::Newline {
            while self.check(TokenKind::Newline) {
                self.advance();
            }
            return Ok(self.previous());
        }

        if self.check(token_type) {
            return Ok(self.advance());
        }
        Err(Traceback {
            message: Some(format!("{}", message)),
            pos: self.previous().pos.unwrap(),
            ..Default::default()
        })
    }

    #[allow(dead_code)] // #TODO: remove this
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
