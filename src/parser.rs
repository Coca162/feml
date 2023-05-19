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

    // fn parse(&mut self) {
    //     let mut chars = self.buffer.iter();
    //     if let Some(letter) = chars.next() {
    //         match letter {
    //             b'[' => {
    //                 if let Some(letter) = chars.next() {
    //                     // Array table
    //                     if letter == &b'[' {
    //                     } else {
    //                         // Table
    //                         let mut name_buffer = Vec::<u8>::new();
    //                         name_buffer.push(*letter);

    //                         for letter in chars {
    //                             if letter == &b']' {
    //                                 if let Ok(name) = std::str::from_utf8(&name_buffer) {
    //                                     let mut hierarchy = name.split('.');
    //                                     let initial_table = self.root.get_or_insert(
    //                                         hierarchy.next().unwrap().to_string(),
    //                                         Value::Table(Table::default()),
    //                                     );
    //                                     if let Value::Table(table) = hierarchy.fold(
    //                                         initial_table,
    //                                         |last_table, table_name| {
    //                                             if let Value::Table(last_table) = last_table {
    //                                                 last_table.get_or_insert(
    //                                                     table_name.to_string(),
    //                                                     Value::Table(Table::default()),
    //                                                 )
    //                                             } else {
    //                                                 panic!(
    //                                                     "Parent for `{table_name}` is not a Table!"
    //                                                 );
    //                                             }
    //                                         },
    //                                     ) {
    //                                         self.current_table = Some(table);
    //                                     } else {
    //                                         panic!("Failed to create table `{name}` - it existed as another value!")
    //                                     }
    //                                     break;
    //                                 } else {
    //                                     panic!("Failed to parse table name as UTF-8.")
    //                                 }
    //                             } else {
    //                                 name_buffer.push(*letter);
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //             b'"' => {}
    //             _ => {}
    //         }
    //     }
    //     self.buffer.clear();
    // }

    // pub fn newline(&'a mut self) {
    //     self.line += 1;
    //     self.parse();
    // }

    pub fn buffer(&mut self, letter: u8) {
        self.buffer.push(letter);
    }
}

