use crate::{
    expression::{And, Expression, Literal, Not, Operation, Operator, Or},
    schema::Value,
};

pub trait Serialize {
    fn fmt(&self) -> String;
}

impl Serialize for Expression {
    fn fmt(&self) -> String {
        match self {
            Expression::And(and) => Serialize::fmt(and),
            Expression::Or(or) => Serialize::fmt(or),
            Expression::Not(not) => Serialize::fmt(not),
            Expression::Operation(operation) => Serialize::fmt(operation),
        }
    }
}

impl Serialize for And {
    fn fmt(&self) -> String {
        format!(
            "({})",
            self.get_subexpressions()
                .iter()
                .map(|e| Serialize::fmt(e))
                .collect::<Vec<String>>()
                .join(" AND ")
        )
    }
}

impl Serialize for Or {
    fn fmt(&self) -> String {
        format!(
            "({})",
            self.get_subexpressions()
                .iter()
                .map(|e| Serialize::fmt(e))
                .collect::<Vec<String>>()
                .join(" OR ")
        )
    }
}

impl Serialize for Not {
    fn fmt(&self) -> String {
        format!("!({})", Serialize::fmt(self.get_subexpression()))
    }
}

impl Serialize for Operation {
    fn fmt(&self) -> String {
        format!(
            "{} {} {}",
            Serialize::fmt(&self.lhs),
            Serialize::fmt(&self.op),
            Serialize::fmt(&self.rhs)
        )
    }
}

impl Serialize for Literal {
    fn fmt(&self) -> String {
        match self {
            Literal::LiteralValue(value) => Serialize::fmt(value),
            Literal::LiteralField(field_name) => field_name.to_string(),
        }
    }
}

fn format_regex(val: &String) -> String {
    format!("/{}/", val.replace("/", "\\/"))
}

fn format_raw(val: &Vec<u8>) -> String {
    format!(
        "|{}|",
        val.iter()
            .map(|byte| format!("{:02x?}", byte))
            .collect::<Vec<String>>()
            .join(" ")
    )
}

impl Serialize for Value {
    fn fmt(&self) -> String {
        match self {
            Value::String(val) => format!("{:?}", val),
            Value::Regex(val) => format_regex(val),
            Value::Number(val) => format!("{}", val),
            Value::Boolean(val) => format!("{}", val),
            Value::Raw(val) => format_raw(val),
            Value::DateTime(val) => val.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true),
            Value::StringList(items) => format!(
                "[{}]",
                items
                    .iter()
                    .map(|val| format!("{:?}", val))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::NumberList(items) => format!(
                "[{}]",
                items
                    .iter()
                    .map(|val| format!("{}", val))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::BooleanList(items) => format!(
                "[{}]",
                items
                    .iter()
                    .map(|val| format!("{}", val))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::RawList(items) => format!(
                "[{}]",
                items
                    .iter()
                    .map(format_raw)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::DateTimeList(items) => format!(
                "[{}]",
                items
                    .iter()
                    .map(|val| val.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::Null => String::from("null"),
        }
    }
}

impl Serialize for Operator {
    fn fmt(&self) -> String {
        self.fmt_static().to_string()
    }
}
