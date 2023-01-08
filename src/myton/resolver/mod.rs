use super::ast::*;

struct Resolver {
    // ...
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
        }
    }

    // pub fn block(&mut self, block: &BlockStatement) {
    //     self.begin_scope();
    //     self.resolve_stmts(&block.statements);
    //     self.end_scope();
    // }
    // 
    // fn resolve_stmts(&mut self, stmts: &[StmtTrait]) {
    //     for stmt in stmts {
    //         self.resolve_stmt(stmt);
    //     }
    // }
    // 
    // fn resolve_stmt(&mut self, stmt: &StmtTrait) {
    //     match stmt {
    //         StmtTrait::Block(block) => self.block(block),
    //     }
    // }
}
