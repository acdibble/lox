use crate::expr::{self, Expr};
use crate::scanner::*;
use crate::stmt::{self, FunctionKind, Stmt};

#[derive(Copy, Clone, PartialEq)]
enum Loop {
    None,
    While,
    For,
}

struct Parser<'a> {
    tokens: &'a Vec<Token<'a>>,
    current: usize,
    last_line: i32,
    had_error: bool,
    panic_mode: bool,

    function_kind: FunctionKind,
    loop_kind: Loop,
}

type ParseResult<T> = std::result::Result<T, ()>;

impl<'a> Parser<'a> {
    fn new(tokens: &'a Vec<Token<'a>>) -> Parser<'a> {
        Parser {
            tokens: tokens,
            current: 0,
            last_line: tokens.last().unwrap().line,
            had_error: false,
            panic_mode: false,
            function_kind: FunctionKind::Script,
            loop_kind: Loop::None,
        }
    }

    fn peek(&self) -> Option<&'a Token<'a>> {
        self.tokens.get(self.current)
    }

    fn is_at_end(&self) -> bool {
        self.peek().is_none()
    }

    fn previous(&self) -> Option<&'a Token<'a>> {
        self.tokens.get(self.current - 1)
    }

    fn advance(&mut self) -> &'a Token<'a> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().unwrap()
    }

    fn check(&self, desired: TokenKind) -> bool {
        match self.peek() {
            Some(Token { kind, .. }) if *kind == desired => true,
            _ => false,
        }
    }

    fn match_current(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            return true;
        }

        false
    }

    fn error(&mut self, token: Option<&Token<'a>>, message: &'static str) {
        if self.panic_mode {
            return;
        }

        let line = if let Some(t) = token {
            t.line
        } else {
            self.last_line
        };

        eprint!("[line {}] Error", line);

        if token.is_none() {
            eprint!(" at end");
        } else if token.unwrap().kind != TokenKind::Error {
            eprint!(" at '{}'", token.unwrap().lexeme);
        }

        eprintln!(": {}", message);
        self.panic_mode = true;
        self.had_error = true;
    }

    fn consume(&mut self, kind: TokenKind, message: &'static str) -> ParseResult<&'a Token<'a>> {
        if self.check(kind) {
            self.advance();
            return Ok(self.previous().unwrap());
        }

        self.error(self.peek(), message);
        Err(())
    }

    fn declaration(&mut self) -> ParseResult<Stmt<'a>> {
        if self.match_current(TokenKind::Fun) {
            return self.function(FunctionKind::Function);
        }
        if self.match_current(TokenKind::Var) {
            return self.var_declaration();
        }

        self.statement()
    }

    fn function(&mut self, kind: FunctionKind) -> ParseResult<Stmt<'a>> {
        let enclosing_kind = self.function_kind;
        self.function_kind = kind;

        let name = self.consume(TokenKind::Identifier, "Expect function name.")?;

        self.consume(TokenKind::LeftParen, "Expect '(' after function name")?;

        let mut params: Vec<&'a Token<'a>> = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                if params.len() >= 255 {
                    self.error(self.peek(), "Can't have more than 255 parameters.");
                }

                params.push(self.consume(TokenKind::Identifier, "Expect parameter name.")?);

                if !self.match_current(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "Expect ')' after parameters.")?;
        self.consume(TokenKind::LeftBrace, "Expect '{' before function body.")?;

        let body = self.block()?;

        self.function_kind = enclosing_kind;

        Ok(Stmt::Function(stmt::Function {
            name,
            params,
            body,
            kind,
            brace: self.previous().unwrap(),
        }))
    }

    fn statement(&mut self) -> ParseResult<Stmt<'a>> {
        if self.match_current(TokenKind::For) {
            return self.for_statement();
        }
        if self.match_current(TokenKind::If) {
            return self.if_statement();
        }
        if self.match_current(TokenKind::Print) {
            return self.print_statement();
        }
        if self.match_current(TokenKind::Return) {
            return self.return_statement();
        }
        if self.match_current(TokenKind::While) {
            return self.while_statement();
        }
        if self.match_current(TokenKind::LeftBrace) {
            return self.block_statement();
        }
        // if self.match_current(TokenKind::Break) {
        //     return self.break_statement();
        // }
        // if self.match_current(TokenKind::Continue) {
        //     return self.continue_statement();
        // }
        self.expression_statement()
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt<'a>> {
        let name = self.consume(TokenKind::Identifier, "Expect variable name.")?;

        let initializer = if self.match_current(TokenKind::Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var(stmt::Var { name, initializer }))
    }

    fn for_statement(&mut self) -> ParseResult<Stmt<'a>> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_current(TokenKind::Semicolon) {
            None
        } else if self.match_current(TokenKind::Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(TokenKind::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenKind::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if !self.check(TokenKind::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenKind::RightParen, "Expect ')' after for clauses")?;

        let enclosing_loop = self.loop_kind;
        self.loop_kind = Loop::For;
        let body = Box::from(self.statement()?);
        self.loop_kind = enclosing_loop;

        Ok(Stmt::For(stmt::For {
            initializer: initializer.map(|stmt| Box::from(stmt)),
            condition,
            increment,
            body,
        }))
    }

    fn if_statement(&mut self) -> ParseResult<Stmt<'a>> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after condition.")?;

        let then_branch = Box::from(self.statement()?);
        let else_branch = if self.match_current(TokenKind::Else) {
            Some(Box::from(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If(stmt::If {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn print_statement(&mut self) -> ParseResult<Stmt<'a>> {
        let keyword = self.previous().unwrap();
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(stmt::Print {
            keyword,
            expression: expr,
        }))
    }

    fn return_statement(&mut self) -> ParseResult<Stmt<'a>> {
        let keyword = self.previous().unwrap();
        let value = if !self.check(TokenKind::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenKind::Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return(stmt::Return { keyword, value }))
    }

    fn while_statement(&mut self) -> ParseResult<Stmt<'a>> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after condition.")?;

        let enclosing_loop = self.loop_kind;
        self.loop_kind = Loop::While;
        let body = Box::from(self.statement()?);
        self.loop_kind = enclosing_loop;

        Ok(Stmt::While(stmt::While { condition, body }))
    }

    fn block(&mut self) -> ParseResult<Vec<Stmt<'a>>> {
        let mut statements: Vec<Stmt<'a>> = Vec::new();

        while !self.is_at_end() && !self.check(TokenKind::RightBrace) {
            statements.push(self.declaration()?);
        }

        self.consume(TokenKind::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn block_statement(&mut self) -> ParseResult<Stmt<'a>> {
        Ok(Stmt::Block(stmt::Block {
            statements: self.block()?,
        }))
    }

    // fn break_statement(&mut self) -> ParseResult<Stmt<'a>> {
    //     if self.loop_kind == Loop::None {
    //         self.error(self.previous(), "Unexpected 'break' statement.");
    //     }
    //     self.consume(TokenKind::Semicolon, "Expect ';' after 'break'.")?;
    //     Ok(Stmt::Break(stmt::Break {
    //         keyword: self.previous().unwrap(),
    //     }))
    // }

    // fn continue_statement(&mut self) -> ParseResult<Stmt<'a>> {
    //     if self.loop_kind == Loop::None {
    //         self.error(self.previous(), "Unexpected 'continue' statement.");
    //     }
    //     self.consume(TokenKind::Semicolon, "Expect ';' after 'continue'.")?;
    //     Ok(Stmt::Continue(stmt::Continue {
    //         keyword: self.previous().unwrap(),
    //     }))
    // }

    fn expression_statement(&mut self) -> ParseResult<Stmt<'a>> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(stmt::Expression { expression: expr }))
    }

    fn expression(&mut self) -> ParseResult<Expr<'a>> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<Expr<'a>> {
        let expr = self.or()?;

        if self.match_current(TokenKind::Equal) {
            let equals = self.previous().unwrap();
            let value = self.assignment()?;

            if let Expr::Variable(expr::Variable { name, .. }) = expr {
                return Ok(Expr::Assign(expr::Assign {
                    name: name,
                    value: Box::from(value),
                }));
            }

            self.error(Some(equals), "Invalid assignment target.");
        }

        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<Expr<'a>> {
        let mut expr = self.and()?;

        while self.match_current(TokenKind::Or) {
            let operator = self.previous().unwrap();
            let right = self.and()?;
            expr = Expr::Logical(expr::Logical {
                left: Box::from(expr),
                operator,
                right: Box::from(right),
            })
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Expr<'a>> {
        let mut expr = self.equality()?;

        while self.match_current(TokenKind::And) {
            let operator = self.previous().unwrap();
            let right = self.and()?;
            expr = Expr::Logical(expr::Logical {
                left: Box::from(expr),
                operator,
                right: Box::from(right),
            })
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Expr<'a>> {
        let mut expr = self.comparison()?;

        while self.match_current(TokenKind::EqualEqual) || self.match_current(TokenKind::BangEqual)
        {
            let operator = self.previous().unwrap();
            let right = Box::from(self.equality()?);
            expr = Expr::Binary(expr::Binary {
                left: Box::from(expr),
                operator,
                right,
            })
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expr<'a>> {
        let mut expr = self.term()?;

        while self.match_current(TokenKind::Greater)
            || self.match_current(TokenKind::GreaterEqual)
            || self.match_current(TokenKind::Less)
            || self.match_current(TokenKind::LessEqual)
        {
            let operator = self.previous().unwrap();
            let right = Box::from(self.term()?);
            expr = Expr::Binary(expr::Binary {
                left: Box::from(expr),
                operator,
                right,
            })
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Expr<'a>> {
        let mut expr = self.factor()?;

        while self.match_current(TokenKind::Plus) || self.match_current(TokenKind::Minus) {
            let operator = self.previous().unwrap();
            let right = self.factor()?;
            expr = Expr::Binary(expr::Binary {
                left: Box::from(expr),
                operator,
                right: Box::from(right),
            })
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expr<'a>> {
        let mut expr = self.unary()?;

        while self.match_current(TokenKind::Star) || self.match_current(TokenKind::Slash) {
            let operator = self.previous().unwrap();
            let right = self.unary()?;
            expr = Expr::Binary(expr::Binary {
                left: Box::from(expr),
                operator,
                right: Box::from(right),
            })
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expr<'a>> {
        if self.match_current(TokenKind::Bang) || self.match_current(TokenKind::Minus) {
            let operator = self.previous().unwrap();
            let right = self.unary()?;
            return Ok(Expr::Unary(expr::Unary {
                operator,
                right: Box::from(right),
            }));
        }

        self.call()
    }

    fn finish_call(&mut self, callee: Expr<'a>) -> ParseResult<Expr<'a>> {
        let mut args: Vec<Expr<'a>> = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                if args.len() >= 255 {
                    self.error(self.peek(), "Can't have more than 255 arguments.")
                }

                args.push(self.expression()?);
                if !self.match_current(TokenKind::Comma) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenKind::RightParen, "Expect ')' after args.")?;

        Ok(Expr::Call(expr::Call {
            callee: Box::from(callee),
            paren,
            args,
        }))
    }

    fn call(&mut self) -> ParseResult<Expr<'a>> {
        let mut expr = self.primary()?;

        if self.match_current(TokenKind::LeftParen) {
            expr = self.finish_call(expr)?;
        }

        Ok(expr)
    }

    fn primary(&mut self) -> ParseResult<Expr<'a>> {
        if let Some(token) = self.peek() {
            match token.kind {
                TokenKind::False
                | TokenKind::True
                | TokenKind::Nil
                | TokenKind::Number
                | TokenKind::String => {
                    self.advance();
                    return Ok(Expr::Literal(expr::Literal { value: token }));
                }
                _ => (),
            }
        }

        if self.match_current(TokenKind::Identifier) {
            return Ok(Expr::Variable(expr::Variable {
                name: self.previous().unwrap(),
            }));
        }

        if self.match_current(TokenKind::LeftParen) {
            let expr = Box::from(self.expression()?);
            self.consume(TokenKind::RightParen, "Expect ')' after expression")?;
            return Ok(Expr::Grouping(expr::Grouping { expr }));
        }

        self.error(self.peek(), "Expected expression.");
        Err(())
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().unwrap().kind == TokenKind::Semicolon {
                return;
            }

            match self.peek().unwrap().kind {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => return,
                _ => (),
            }

            self.advance();
        }
    }
}

pub fn parse_tokens<'a>(tokens: &'a Vec<Token<'a>>) -> Option<Vec<Stmt<'a>>> {
    let mut parser = Parser::new(tokens);
    let mut statements: Vec<Stmt<'a>> = Default::default();
    while !parser.is_at_end() {
        match parser.declaration() {
            Ok(stmt) => statements.push(stmt),
            Err(_) => {
                parser.synchronize();
            }
        }
    }

    if parser.had_error {
        None
    } else {
        Some(statements)
    }
}
