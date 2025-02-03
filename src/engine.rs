use std::fmt::{Debug, Display};

use regex::Regex;
use thiserror::Error;

use crate::{
    expression::{Expression, Literal, Operation, Operator},
    misc::is_sublist,
    schema::{Schema, Type, Value},
};

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("A field with the name '{0}' does not exist")]
    InvalidFieldError(String),
    #[error("Cannot check if {0}")]
    InvalidOperatorError(InvalidOperatorError),
}

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("A field with the name '{0}' does not exist")]
    InvalidFieldError(String),
    #[error("Cannot check if {0}")]
    InvalidOperatorError(InvalidOperatorError),
    #[error("Invalid date range")]
    InvalidDateRangeError,
}

pub struct InvalidOperatorError(Type, Operator, Type);

impl Debug for InvalidOperatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            &self.0.variant_name(),
            &self.1.fmt_static(),
            &self.2.variant_name()
        )
    }
}

impl Display for InvalidOperatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

pub struct Engine<T> {
    schema: Schema<T>,
}

impl<T> Engine<T> {
    pub fn new(schema: Schema<T>) -> Self {
        Self { schema }
    }

    pub fn validate(&self, expression: &Expression) -> Result<(), ValidationError> {
        match expression {
            Expression::And(and) => and
                .get_subexpressions()
                .iter()
                .try_for_each(|i| self.validate(i)),
            Expression::Or(or) => or
                .get_subexpressions()
                .iter()
                .try_for_each(|i| self.validate(i)),
            Expression::Not(not) => self.validate(not.get_subexpression()),
            Expression::Operation(operation) => self.validate_operation(operation),
        }
    }

    fn validate_operation(&self, operation: &Operation) -> Result<(), ValidationError> {
        let lhs = self.extract_literal_type(&operation.lhs)?;
        let rhs = self.extract_literal_type(&operation.rhs)?;

        let operator_error = || {
            ValidationError::InvalidOperatorError(InvalidOperatorError(
                lhs,
                operation.op.clone(),
                rhs,
            ))
        };

        if rhs.is_null() || lhs.is_null() {
            return match operation.op {
                Operator::Eq | Operator::Ne | Operator::In => Ok(()),
                _ => Err(operator_error()),
            };
        }

        match lhs {
            Type::String => match rhs {
                Type::String => match operation.op {
                    Operator::Eq | Operator::Ne | Operator::In => Ok(()),
                    // Invalid operation
                    _ => Err(operator_error()),
                },
                Type::StringList => match operation.op {
                    Operator::In => Ok(()),
                    _ => Err(operator_error()),
                },
                _ => Err(operator_error()),
            },
            Type::Regex => match rhs {
                Type::String => match operation.op {
                    Operator::In => Ok(()),
                    _ => Err(operator_error()),
                },
                Type::StringList => match operation.op {
                    Operator::In => Ok(()),
                    _ => Err(operator_error()),
                },
                _ => Err(operator_error()),
            },
            Type::Number => match rhs {
                Type::Number => match operation.op {
                    Operator::Eq
                    | Operator::Ne
                    | Operator::Gt
                    | Operator::Gte
                    | Operator::Lt
                    | Operator::Lte => Ok(()),
                    _ => Err(operator_error()),
                },
                Type::NumberList => match operation.op {
                    Operator::In => Ok(()),
                    _ => Err(operator_error()),
                },
                _ => Err(operator_error()),
            },
            Type::Boolean => match rhs {
                Type::Boolean => match operation.op {
                    Operator::Eq | Operator::Ne => Ok(()),
                    _ => Err(operator_error()),
                },
                Type::BooleanList => match operation.op {
                    Operator::In => Ok(()),
                    _ => Err(operator_error()),
                },
                _ => Err(operator_error()),
            },
            Type::Raw => match rhs {
                Type::Raw => match operation.op {
                    Operator::Eq | Operator::Ne | Operator::In => Ok(()),
                    _ => Err(operator_error()),
                },
                Type::RawList => match operation.op {
                    Operator::In => Ok(()),
                    _ => Err(operator_error()),
                },
                _ => Err(operator_error()),
            },
            Type::DateTime => match rhs {
                Type::DateTime => match operation.op {
                    Operator::Eq
                    | Operator::Ne
                    | Operator::Gt
                    | Operator::Gte
                    | Operator::Lt
                    | Operator::Lte => Ok(()),
                    _ => Err(operator_error()),
                },
                Type::DateTimeList => match operation.op {
                    Operator::In => Ok(()),
                    _ => Err(operator_error()),
                },
                _ => Err(operator_error()),
            },
            Type::StringList => match rhs {
                Type::StringList => match operation.op {
                    Operator::Eq | Operator::Ne => Ok(()),
                    _ => Err(operator_error()),
                },
                _ => Err(operator_error()),
            },
            Type::NumberList => match rhs {
                Type::NumberList => match operation.op {
                    Operator::Eq | Operator::Ne => Ok(()),
                    _ => Err(operator_error()),
                },
                _ => Err(operator_error()),
            },
            Type::BooleanList => match rhs {
                Type::BooleanList => match operation.op {
                    Operator::Eq | Operator::Ne => Ok(()),
                    _ => Err(operator_error()),
                },
                _ => Err(operator_error()),
            },
            Type::RawList => match rhs {
                Type::RawList => match operation.op {
                    Operator::Eq | Operator::Ne => Ok(()),
                    _ => Err(operator_error()),
                },
                _ => Err(operator_error()),
            },
            Type::DateTimeList => match rhs {
                Type::DateTimeList => match operation.op {
                    Operator::Eq | Operator::Ne => Ok(()),
                    _ => Err(operator_error()),
                },
                _ => Err(operator_error()),
            },
            Type::Null => Ok(()),
        }
    }

