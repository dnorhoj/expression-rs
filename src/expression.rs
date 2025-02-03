use crate::{schema::Value, serialize::Serialize};

#[derive(Clone, Debug)]
pub enum Expression {
    And(And),
    Or(Or),
    Not(Not),
    Operation(Operation),
}

impl Expression {
    pub fn serialize(&self) -> String {
        Serialize::fmt(self)
    }
}

#[derive(Clone, Debug)]
pub struct And(Vec<Expression>);

impl And {
    pub fn new(subexpressions: Vec<Expression>) -> Self {
        Self(subexpressions)
    }

    pub fn get_subexpressions(&self) -> &Vec<Expression> {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct Or(Vec<Expression>);

impl Or {
    pub fn new(subexpressions: Vec<Expression>) -> Self {
        Self(subexpressions)
    }

    pub fn get_subexpressions(&self) -> &Vec<Expression> {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct Not(Box<Expression>);

impl Not {
    pub fn new(subexpression: Expression) -> Self {
        Self(Box::new(subexpression))
    }

    pub fn get_subexpression(&self) -> &Expression {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct Operation {
    pub lhs: Literal,
    pub op: Operator,
    pub rhs: Literal,
}

impl Operation {
    pub fn new(lhs: Literal, op: Operator, rhs: Literal) -> Self {
        Self { lhs, op, rhs }
    }
}

#[derive(Clone, Debug)]
pub enum Literal {
    LiteralValue(Value),
    LiteralField(String),
}

#[derive(Clone, Debug)]
pub enum Operator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
}

impl Operator {
    pub fn fmt_static(&self) -> &'static str {
        match self {
            Operator::Eq => "==",
            Operator::Ne => "!=",
            Operator::Gt => ">",
            Operator::Gte => ">=",
            Operator::Lt => "<",
            Operator::Lte => "<=",
            Operator::In => "IN",
        }
    }
}
