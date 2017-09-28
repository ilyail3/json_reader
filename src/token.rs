use std::io::{Read, Bytes};
use std::io::{Result, Error, ErrorKind};
use std::iter::Peekable;

const LF: u8 = '\n' as u8;
const CR: u8 = '\r' as u8;
const SPACE: u8 = ' ' as u8;
const TAB: u8 = '\t' as u8;
const COMMA: u8 = ',' as u8;
const DOT: u8 = '.' as u8;
const MINUS: u8 = '-' as u8;
const ARRAY_OPEN: u8 = '[' as u8;
const ARRAY_CLOSE: u8 = ']' as u8;
const OBJECT_OPEN: u8 = '{' as u8;
const OBJECT_CLOSE: u8 = '}' as u8;
const NUMBER0: u8 = '0' as u8;
const NUMBER9: u8 = '9' as u8;
const ESCAPE_CHAR: u8 = '\\' as u8;
const CHAR_N: u8 = 'n' as u8;
const CHAR_T: u8 = 't' as u8;
const CHAR_R: u8 = 'r' as u8;
const CHAR_F: u8 = 'f' as u8;
const CHAR_DOUBLE_QUOTE: u8 = '"' as u8;


macro_rules! error {
    ($msg:expr) => {
        Err(Error::new(ErrorKind::Other,$msg))
    };
    ($fmt:expr, $($arg:expr), *) => {
        Err(Error::new(ErrorKind::Other, format!($fmt, $($arg ),*)))
    };
}

#[derive(Debug,PartialEq)]
enum Container {
    Document,
    Object,
    Array
}

#[derive(Debug,Clone,PartialEq)]
pub enum JsonToken {
    ObjectStart,
    ArrayStart,
    ObjectEnd,
    ArrayEnd,
    Key(String),
    String(String),
    Number(String),
    Null,
    True,
    False
}

pub struct ReaderLexer<R> where R: Read {
    read: Peekable<Bytes<R>>,
    container: Vec<Container>,
    next_key: bool,
    elem_sep: bool
}



impl<R> ReaderLexer<R> where R: Read {
    pub fn new(r: R) -> ReaderLexer<R> {
        let mut c = Vec::new();

        c.push(Container::Document);

        ReaderLexer {
            read: r.bytes().peekable(),
            container: c,
            next_key: false,
            elem_sep: false
        }
    }

    fn next_byte(&mut self) -> Option<Result<u8>> {
        let mut return_byte:u8 = LF;

        while return_byte == LF as u8 || return_byte == CR as u8 || return_byte == TAB as u8 || return_byte == SPACE as u8 {
            match self.read.next() {
                None => return None,
                Some(r) => match r {
                    Err(e) => return Some(Err(e)),
                    Ok(b) => return_byte = b
                }
            }
        }

        if self.elem_sep {
            if return_byte == COMMA {
                self.elem_sep = false;
                return self.next_byte();
            } else if return_byte == ARRAY_CLOSE || return_byte == OBJECT_CLOSE {
                // The read string function will rest to true
                self.elem_sep = false;
            } else {
                return Some(error!("Expecting separator got:{}", return_byte));
            }
        }

        Some(Ok(return_byte))
    }

    fn read_string(&mut self) -> Result<String> {
        let mut escape = false;
        let mut result_string: Vec<u8> = Vec::new();
        let mut byte: u8;
        let mut done: bool = false;

        while !done {
            let next = self.read.next();

            if next.is_none() {
                return error!("Stream ended before string termination");
            }

            match next.unwrap() {
                Err(e) => return Err(e),
                Ok(b) => {
                    byte = b;

                    if escape {
                        match byte {
                            ESCAPE_CHAR => result_string.push(ESCAPE_CHAR),
                            CHAR_N => result_string.push(LF),
                            CHAR_R => result_string.push(CR),
                            CHAR_T => result_string.push(TAB),
                            CHAR_DOUBLE_QUOTE => result_string.push(CHAR_DOUBLE_QUOTE),
                            _ => return error!("Escaping unexpected char:{}", byte)
                        };

                        escape = false;
                    } else {
                        if byte == ESCAPE_CHAR {
                            escape = true;
                        } else if byte == CHAR_DOUBLE_QUOTE {
                            done = true;
                        } else {
                            result_string.push(byte);
                        }
                    }
                }
            }
        }

        match String::from_utf8(result_string) {
            Err(_) => error!("UTF8 decode failed"),
            Ok(s) => Ok(s)
        }
    }

