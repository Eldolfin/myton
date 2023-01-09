use super::statement::*;
use super::expression::*;
use super::token::Token;
use super::traceback::Traceback;
use std::collections::HashMap;


type ResolveResult = Result<(), Traceback>;
pub type UUID = usize;

pub struct Resolver {
    scopes :Vec<HashMap<String, bool>>,
    locals :HashMap<UUID, usize>, // UUID -> depth
}

pub trait Resolvable {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult;
}



impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            scopes: vec![HashMap::new()],
            locals: HashMap::new(),
        }
    }
    // STATEMENTS

    fn block(&mut self, block: &BlockStatement) -> ResolveResult {
        self.begin_scope();
        self.stmts(&block.statements);
        self.end_scope();
        Ok(())
    }

    fn stmts(&mut self, stmts: &Vec<STMT>) -> ResolveResult {
        for stmt in stmts {
            self.stmt(stmt)?;
        }
        Ok(())
    }

    fn stmt(&mut self, stmt: &STMT) -> ResolveResult {
         stmt.resolve(self)
    }

    
    fn expression_stmt(&mut self, expr: &ExpressionStatement) -> ResolveResult {
        expr.expression.resolve(self)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn var(&mut self, stmt: &VarStatement) -> ResolveResult {
        self.declare(&stmt.name)?;
        stmt.initializer.resolve(self)?;
        self.define(&stmt.name)?;

        Ok(())
    }

    fn declare(&mut self, name: &Token) -> ResolveResult {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.value.clone(), false);
        }

        Ok(())
    }

    fn define(&mut self, name: &Token) -> ResolveResult {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.value.clone(), true);
        }

        Ok(())
    }

    fn var_expr(&mut self, expr: &Variable) -> ResolveResult {
        if let Some(scope) = self.scopes.last() {
            if let Some(false) = scope.get(&expr.token.value) {
                return Err(Traceback::from(format!("Cannot read local variable in its own initializer.")));
            }
        }

        let casted :EXPR = Box::new(expr.clone());

        self.local(&casted, &expr.token.clone());
        Ok(())
    }

    fn local(&mut self, expr: &EXPR, name: &Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.value) {
                self.locals.insert(expr.uuid(), i);
                return;
            }
        }
    }

    fn function(&mut self, function: &FunctionStatement) -> ResolveResult {
        self.declare(&function.inner.borrow().name)?;
        self.define(&function.inner.borrow().name)?;

        self.resolve_function(function)?;

        Ok(())
    }

    fn resolve_function(&mut self, function: &FunctionStatement) -> ResolveResult {
        self.begin_scope();
        for param in &function.inner.borrow().parameters {
            self.declare(param)?;
            self.define(param)?;
        }
        self.stmt(&function.inner.borrow().body)?;
        self.end_scope();
        Ok(())
    }

    fn iff(&mut self, stmt: &IfStatement) -> ResolveResult {
        stmt.condition.resolve(self)?;
        stmt.then_branch.resolve(self)?;
        if let Some(else_branch) = &stmt.else_branch {
            else_branch.resolve(self)?;
        }
        Ok(())
    }

    fn print(&mut self, stmt: &PrintStatement) -> ResolveResult {
        stmt.expression.resolve(self)
    }

    fn reteurn(&mut self, stmt: &ReturnStatement) -> ResolveResult {
        if let Some(value) = &stmt.value {
            value.resolve(self)?;
        }
        Ok(())
    }

    fn whyle(&mut self, stmt: &WhileStatement) -> ResolveResult {
        stmt.condition.resolve(self)?;
        stmt.body.resolve(self)?;
        Ok(())
    }

    fn foreach(&mut self, stmt: &ForeachStatement) -> ResolveResult {
        self.declare(&stmt.variable)?;
        self.define(&stmt.variable)?;
        stmt.collection.resolve(self)?;
        stmt.body.resolve(self)?;
        Ok(())
    }



    // EXPRESSIONS
    fn binary(&mut self, expr: &Binary) -> ResolveResult {
        expr.left.resolve(self)?;
        expr.right.resolve(self)?;
        Ok(())
    }

    fn call(&mut self, expr: &Call) -> ResolveResult {
        expr.callee.resolve(self)?;
        for arg in &expr.arguments {
            arg.resolve(self)?;
        }
        Ok(())
    }

    fn grouping(&mut self, expr: &Grouping) -> ResolveResult {
        expr.expression.resolve(self)
    }

    fn literal(&mut self, _: &Literal) -> ResolveResult {
        Ok(())
    }

    fn logical(&mut self, expr: &Logical) -> ResolveResult {
        expr.left.resolve(self)?;
        expr.right.resolve(self)?;
        Ok(())
    }

    fn unary(&mut self, expr: &Unary) -> ResolveResult {
        expr.right.resolve(self)
    }

    fn list(&mut self, expr: &List) -> ResolveResult {
        for element in &expr.elements {
            element.resolve(self)?;
        }
        Ok(())
    }
}


impl Resolvable for FunctionStatement {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.function(self)
    }
}

impl Resolvable for ExpressionStatement {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.expression_stmt(self)
    }
}


impl Resolvable for VarStatement {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.var(self)
    }
}

impl Resolvable for BlockStatement {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.block(self)
    }
}

impl Resolvable for ReturnStatement {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.reteurn(self)
    }
}

impl Resolvable for ForeachStatement {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.foreach(self)
    }
}

impl Resolvable for WhileStatement{
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.whyle(self)
    }
}

impl Resolvable for PrintStatement {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.print(self)
    }
}

impl Resolvable for IfStatement {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.iff(self)
    }
}

// Expressions
impl Resolvable for Binary {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.binary(self) 
    }
}
impl Resolvable for Logical {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.logical(self) 
    }
}
impl Resolvable for Call {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.call(self) 
    }
}
impl Resolvable for Grouping {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.grouping(self) 
    }
}
impl Resolvable for Literal {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.literal(self) 
    }
}

impl Resolvable for Variable {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.var_expr(self)
    }
}

impl Resolvable for List {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.list(self)
    }
}

impl Resolvable for Unary {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.unary(self)
    }
}
