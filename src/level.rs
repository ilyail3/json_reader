use std::io::{Read, Bytes};
use std::io::Result;

const ESCAPE_CHAR: u8 = '\\' as u8;
const CHAR_DOUBLE_QUOTE: u8 = '"' as u8;
const ARRAY_OPEN: u8 = '[' as u8;
const ARRAY_CLOSE: u8 = ']' as u8;
const OBJECT_OPEN: u8 = '{' as u8;
const OBJECT_CLOSE: u8 = '}' as u8;

pub struct LevelReader<R> where R:Read {
    level:u32,
    target_level: u32,
    read:Bytes<R>,
    in_string:bool,
    escape:bool
}

impl<R> Iterator for LevelReader<R>
    where R: Read {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_char_result = self.read.next();
        let mut pre = true;
        let mut result:Vec<u8> = Vec::new();

        while !next_char_result.is_none() && (pre || self.level > self.target_level) {
            match next_char_result.unwrap() {
                Err(e) => return Some(Err(e)),
                Ok(b) => {
                    // Handle the byte
                    if self.escape {
                        self.escape = false;
                    } else if self.in_string && b == ESCAPE_CHAR {
                        self.escape = true;
                    } else if self.in_string && b == CHAR_DOUBLE_QUOTE {
                        self.in_string = false;
                    } else if !self.in_string && b == CHAR_DOUBLE_QUOTE {
                        self.in_string = true;
                    } else if !self.in_string && (b == ARRAY_OPEN || b == OBJECT_OPEN) {
                        self.level += 1;

                        if pre && self.level > self.target_level {
                            pre = false;
                        }
                    } else if !self.in_string && (b == ARRAY_CLOSE || b == OBJECT_CLOSE) {
                        self.level -= 1;
                    }

                    if !pre { result.push(b); };
                }
            }

            next_char_result = self.read.next();
        }

        if result.is_empty() {
            None
        } else {
            Some(Ok(result))
        }
    }
}

impl <R> LevelReader<R> where R:Read {
    pub fn new(r: R, target_level:u32) -> LevelReader<R> {
        LevelReader {
            level: 0,
            read: r.bytes(),
            in_string: false,
            escape: false,
            target_level
        }
    }
}