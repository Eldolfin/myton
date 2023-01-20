use super::statement::*;
use super::expression::*;
use super::token::Token;
use super::traceback::Traceback;
use std::collections::HashMap;


type ResolveResult = Result<(), Traceback>;
pub type UUID = usize;

pub struct Resolver {
    scopes :Vec<HashMap<String, bool>>,
    pub locals :HashMap<UUID, usize>, // UUID -> depth
    current_function :FunctionType,
    current_class :ClassType,
}

#[derive(Clone, Copy)]
enum FunctionType {
    None,
    Function,
    Method,
}

#[derive(Clone, Copy)]
enum ClassType {
    None,
    Class,
    Subclass,
}

pub trait Resolvable {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult;
}



impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            scopes: vec![HashMap::new()],
            locals: HashMap::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }
    // STATEMENTS

    fn block(&mut self, block: &BlockStatement) -> ResolveResult {
        // self.begin_scope();
        self.stmts(&block.statements)?;
        // self.end_scope();
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
        // if let Some(scope) = self.scopes.last() {
        //     if let Some(false) = scope.get(&expr.token.value) {
        //         return Err(Traceback::from(format!("Cannot read local variable in its own initializer.")));
        //     }
        // }

        let casted :EXPR = Box::new(expr.clone());

        self.local(&casted, &expr.name.clone());
        Ok(())
    }

    fn local(&mut self, expr: &EXPR, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.value) {
                self.locals.insert(expr.uuid(), i);
                return;
            }
        }
    }

    fn function(&mut self, function: &FunctionStatement) -> ResolveResult {
        self.declare(&function.inner.borrow().name)?;
        self.define(&function.inner.borrow().name)?;

        self.resolve_function(function, FunctionType::Function)
    }

    fn resolve_function(&mut self, function: &FunctionStatement, tipe: FunctionType) -> ResolveResult {
        let enclosing_function : FunctionType = self.current_function.clone();
        self.current_function = tipe;

        self.begin_scope();
        for param in &function.inner.borrow().parameters {
            self.declare(param)?;
            self.define(param)?;
        }
        self.stmt(&function.inner.borrow().body)?;
        self.end_scope();

        self.current_function = enclosing_function;

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
        if matches!(self.current_function, FunctionType::None) {
            return Err(Traceback::from(format!("'return' outside function")));
        }

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

    fn global(&mut self, _: &GlobalStatement) -> ResolveResult {
        Ok(())
    }

    fn nonlocal(&mut self, _: &NonlocalStatement) -> ResolveResult {
        Ok(())
    }
    
    fn class(&mut self, class: &ClassStatement) -> ResolveResult {
        let enclosing_class  = self.current_class;
        self.current_class = ClassType::Class;

        self.declare(&class.name)?;
        self.define(&class.name)?;

        if let Some(superclass) = &class.superclass {
            if superclass.name.value == class.name.value {
                return Err(Traceback {
                    message: Some(format!("A class cannot inherit from itself.")),
                    pos: class.name.pos.unwrap(),
                    ..Default::default()
                });
            }

            self.current_class = ClassType::Subclass;
            superclass.resolve(self)?;

            self.begin_scope();
            self.scopes.last_mut().unwrap().insert("super".to_string(), true);
        }

        self.begin_scope();
        self.scopes.last_mut().unwrap().insert("this".to_string(), true);

        for method in &class.methods {
            let declaration = FunctionType::Method;
            self.resolve_function(method, declaration)?;
        }

        self.end_scope();

        if class.superclass.is_some() {
            self.end_scope();
        }

        self.current_class = enclosing_class;

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

    fn get(&mut self, expr: &Get) -> ResolveResult {
        expr.object.resolve(self)
    }

    fn set(&mut self, expr: &Set) -> ResolveResult {
        expr.object.resolve(self)?;
        expr.value.resolve(self)
    }

    fn this(&mut self, expr: &This) -> ResolveResult {
        if matches!(self.current_class, ClassType::None) {
            return Err(Traceback::from(format!("Cannot use 'this' outside of a class.")));
        }

        let casted :EXPR = Box::new(expr.clone());
        let mut keyword = expr.keyword.clone();

        // because we allow multiple types of this keywords
        // (this and self), we set the keyword to "this" so
        // that the local function can find it.
        keyword.value = "this".to_string();

        self.local(&casted, &keyword);
        Ok(())
    }

    fn superr(&mut self, expr: &Super) -> ResolveResult {
        if matches!(self.current_class, ClassType::None) {
            return Err(Traceback {
                message: Some(format!("Cannot use 'super' outside of a class.")),
                pos: expr.keyword.pos.unwrap(),
                ..Default::default()
            });
        } else if !matches!(self.current_class, ClassType::Subclass) {
            return Err(Traceback {
                message: Some(format!("Cannot use 'super' in a class with no superclass.")),
                pos: expr.keyword.pos.unwrap(),
                ..Default::default()
            });
        }

        let casted :EXPR = Box::new(expr.clone());

        self.local(&casted, &expr.keyword);
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

impl Resolvable for GlobalStatement {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.global(self)
    }
}

impl Resolvable for NonlocalStatement {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.nonlocal(self)
    }
}

impl Resolvable for ClassStatement {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.class(self)
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

impl Resolvable for Get {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.get(self)
    }
}

impl Resolvable for Set {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.set(self)
    }
}

impl Resolvable for This {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.this(self)
    }
}

impl Resolvable for Super {
    fn resolve(&self, resolver: &mut Resolver) -> ResolveResult {
        resolver.superr(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::myton::{Interpreter, parser::Parser, lexer::Lexer};

    #[test]
    fn test_variable_resolving() {
        let code = 
"a=\"global\"
def f():
  def print_A():
    print(a)
  print_A()
  a=\"local\"
  print_A()
f()".to_string();

        let mut interpreter = Interpreter::new();
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens.clone(), interpreter.output.clone());
        let program = parser.parse().unwrap();

        for stmt in &program {
            stmt.resolve(&mut interpreter.resolver).unwrap();
        }

        let locals = interpreter.resolver.locals;

        let mut message = String::new();

        for (u,d) in &locals {
             message += &format!("{:?} {:?}\n", u , d).to_string();
        }
        
        // assert!(false, "{}", message);

        assert_eq!(locals[&19], 2);
        assert_eq!(locals[&22], 0);
        assert_eq!(locals[&30], 0);
        assert_eq!(locals[&34], 0);
    }
}