    fn read_number(&mut self, first_char:u8) -> Result<String> {
        let mut result_string: Vec<u8> = Vec::new();
        result_string.push(first_char);

        let mut with_decimal_point = false;

        while match self.read.peek() {
            None => false,
            Some(result) => match *result {
                Err(_) => false,
                Ok(ref c) => if !with_decimal_point && *c == DOT {
                    with_decimal_point = true;
                    true
                } else if *c >= NUMBER0 && *c <= NUMBER9 {
                    true
                } else {
                    false
                }
            }
        } {
            result_string.push(self.read.next().unwrap().unwrap())
        }

        match String::from_utf8(result_string) {
            Err(_) => error!("UTF8 decode failed"),
            Ok(s) => Ok(s)
        }
    }

    fn read_const(&mut self, const_val: &'static str) -> Result<()> {
        let mut chars = const_val.to_string();

        chars.remove(0);

        while !chars.is_empty() {
            let mut m = [0; 1];
            chars.remove(0).encode_utf8(&mut m);

            match self.read.next() {
                None => return error!("EOF when expecting string:{}", const_val),
                Some(r) => match r {
                    Err(e) => return Err(e),
                    Ok(v) => if v != m[0] {
                        return error!("String incomplete:{}", const_val);
                    }
                }
            }
        }

        Ok(())
    }
}


impl<R> Iterator for ReaderLexer<R>
    where R: Read {
    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.next_byte() {
            Some(result) => match result {
                Ok(byte) =>
                //Some(Ok(JsonToken::ObjectStart)),
                    if self.next_key && byte != CHAR_DOUBLE_QUOTE && byte != OBJECT_CLOSE {
                        Some(error!("Expected key, got:{}", byte))
                    } else if self.next_key && byte == CHAR_DOUBLE_QUOTE {
                        self.next_key = false;
                        self.elem_sep = false;

                        let key = Some(self.read_string().map(|f| JsonToken::Key(f)));
                        let next_char = self.next_byte();

                        if next_char.is_none() {
                            return Some(Err(Error::new(ErrorKind::Other, "EOF after key string")));
                        };

                        match next_char.unwrap() {
                            Err(e) => return Some(Err(e)),
                            Ok(b) => if b != 58 {
                                return Some(error!("Expected key separator, got:{}", b));
                            }
                        };

                        key
                    } else if byte == ARRAY_OPEN {
                        self.container.push(Container::Array);
                        self.elem_sep = false;

                        Some(Ok(JsonToken::ArrayStart))
                    } else if byte == ARRAY_CLOSE {
                        if self.container.pop().unwrap() != Container::Array {
                            return Some(Err(Error::new(ErrorKind::Other, "Not expecting array end")))
                        };

                        self.elem_sep = true;
                        Some(Ok(JsonToken::ArrayEnd))
                    } else if byte == CHAR_DOUBLE_QUOTE {
                        self.elem_sep = true;

                        Some(self.read_string().map(|f| JsonToken::String(f)))
                    } else if byte == OBJECT_OPEN {
                        self.container.push(Container::Object);

                        self.elem_sep = false;
                        self.next_key = true;

                        Some(Ok(JsonToken::ObjectStart))
                    } else if byte == OBJECT_CLOSE {
                        if self.container.pop().unwrap() != Container::Object {
                            return Some(Err(Error::new(ErrorKind::Other, "Not expecting obj end")))
                        };

                        self.elem_sep = true;
                        self.next_key = false;

                        Some(Ok(JsonToken::ObjectEnd))
                    } else if byte == CHAR_N /* handle null */ {
                        match self.read_const("null") {
                            Err(e) => return Some(Err(e)),
                            Ok(_) => {}
                        };


                        self.elem_sep = true;
                        Some(Ok(JsonToken::Null))
                    } else if byte == CHAR_T /* handle true */ {
                        match self.read_const("true") {
                            Err(e) => return Some(Err(e)),
                            Ok(_) => {}
                        };


                        self.elem_sep = true;
                        Some(Ok(JsonToken::True))
                    } else if byte == CHAR_F /* handle false */ {
                        match self.read_const("false") {
                            Err(e) => return Some(Err(e)),
                            Ok(_) => {}
                        };


                        self.elem_sep = true;
                        Some(Ok(JsonToken::False))
                    } else if (byte >= NUMBER0 && byte <= NUMBER9) || byte == MINUS {
                        self.elem_sep = true;


                        Some(self.read_number(byte).map(|f| JsonToken::Number(f)))
                    } else {
                        Some(error!("Unexpected character:{}", byte))
                    },
                Err(e) =>
                    Some(Err(e))
            },
            None => None
        };

        if self.elem_sep {
            match *(self.container.last().unwrap()) {
                Container::Object => {
                    self.next_key = true;
                }
                _ => {}
            }
        }

        result
    }

    type Item = Result<JsonToken>;
}