#[test]
fn bruh() {
    let mut buff = Vec::new();
    assert_eq!(parse_value(&mut r#"{ { } }"#.bytes(), &mut buff), Some(ValueType::Table));
    buff.clear();

    assert_eq!(parse_value(&mut r#"[ [ [], [] ] ], 
    [ [], [] ], 
    [ [], [] ]]"#.bytes(), &mut buff), Some(ValueType::Array));
    buff.clear();

    assert_eq!(parse_value(&mut r#"'''
Hello
My Old
Friend'''"#.bytes(), &mut buff), Some(ValueType::LiteralString("Hello\nMy Old\nFriend".to_string())));
    buff.clear();

    let mut inline_table = r#"[ ], empty = { }, epic = 420, bruh = nan }"#.bytes();
    assert_eq!(parse_value(&mut inline_table, &mut buff), Some(ValueType::Array));
    assert_eq!(inline_table.collect::<Vec<u8>>(), b", empty = { }, epic = 420, bruh = nan }");
    buff.clear();

    assert_eq!(parse_value(&mut r#"'69'"#.bytes(), &mut buff), Some(ValueType::LiteralString("69".to_string())));
    buff.clear();

    assert_eq!(parse_value(&mut r#""yeet""#.bytes(), &mut buff), Some(ValueType::String));
    buff.clear();

    assert_eq!(parse_value(&mut r#"69"#.bytes(), &mut buff), Some(ValueType::Integer(69)));
    buff.clear();

    assert_eq!(parse_value(&mut r#"6.9"#.bytes(), &mut buff), Some(ValueType::Float(6.9)));
    buff.clear();

    match parse_value(&mut r#"nan"#.bytes(), &mut buff) {
        Some(ValueType::Float(nan)) if nan.is_nan() => (),
        failed => panic!("{failed:?} is not nan!")
    }
    buff.clear();

    assert_eq!(parse_value(&mut r#"+inf"#.bytes(), &mut buff), Some(ValueType::Float(f64::INFINITY)));
    buff.clear();

    assert_eq!(parse_value(&mut r#"-inf"#.bytes(), &mut buff), Some(ValueType::Float(f64::NEG_INFINITY)));
    buff.clear();
}

#[derive(PartialEq, Debug)]
enum ValueType {
    String,
    LiteralString(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Table,
    Array,
    PossibleDateTime
}

fn parse_value(bytes: &mut impl Iterator<Item = u8>, buffer: &mut Vec<u8>) -> Option<ValueType> {
    let first = loop {
        match bytes.next()? {
            b' ' => (),
            byte => break byte 
        }
    };
    
    let mut bool_pattern = None;

    match first {
        b'"' => {
            consume_string(bytes, buffer)?;
            return Some(ValueType::String)
        },
        b'\'' => {
            consume_literal_string(bytes, buffer)?;
            let string = std::str::from_utf8(buffer).unwrap().to_string();
            return Some(ValueType::LiteralString(string))
        },
        b'{' => {
            consume_brackets::<b'{', b'}'>(bytes, buffer)?;
            return Some(ValueType::Table)
        },
        b'[' => {
            consume_brackets::<b'[', b']'>(bytes, buffer)?;
            return Some(ValueType::Array)
        },
        b't' => {
            buffer.push(first);
            bool_pattern = Some((b"rue".iter(), true))
        },
        b'f' => {
            buffer.push(first);
            bool_pattern = Some((b"alse".iter(), false));
        } 
        b'0' => {
            buffer.push(first);
            let byte = bytes.next();
            match byte {
                Some(b' ') | Some(b'\n') | None => ValueType::Integer(0),
                Some(current) => {
                    let next = bytes.next()?;
                    let valid_int = matches!(next, b'1'..=b'9');
                    
                    if matches!(current, b'h' | b'o' | b'b') && valid_int {
                        buffer.pop()?;
                    }

                    loop {
                        let byte = bytes.next();
                        match byte {
                            Some(b'\n') | Some(b',') | None => break,
                            Some(byte) => buffer.push(byte),
                        }
                    }

                    let string = std::str::from_utf8(buffer).unwrap();

                    return Some(match (current, valid_int) {
                        (b'h', true) => ValueType::Integer(i64::from_str_radix(string, 16).ok()?),
                        (b'o', true) => ValueType::Integer(i64::from_str_radix(string, 8).ok()?),
                        (b'b', true) => ValueType::Integer(i64::from_str_radix(string, 2).ok()?),
                        (_, _) => ValueType::PossibleDateTime
                    });
                },
            };
        }
        b'1'..=b'9' | b'+' | b'-' => buffer.push(first),
        _ => buffer.push(first)
    };

    for byte in bytes {
        let next_match = bool_pattern.as_mut().map_or((None, None), |(iter, bools)| (iter.next(), Some(*bools)));

        match next_match {
            (Some(next_match), Some(_)) if next_match != &byte => {
                bool_pattern = None;
            }
            (Some(_), Some(_)) => (),
            (None, Some(bools)) => {
                return Some(ValueType::Boolean(bools))
            }
            (_, None) => ()
        }

        match byte {
            b'\n' | b',' => break,
            _ => buffer.push(byte),
        }
    }

    let string = std::str::from_utf8(buffer).unwrap();

    Some(string.parse::<i64>().ok().map(ValueType::Integer).or_else(|| string.parse::<f64>().ok().map(ValueType::Float)).unwrap_or(ValueType::PossibleDateTime))
}

fn consume_string(bytes: &mut impl Iterator<Item = u8>, buffer: &mut Vec<u8>) -> Option<()> {      
    const DELIMITER: u8 = b'"';

    let is_multi_line = handle_multiline::<DELIMITER>(bytes, buffer)?;

    let mut escaped = false;
    
    loop {
        let byte: u8 = bytes.next()?;
        match byte {
            DELIMITER if !escaped  && handle_possible_ending::<DELIMITER>(is_multi_line, bytes)? => return Some(()),
            b'\n' if !is_multi_line => return None,
            b'\\' => escaped = true,
            _ => escaped = false
        }
        buffer.push(byte);
    };
}

fn consume_literal_string(bytes: &mut impl Iterator<Item = u8>, buffer: &mut Vec<u8>) -> Option<()> {      
    const DELIMITER: u8 = b'\'';

    let is_multi_line = handle_multiline::<DELIMITER>(bytes, buffer)?;
    
    loop {
        let byte = bytes.next()?;
        match byte {
            DELIMITER if handle_possible_ending::<DELIMITER>(is_multi_line, bytes)? => return Some(()),
            b'\n' if !is_multi_line => return None,
            _ => ()
        }
        buffer.push(byte);
    };
}

fn handle_multiline<const DELIMITER: u8>(bytes: &mut impl Iterator<Item = u8>, buffer: &mut Vec<u8>) -> Option<bool> {
    let is_multi_line;
    let second = bytes.next()?;
    if second == DELIMITER {
        if bytes.next()? == DELIMITER {
            is_multi_line = true;

            match bytes.next()? {
                b'\n' => (),
                byte => buffer.push(byte)
            }
        } else {
            return None;
        }
    } else {
        buffer.push(second);
        is_multi_line = false
    }
    Some(is_multi_line)
}

fn handle_possible_ending<const DELIMITER: u8>(is_multi_line: bool, bytes: &mut impl Iterator<Item = u8>) -> Option<bool> {
    if !is_multi_line || (bytes.next()? == DELIMITER && bytes.next()? == DELIMITER) {
        loop {
            let byte = bytes.next();
            match byte {
                Some(b'\n') | Some(b',') | None => return Some(true),
                Some(_) => (),
            }
        }
    }

    Some(false)
}

fn consume_brackets<const START: u8, const END: u8>(bytes: &mut impl Iterator<Item = u8>, buffer: &mut Vec<u8>) -> Option<()> {
    let mut start = 1;
    let mut end = 0;
    
    for byte in bytes {
        buffer.push(byte);

        if byte == START {
            start += 1;
        }

        if byte == END {
            end += 1;
        }

        if start == end  {
            return Some(())
        }
    };

    None
}

enum Token {
    Key(String),
    Value(Value),
}

// pub fn parse_string(toml: String) -> Table {
//     println!("PARSING:\n---\n{toml}\n---");
//     let mut ctx = Context::new();
//     let chars = toml.into_bytes();
//     let mut chars = chars.iter();

//     while let Some(letter) = chars.next() {
//         match letter {
//             b'#' => {
//                 for letter in chars.by_ref() {
//                     if letter == &b'\n' {
//                         ctx.newline();
//                         break;
//                     }
//                 }
//             }
//             b'\n' => ctx.newline(),
//             _ => ctx.buffer(*letter),
//         }
//     }

//     ctx.root
// }
