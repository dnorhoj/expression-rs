use chrono::{DateTime, Utc};
use expression::{AutoSchema, Parser};

#[derive(AutoSchema)]
struct Test {
    str: String,
    num: f64,
    boo: bool,
    raw: Vec<u8>,
    dat: DateTime<Utc>,
    str_list: Vec<String>,
    num_list: Vec<f64>,
    boo_list: Vec<bool>,
    raw_list: Vec<Vec<u8>>,
    dat_list: Vec<DateTime<Utc>>,
}

pub fn main() {
    let test = Test {
        str: String::from("Jeff"),
        num: 10.,
        boo: true,
        raw: b"asd".into(),
        dat: Utc::now(),
        str_list: vec![String::from("Jeff")],
        num_list: vec![10.],
        boo_list: vec![true],
        raw_list: vec![b"asd".into()],
        dat_list: vec![Utc::now()],
    };

    let expr = Parser::parse("(/[Jj].../ IN name AND jens > 0)").unwrap();

    let result = Test::get_engine().execute(&expr, &test).unwrap();

    println!("Result: {}", result);
}