    pub fn execute(&self, expression: &Expression, target: &T) -> Result<bool, ExecutionError> {
        match expression {
            Expression::And(and) => {
                for i in and.get_subexpressions() {
                    if !self.execute(i, target)? {
                        return Ok(false);
                    }
                }

                return Ok(true);
            }
            Expression::Or(or) => {
                for i in or.get_subexpressions() {
                    if self.execute(i, target)? {
                        return Ok(true);
                    }
                }

                return Ok(false);
            }
            Expression::Not(not) => self
                .execute(not.get_subexpression(), target)
                .map(|result| !result),
            Expression::Operation(operation) => self.execute_operation(operation, target),
        }
    }

    fn execute_operation(&self, operation: &Operation, target: &T) -> Result<bool, ExecutionError> {
        let lhs = self.extract_literal(&operation.lhs, target)?;
        let rhs = self.extract_literal(&operation.rhs, target)?;

        let operator_error = || {
            ExecutionError::InvalidOperatorError(InvalidOperatorError(
                lhs.get_type(),
                operation.op.clone(),
                rhs.get_type(),
            ))
        };

        if lhs.is_null() {
            if rhs.is_null() {
                return Ok(match operation.op {
                    Operator::Eq => true,
                    _ => false,
                });
            } else {
                return Ok(match operation.op {
                    Operator::Ne => true,
                    _ => false,
                });
            }
        } else if rhs.is_null() {
            return Ok(match operation.op {
                Operator::Ne => true,
                _ => false,
            });
        }

        Ok(match &lhs {
            Value::String(lhv) => match &rhs {
                Value::String(rhv) => match operation.op {
                    Operator::Eq => lhv == rhv,
                    Operator::Ne => lhv != rhv,
                    Operator::In => rhv.contains(lhv),
                    _ => return Err(operator_error()),
                },
                Value::StringList(rhv) => match operation.op {
                    Operator::In => rhv.contains(&lhv),
                    _ => return Err(operator_error()),
                },
                _ => return Err(operator_error()),
            },
            Value::Regex(lhv) => match &rhs {
                Value::String(rhv) => match operation.op {
                    Operator::In => {
                        let regex = Regex::new(lhv).unwrap();

                        regex.is_match(&rhv)
                    }
                    _ => return Err(operator_error()),
                },
                Value::StringList(rhv) => match operation.op {
                    Operator::In => {
                        let regex = Regex::new(lhv).unwrap();

                        rhv.iter().any(|v| regex.is_match(v))
                    }
                    _ => return Err(operator_error()),
                },
                _ => return Err(operator_error()),
            },
            Value::Number(lhv) => match &rhs {
                Value::Number(rhv) => match operation.op {
                    Operator::Eq => lhv == rhv,
                    Operator::Ne => lhv != rhv,
                    Operator::Gt => lhv > rhv,
                    Operator::Gte => lhv >= rhv,
                    Operator::Lt => lhv < rhv,
                    Operator::Lte => lhv <= rhv,
                    _ => return Err(operator_error()),
                },
                Value::NumberList(rhv) => match operation.op {
                    Operator::In => rhv.contains(lhv),
                    _ => return Err(operator_error()),
                },
                _ => return Err(operator_error()),
            },
            Value::Boolean(lhv) => match &rhs {
                Value::Boolean(rhv) => match operation.op {
                    Operator::Eq => lhv == rhv,
                    Operator::Ne => lhv != rhv,
                    _ => return Err(operator_error()),
                },
                Value::BooleanList(rhv) => match operation.op {
                    Operator::In => rhv.contains(lhv),
                    _ => return Err(operator_error()),
                },
                _ => return Err(operator_error()),
            },
            Value::Raw(lhv) => match &rhs {
                Value::Raw(rhv) => match operation.op {
                    Operator::Eq => lhv == rhv,
                    Operator::Ne => lhv != rhv,
                    Operator::In => is_sublist(&rhv, &lhv),
                    _ => return Err(operator_error()),
                },
                Value::RawList(rhv) => match operation.op {
                    Operator::In => rhv.iter().any(|v| lhv == v),
                    _ => return Err(operator_error()),
                },
                _ => return Err(operator_error()),
            },
            Value::DateTime(lhv) => match &rhs {
                Value::DateTime(rhv) => match operation.op {
                    Operator::Eq => lhv == rhv,
                    Operator::Ne => lhv != rhv,
                    Operator::Gt => lhv > rhv,
                    Operator::Gte => lhv >= rhv,
                    Operator::Lt => lhv < rhv,
                    Operator::Lte => lhv <= rhv,
                    _ => return Err(operator_error()),
                },
                Value::DateTimeList(rhv) => match operation.op {
                    Operator::In => {
                        if rhv.len() != 2 {
                            return Err(ExecutionError::InvalidDateRangeError);
                        }

                        let from = rhv.get(0).unwrap();
                        let until = rhv.get(1).unwrap();

                        lhv >= from && lhv < until
                    }
                    _ => return Err(operator_error()),
                },
                _ => return Err(operator_error()),
            },
            Value::StringList(lhv) => match &rhs {
                Value::StringList(rhv) => match operation.op {
                    Operator::Eq => lhv == rhv,
                    Operator::Ne => lhv != rhv,
                    _ => return Err(operator_error()),
                },
                _ => return Err(operator_error()),
            },
            Value::NumberList(lhv) => match &rhs {
                Value::NumberList(rhv) => match operation.op {
                    Operator::Eq => lhv == rhv,
                    Operator::Ne => lhv != rhv,
                    _ => return Err(operator_error()),
                },
                _ => return Err(operator_error()),
            },
            Value::BooleanList(lhv) => match &rhs {
                Value::BooleanList(rhv) => match operation.op {
                    Operator::Eq => lhv == rhv,
                    Operator::Ne => lhv != rhv,
                    _ => return Err(operator_error()),
                },
                _ => return Err(operator_error()),
            },
            Value::RawList(lhv) => match &rhs {
                Value::RawList(rhv) => match operation.op {
                    Operator::Eq => lhv == rhv,
                    Operator::Ne => lhv != rhv,
                    _ => return Err(operator_error()),
                },
                _ => return Err(operator_error()),
            },
            Value::DateTimeList(lhv) => match &rhs {
                Value::DateTimeList(rhv) => match operation.op {
                    Operator::Eq => lhv == rhv,
                    Operator::Ne => lhv != rhv,
                    _ => return Err(operator_error()),
                },
                _ => return Err(operator_error()),
            },
            Value::Null => unreachable!(),
        })
    }

    fn extract_literal_type(&self, literal: &Literal) -> Result<Type, ValidationError> {
        Ok(match &literal {
            Literal::LiteralValue(value) => value.get_type(),
            Literal::LiteralField(field_name) => {
                self.schema
                    .get_field(field_name)
                    .ok_or_else(|| ValidationError::InvalidFieldError(field_name.to_string()))?
                    .field_type
            }
        })
    }

    fn extract_literal(&self, literal: &Literal, target: &T) -> Result<Value, ExecutionError> {
        Ok(match &literal {
            Literal::LiteralValue(value) => value.clone(),
            Literal::LiteralField(field_name) => {
                let field_extractor = &self
                    .schema
                    .get_field(field_name)
                    .ok_or_else(|| ExecutionError::InvalidFieldError(field_name.to_string()))?
                    .field_extractor;

                (*field_extractor)(target)
            }
        })
    }
}
