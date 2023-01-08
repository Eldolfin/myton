use super::statement::*;

struct Resolver {
    // ...
}

trait ResolveTrait {
    fn resolve(&mut self, stmt: Resolvable);
}

enum Resolvable {
    Block(BlockStatement),
    Expression(ExpressionStatement),
    Function(FunctionStatement),
    If(IfStatement),
    Print(PrintStatement),
    Return(ReturnStatement),
    Var(VarStatement),
    While(WhileStatement),
}

impl ResolveTrait for Resolver {
    fn resolve(&mut self, stmt: Resolvable) -> Result<(), String> {
        match stmt {
            Resolvable::Block(stmt) => self.resolve_block(stmt)?,
            Resolvable::Expression(stmt) => self.resolve_expression(stmt),
            Resolvable::Function(stmt) => self.resolve_function(stmt),
            Resolvable::If(stmt) => self.resolve_if(stmt),
            Resolvable::Print(stmt) => self.resolve_print(stmt),
            Resolvable::Return(stmt) => self.resolve_return(stmt),
            Resolvable::Var(stmt) => self.resolve_var(stmt),
            Resolvable::While(stmt) => self.resolve_while(stmt),
        }
    }
}

impl Resolver {
    pub fn resolve_block(&mut self, block: &BlockStatement) {
        self.begin_scope();
        self.resolve_stmts(&block.statements);
        self.end_scope();
    }

    fn resolve_stmts(&mut self, stmts: &[StmtTrait]) {
        for stmt in stmts {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &StmtTrait) {
        match stmt {
            StmtTrait::Block(block) => self.block(block),
        }
    }
}
