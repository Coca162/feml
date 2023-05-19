use std::{collections::BTreeMap, fs, io::ErrorKind, path};
mod parser;

/// Represents all valid TOML types
#[derive(Debug)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Table(Table),
    Array(Vec<Value>),
}

impl TryFrom<Value> for Table {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Table(data) => Ok(data),
            _ => Err(()),
        }
    }
}

/// A set of TOML key-value pairs
#[derive(Default, Debug)]
pub struct Table {
    data: BTreeMap<String, Value>,
}
impl Table {
    pub fn insert(&mut self, key: String, value: Value) {
        self.data.entry(key).or_insert(value);
    }
    pub fn get_or_insert(&mut self, key: String, value: Value) -> &mut Value {
        self.data.entry(key).or_insert(value)
    }
    pub fn get(&self, item: String) -> Option<&Value> {
        self.data.get(&item)
    }
    pub fn get_mut(&mut self, item: String) -> Option<&mut Value> {
        self.data.get_mut(&item)
    }
}

#[derive(Debug)]
pub enum Error {
    FileNotFound,
    FileReadError,
}

// pub fn parse_file<P: AsRef<path::Path>>(name: P) -> Result<Table, Error> {
//     match fs::read_to_string(name) {
//         Ok(file) => Ok(parser::parse_string(file)),
//         Err(e) => match e.kind() {
//             ErrorKind::NotFound => Err(Error::FileNotFound),
//             _ => Err(Error::FileReadError),
//         },
//     }
// }
