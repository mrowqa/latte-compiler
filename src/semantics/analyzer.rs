use super::global_context::GlobalContext;
use model::ast::*;
use frontend_error::FrontendResult;

pub struct SemanticAnalyzer<'a> {
    ast: &'a Program,
    ctx: Option<GlobalContext<'a>>,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(prog: &'a Program) -> Self {
        SemanticAnalyzer {
            ast: prog,
            ctx: None,
        }
    }

    pub fn perform_full_analysis(&mut self) -> FrontendResult<()> {
        let _ = self.calculate_global_context()?;
        // todo...
        Ok(())
    }

    fn calculate_global_context(&mut self) -> FrontendResult<()> {
        if let Some(_) = self.ctx {
            return Ok(());
        }

        match GlobalContext::from(self.ast) {
            Ok(ctx) => {
                self.ctx = Some(ctx);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
