use std::{collections::HashMap, rc::Rc};

use chrono::{DateTime, Utc};

#[derive(Clone, Copy, Debug)]
pub enum Type {
    String,
    Regex,
    Number,
    Boolean,
    Raw,
    DateTime,
    StringList,
    NumberList,
    BooleanList,
    RawList,
    DateTimeList,
    Null,
}

impl Type {
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn variant_name(&self) -> &'static str {
        match self {
            Type::String => "String",
            Type::Regex => "Regex",
            Type::Number => "Number",
            Type::Boolean => "Boolean",
            Type::Raw => "Raw",
            Type::DateTime => "DateTime",
            Type::StringList => "StringList",
            Type::NumberList => "NumberList",
            Type::BooleanList => "BooleanList",
            Type::RawList => "RawList",
            Type::DateTimeList => "DateTimeList",
            Type::Null => "Null",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Regex(String),
    Number(f64),
    Boolean(bool),
    Raw(Vec<u8>),
    DateTime(DateTime<Utc>),
    StringList(Vec<String>),
    NumberList(Vec<f64>),
    BooleanList(Vec<bool>),
    RawList(Vec<Vec<u8>>),
    DateTimeList(Vec<DateTime<Utc>>),
    Null,
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn get_type(&self) -> Type {
        match self {
            Value::String(_) => Type::String,
            Value::Regex(_) => Type::Regex,
            Value::Number(_) => Type::Number,
            Value::Boolean(_) => Type::Boolean,
            Value::Raw(_) => Type::Raw,
            Value::DateTime(_) => Type::DateTime,
            Value::StringList(_) => Type::StringList,
            Value::NumberList(_) => Type::NumberList,
            Value::BooleanList(_) => Type::BooleanList,
            Value::RawList(_) => Type::RawList,
            Value::DateTimeList(_) => Type::DateTimeList,
            Value::Null => Type::Null,
        }
    }

    pub fn get_type_name(&self) -> &'static str {
        self.get_type().variant_name()
    }
}

pub struct Field<T> {
    pub field_type: Type,
    pub field_extractor: Box<dyn Fn(&T) -> Value>,
}

impl<T> Field<T> {
    pub fn new(field_type: Type, field_extractor: Box<dyn Fn(&T) -> Value>) -> Self {
        Self {
            field_type,
            field_extractor,
        }
    }
}

pub struct SchemaBuilder<T> {
    fields: HashMap<&'static str, Rc<Field<T>>>,
}

macro_rules! field_extractor_builder {
    ($fn_name:ident, $type_:ty, $enum_name:ident) => {
        pub fn $fn_name(
            mut self,
            field_name: &'static str,
            extractor: impl Fn(&T) -> Option<$type_> + 'static,
        ) -> Self {
            let wrapped_extractor = Box::new(move |target: &T| {
                extractor(target).map_or_else(|| Value::Null, |val| Value::$enum_name(val))
            });

            self.fields.insert(
                field_name,
                Rc::new(Field::new(Type::$enum_name, wrapped_extractor)),
            );

            self
        }
    };
}

impl<T> SchemaBuilder<T> {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    field_extractor_builder!(with_string_field, String, String);
    field_extractor_builder!(with_number_field, f64, Number);
    field_extractor_builder!(with_boolean_field, bool, Boolean);
    field_extractor_builder!(with_raw_field, Vec<u8>, Raw);
    field_extractor_builder!(with_datetime_field, DateTime<Utc>, DateTime);
    field_extractor_builder!(with_string_list_field, Vec<String>, StringList);
    field_extractor_builder!(with_number_list_field, Vec<f64>, NumberList);
    field_extractor_builder!(with_boolean_list_field, Vec<bool>, BooleanList);
    field_extractor_builder!(with_raw_list_field, Vec<Vec<u8>>, RawList);
    field_extractor_builder!(with_datetime_list_field, Vec<DateTime<Utc>>, DateTimeList);

    pub fn build(self) -> Schema<T> {
        Schema {
            fields: self.fields,
        }
    }
}

pub struct Schema<T> {
    fields: HashMap<&'static str, Rc<Field<T>>>,
}

impl<T> Schema<T> {
    pub fn get_field(&self, field_name: &str) -> Option<Rc<Field<T>>> {
        self.fields.get(field_name).cloned()
    }
}
