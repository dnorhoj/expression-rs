use chrono::{DateTime, Utc};
use pom::{Error, parser::*};
use thiserror::Error;

use core::str;
use std::str::FromStr;

use crate::{
    expression::{And, Expression, Literal, Not, Operation, Operator, Or},
    schema::Value,
};

// A bit reworked version of seq to allow ascii lower/upper to be treated as the same.
// Original source: https://github.com/J-F-Liu/pom/blob/0fd011c736ea77b06c6215da8f3fc7140087a719/src/parser.rs#L285-L309
pub fn seq_nocase<'a, 'b: 'a>(tag: &'b [u8]) -> Parser<'a, u8, &'a [u8]> {
    Parser::new(move |input: &'a [u8], start: usize| {
        let mut index = 0;
        loop {
            let pos = start + index;
            if index == tag.len() {
                return Ok((tag, pos));
            }
            let Some(s) = input.get(pos) else {
                return Err(Error::Incomplete);
            };
            if tag[index] != s.to_ascii_lowercase() && tag[index] != s.to_ascii_uppercase() {
                return Err(Error::Mismatch {
                    message: format!("seq {:?} expect: {:?}, found: {:?}", tag, tag[index], s),
                    position: pos,
                });
            }
            index += 1;
        }
    })
}

macro_rules! list_parser {
    ($fn_name:ident, $type_:ty, $value_fn:expr) => {
        fn $fn_name<'a>() -> Parser<'a, u8, Vec<$type_>> {
            ((sym(b'[') + space()) * ($value_fn() - space())
                + ((sym(b',') + space()) * $value_fn() - space()).repeat(0..)
                - sym(b']'))
            .map(|(first, mut values)| {
                values.insert(0, first);

                values
            })
        }
    };
}

fn space<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t\r\n").repeat(0..).discard().name("space")
}

fn number<'a>() -> Parser<'a, u8, f64> {
    let integer = one_of(b"123456789") - one_of(b"0123456789").repeat(0..) | sym(b'0');
    let frac = sym(b'.') + one_of(b"0123456789").repeat(1..);
    let exp = one_of(b"eE") + one_of(b"+-").opt() + one_of(b"0123456789").repeat(1..);
    let number = sym(b'-').opt() + integer + frac.opt() + exp.opt();
    number
        .collect()
        .convert(str::from_utf8)
        .convert(|s| f64::from_str(&s))
        .name("number")
}

list_parser!(number_list, f64, number);

fn raw<'a>() -> Parser<'a, u8, Vec<u8>> {
    let parser = (sym(b'|') - space())
        * (one_of(b"0123456789abcdefABCDEF") + one_of(b"0123456789abcdefABCDEF") - space())
            .map(|(a, b)| u8::from_str_radix(str::from_utf8(&[a, b]).unwrap(), 16).unwrap())
            .repeat(1..)
        - (sym(b'|') - space());

    parser.name("raw")
}

list_parser!(raw_list, Vec<u8>, raw);

fn string<'a>() -> Parser<'a, u8, String> {
    let special_char = sym(b'\\')
        | sym(b'/')
        | sym(b'"')
        | sym(b'b').map(|_| b'\x08')
        | sym(b'f').map(|_| b'\x0C')
        | sym(b'n').map(|_| b'\n')
        | sym(b'r').map(|_| b'\r')
        | sym(b't').map(|_| b'\t');
    let escape_sequence = sym(b'\\') * special_char;
    let string = sym(b'"') * (none_of(b"\\\"") | escape_sequence).repeat(0..) - sym(b'"');
    string.convert(String::from_utf8).name("string")
}

list_parser!(string_list, String, string);

fn regex_string<'a>() -> Parser<'a, u8, String> {
    let string = sym(b'/') * (seq(b"\\/").map(|_| b'/') | none_of(b"/")).repeat(0..) - sym(b'/');
    string.convert(String::from_utf8).name("regex_string")
}

fn datetime<'a>() -> Parser<'a, u8, DateTime<Utc>> {
    let num = || one_of(b"1234567890");

    let parser = num().repeat(4)
        + sym(b'-')
        + num().repeat(2)
        + sym(b'-')
        + num().repeat(2)
        + sym(b'T')
        + num().repeat(2)
        + sym(b':')
        + num().repeat(2)
        + sym(b':')
        + num().repeat(2)
        + (sym(b'.') + num().repeat(1..6)).opt()
        + (sym(b'Z').collect()
            | (one_of(b"+-") + num().repeat(2) + sym(b':') + num().repeat(2)).collect());

    parser
        .collect()
        .convert(str::from_utf8)
        .convert(|s| DateTime::parse_from_rfc3339(s).map(|date| date.to_utc()))
}

