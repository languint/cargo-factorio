use factorio_ir::{module::Module, scope::Scope, statement::Statement};

use crate::{LuaGenerator, LuaGeneratorResult};

impl LuaGenerator {
    /// Generate code for a given [`Statement`].
    pub(crate) fn generate_statement(
        &mut self,
        statement: &Statement,
        module: Option<&Module>,
        module_name: Option<&str>,
        scope: Scope,
    ) -> LuaGeneratorResult<()> {
        match statement {
            Statement::FunctionDecl(function) => {
                self.generate_function(function, module, scope, module_name)?;
            }
            Statement::StructDecl(struct_decl) => {
                self.generate_struct(struct_decl, module, scope, module_name)?;
            }
            Statement::VariableDecl {
                name,
                value,
                source_type,
                ..
            } => {
                let value = self.generate_expression(value);
                let type_comment = self.variable_type_comment(source_type.as_deref());
                let line = match (scope, module_name) {
                    (Scope::Public, Some(module_name)) => {
                        format!("{module_name}.{name}{type_comment} = {value}")
                    }
                    _ => format!("local {name}{type_comment} = {value}"),
                };
                self.write_line(&line);
            }
            Statement::Assignment { target, value } => {
                let target = self.generate_expression(target);
                let value = self.generate_expression(value);
                self.write_line(&format!("{target} = {value}"));
            }
            Statement::Conditional {
                condition,
                then_block,
                else_block,
            } => {
                let condition = self.generate_expression(condition);
                self.write_line(&format!("if {condition} then"));

                self.indent_level += 1;
                for statement in then_block {
                    self.generate_statement(statement, module, module_name, Scope::Private)?;
                }
                self.indent_level -= 1;

                if !else_block.is_empty() {
                    self.write_line("else");
                    self.indent_level += 1;
                    for statement in else_block {
                        self.generate_statement(statement, module, module_name, Scope::Private)?;
                    }
                    self.indent_level -= 1;
                }
                self.write_line("end");
            }
            Statement::Return(value) => {
                let line = value.as_ref().map_or_else(
                    || "return".to_string(),
                    |value| format!("return {}", self.generate_expression(value)),
                );
                self.write_line(&line);
            }
            Statement::Expr(expression) => {
                self.write_line(&self.generate_expression(expression));
            }
        }

        Ok(())
    }
}
