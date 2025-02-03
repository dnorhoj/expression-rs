use chrono::{DateTime, Utc};
use expression::{Engine, Parser, SchemaBuilder};

struct Person {
    birthday: DateTime<Utc>,
    name: String,
    gamertag: String,
}

fn get_person() -> Person {
    Person {
        birthday: DateTime::parse_from_rfc3339("1999-07-13T00:00:00Z")
            .unwrap()
            .into(),
        name: "John Smith".to_string(),
        gamertag: "jsmith99".to_string(),
    }
}

fn main() {
    // Define schema - defines which fields exist on the target, as well as how to extract them
    let schema = SchemaBuilder::<Person>::new()
        .with_string_field("name", |p| Some(p.name.clone()))
        .with_string_field("gamertag", |p| Some(p.gamertag.clone()))
        .with_number_field("age", |p| {
            // This field (age) is calculated from the birthday field
            Some(Utc::now().years_since(p.birthday).unwrap_or(0) as f64)
        })
        .with_datetime_field("birthday", |p| Some(p.birthday))
        .build();

    let engine = Engine::new(schema);

    let expression = r#"(birthday in [1990-01-01T00:00:00Z, 2000-01-01T00:00:00Z] and age == 25)"#;

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
