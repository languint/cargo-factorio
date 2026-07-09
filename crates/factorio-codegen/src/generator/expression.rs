use factorio_ir::expression::Expression;

use crate::LuaGenerator;

impl LuaGenerator {
    pub(crate) fn generate_expression(&self, expression: &Expression) -> String {
        self.generate_expression_prec(expression, 0)
    }

    pub(crate) fn generate_expression_prec(&self, expression: &Expression, min_prec: u8) -> String {
        match expression {
            Expression::BinaryOp { lhs, op, rhs } => {
                let prec = Self::operator_precedence(*op);
                let lhs_str = self.generate_expression_prec(lhs, prec);
                let rhs_str = self.generate_expression_prec(rhs, prec.saturating_add(1));
                let result = format!("{} {} {}", lhs_str, Self::generate_operator(*op), rhs_str);

                if prec < min_prec {
                    format!("({result})")
                } else {
                    result
                }
            }
            _ => self.generate_atom(expression),
        }
    }

    /// Generate the smallest level of code (an atom).
    pub(crate) fn generate_atom(&self, expression: &Expression) -> String {
        match expression {
            Expression::Literal(literal) => Self::generate_literal(literal),
            Expression::Identifier(name) => name.clone(),
            Expression::FieldAccess { base, field } => {
                let base = self.generate_expression(base);
                format!("{base}.{field}")
            }
            Expression::QualifiedPath { segments } => {
                if let Some((struct_name, table_path)) = &self.struct_table_context
                    && segments
                        .first()
                        .is_some_and(|segment| segment == struct_name)
                {
                    let suffix = segments
                        .get(1..)
                        .map_or_else(String::new, |rest| rest.join("."));
                    if suffix.is_empty() {
                        return table_path.clone();
                    }
                    return format!("{table_path}.{suffix}");
                }

                segments.join(".")
            }
            Expression::Call { func, args } => {
                let func = self.generate_expression(func);
                let args = args
                    .iter()
                    .map(|arg| self.generate_expression(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{func}({args})")
            }
            Expression::MethodCall {
                receiver,
                method,
                args,
            } => {
                let receiver = self.generate_expression(receiver);
                let args = args
                    .iter()
                    .map(|arg| self.generate_expression(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{receiver}:{method}({args})")
            }
            Expression::StructLiteral { fields } => {
                let fields = fields
                    .iter()
                    .map(|(name, value)| format!("{name} = {}", self.generate_expression(value)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let literal = format!("{{ {fields} }}");

                if let Some((_, table_path)) = &self.struct_table_context {
                    format!("setmetatable({literal}, {{ __index = {table_path} }})")
                } else {
                    literal
                }
            }
            Expression::FormatConcat { parts } => parts
                .iter()
                .map(|part| self.generate_expression(part))
                .collect::<Vec<_>>()
                .join(" .. "),
            Expression::BinaryOp { .. } => {
                unreachable!("binary operators are handled by generate_expression_prec")
            }
        }
    }
}
