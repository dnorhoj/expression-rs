use chrono::{DateTime, Utc};
use expression::{AutoSchema, Parser};

#[derive(AutoSchema)]
struct StreetInfo {
    street_name: String,
    house_number: f64,
}

#[derive(AutoSchema)]
struct Address {
    street_info: Option<StreetInfo>,
}

#[derive(AutoSchema)]
struct Person {
    birthday: DateTime<Utc>,
    name: String,
    gamertag: String,
    ost: Option<Vec<f64>>,
    address: Address,
}

fn get_person() -> Person {
    Person {
        birthday: DateTime::parse_from_rfc3339("1999-07-13T00:00:00Z")
            .unwrap()
            .into(),
        name: "John Smith".to_string(),
        gamertag: "jsmith99".to_string(),
        ost: None,
        address: Address {
            street_info: Some(StreetInfo {
                street_name: String::from("Funny Rd."),
                house_number: 15.,
            }),
        },
    }
}

fn main() {
    // Define schema - defines which fields exist on the target, as well as how to extract them
    let engine = Person::get_engine();

    //let expression = r#"birthday in [1990-01-01T00:00:00Z, 2000-01-01T00:00:00Z]"#;
    //let expression = r#"address:street_name == 1"#;
    let expression = r#"address:street_info:street_name == "Funny Rd.""#;

    let parsed = match Parser::parse(expression) {
        Ok(parsed) => parsed,
        Err(e) => panic!("Parsing failed: {}", e.to_string()),
    };

    println!("Parsed: {:?}", parsed);

    match engine.validate(&parsed) {
        Ok(_) => println!("Validation passed!"),
        Err(e) => println!("Validation failed: {}", e),
    }

    println!("Serialization: {}", parsed.serialize());

    match engine.execute(&parsed, &get_person()) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Execution failed: {}", e),
    };
}