list_parser!(datetime_list, DateTime<Utc>, datetime);

fn field<'a>() -> Parser<'a, u8, String> {
    let parser = (one_of(b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_")
        + one_of(b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_:0123456789").repeat(0..))
    .collect()
    .convert(str::from_utf8)
    .map(|s| String::from(s));

    parser.name("field")
}

fn operator<'a>() -> Parser<'a, u8, Operator> {
    let parser = seq(b"==").map(|_| Operator::Eq)
        | seq(b"!=").map(|_| Operator::Ne)
        | seq(b">=").map(|_| Operator::Gte)
        | seq(b"<=").map(|_| Operator::Lte)
        | seq(b">").map(|_| Operator::Gt)
        | seq(b"<").map(|_| Operator::Lt)
        | seq_nocase(b"in").map(|_| Operator::In);

    parser.name("operator")
}

fn literal<'a>() -> Parser<'a, u8, Literal> {
    let parser = seq_nocase(b"null").map(|_| Literal::LiteralValue(Value::Null))
        | seq_nocase(b"true").map(|_| Literal::LiteralValue(Value::Boolean(true)))
        | seq_nocase(b"false").map(|_| Literal::LiteralValue(Value::Boolean(false)))
        | string().map(|str| Literal::LiteralValue(Value::String(str)))
        | regex_string().map(|pattern| Literal::LiteralValue(Value::Regex(pattern)))
        | raw().map(|bytes| Literal::LiteralValue(Value::Raw(bytes)))
        | datetime().map(|datetime| Literal::LiteralValue(Value::DateTime(datetime)))
        | number().map(|num| Literal::LiteralValue(Value::Number(num)))
        | string_list().map(|str| Literal::LiteralValue(Value::StringList(str)))
        | raw_list().map(|bytes| Literal::LiteralValue(Value::RawList(bytes)))
        | datetime_list().map(|datetime| Literal::LiteralValue(Value::DateTimeList(datetime)))
        | number_list().map(|num| Literal::LiteralValue(Value::NumberList(num)))
        | field().map(|field| Literal::LiteralField(field));

    parser.name("literal")
}

fn operation<'a>() -> Parser<'a, u8, Operation> {
    let parser = ((literal() - space()) + (operator() - space()) + literal())
        .map(|((lhs, op), rhs)| Operation::new(lhs, op, rhs));

    parser.name("operation")
}

fn and<'a>() -> Parser<'a, u8, And> {
    let parser = ((sym(b'(') - space())
        * ((call(expression) - space() - seq_nocase(b"and") - space())
            + (call(expression) - space() - (seq_nocase(b"and") - space()).opt()).repeat(1..))
        - (space() + sym(b')')))
    .map(|(first, mut operations)| {
        operations.insert(0, first);

        And::new(operations)
    });

    parser.name("and")
}

fn or<'a>() -> Parser<'a, u8, Or> {
    let parser = ((sym(b'(') - space())
        * ((call(expression) - space() - seq_nocase(b"or") - space())
            + (call(expression) - space() - (seq_nocase(b"or") - space()).opt()).repeat(1..))
        - (space() + sym(b')')))
    .map(|(first, mut operations)| {
        operations.insert(0, first);

        Or::new(operations)
    });

    parser.name("or")
}

fn not<'a>() -> Parser<'a, u8, Not> {
    let parser = ((sym(b'!') + space() + sym(b'(') + space()) * call(expression)
        - (space() + sym(b')')))
    .map(|ex| Not::new(ex));

    parser.name("not")
}

fn expression<'a>() -> Parser<'a, u8, Expression> {
    let expression = and().map(|and| Expression::And(and))
        | or().map(|or| Expression::Or(or))
        | not().map(|not| Expression::Not(not))
        | operation().map(|op| Expression::Operation(op));

    expression.name("expression")
}

fn parser<'a>() -> Parser<'a, u8, Expression> {
    space() * expression() - end()
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    ParsingError(#[from] pom::Error),
}

pub struct ExpressionParser;

impl ExpressionParser {
    pub fn parse(input: &str) -> Result<Expression, ParseError> {
        let expression = parser().parse(input.as_bytes())?;

        Ok(expression)
    }
}
