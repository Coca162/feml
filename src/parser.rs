use core::str::Split;

use crate::{Table, Value};

struct Context<'a> {
    pub root: Table,
    pub current_table: Option<&'a Table>,
    pub last_token: Option<Token>,
    pub line: u32,
    buffer: Vec<u8>,
}
impl<'a> Context<'a> {
    pub fn new() -> Self {
        Self {
            root: Table::default(),
            buffer: Vec::<u8>::new(),
            current_table: None,
            last_token: None,
            line: 0,
        }
    }

    fn parse(&mut self) {
        let mut chars = self.buffer.iter();
        if let Some(letter) = chars.next() {
            match letter {
                b'[' => {
                    if let Some(letter) = chars.next() {
                        // Array table
                        if letter == &b'[' {
                        } else {
                            // Table
                            let mut name_buffer = Vec::<u8>::new();
                            name_buffer.push(*letter);

                            for letter in chars {
                                if letter == &b']' {
                                    if let Ok(name) = std::str::from_utf8(&name_buffer) {
                                        let mut hierarchy = name.split('.');
                                        let initial_table = self.root.get_or_insert(
                                            hierarchy.next().unwrap().to_string(),
                                            Value::Table(Table::default()),
                                        );
                                        if let Value::Table(table) = hierarchy.fold(
                                            initial_table,
                                            |last_table, table_name| {
                                                if let Value::Table(last_table) = last_table {
                                                    last_table.get_or_insert(
                                                        table_name.to_string(),
                                                        Value::Table(Table::default()),
                                                    )
                                                } else {
                                                    panic!(
                                                        "Parent for `{table_name}` is not a Table!"
                                                    );
                                                }
                                            },
                                        ) {
                                            self.current_table = Some(table);
                                        } else {
                                            panic!("Failed to create table `{name}` - it existed as another value!")
                                        }
                                        break;
                                    } else {
                                        panic!("Failed to parse table name as UTF-8.")
                                    }
                                } else {
                                    name_buffer.push(*letter);
                                }
                            }
                        }
                    }
                }
                b'"' => {}
                _ => {}
            }
        }
        self.buffer.clear();
    }

    pub fn newline(&'a mut self) {
        self.line += 1;
        self.parse();
    }

    pub fn buffer(&mut self, letter: u8) {
        self.buffer.push(letter);
    }
}

enum Token {
    Key(String),
    Value(Value),
}

pub fn parse_string(toml: String) -> Table {
    println!("PARSING:\n---\n{toml}\n---");
    let mut ctx = Context::new();
    let chars = toml.into_bytes();
    let mut chars = chars.iter();

    while let Some(letter) = chars.next() {
        match letter {
            b'#' => {
                for letter in chars.by_ref() {
                    if letter == &b'\n' {
                        ctx.newline();
                        break;
                    }
                }
            }
            b'\n' => ctx.newline(),
            _ => ctx.buffer(*letter),
        }
    }

    ctx.root
}
