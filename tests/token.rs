extern crate json_reader;

use std::io::Cursor;
use json_reader::ReaderLexer;
use json_reader::JsonToken;

fn expect(body: &str, tokens: &[JsonToken]) {
    let mut buf = Cursor::new(body);
    let mut a = ReaderLexer::new(&mut buf);


    let mut left = Vec::from(tokens);

    while !left.is_empty(){
        let expected = left.remove(0);

        let got = match a.next() {
            None => panic!("Expecting {:?}, got None",expected),
            Some(result) => match result {
                Err(e) => panic!("Error thrown parsing:{:?}", e),
                Ok(v) => v
            }
        };

        if expected != got {
            panic!("Expected {:?} got {:?}", expected, got);
        }
    }

    match a.next() {
        None => {}
        Some(a) => panic!("Left over item after parse:{:?}", a)
    }
}

#[test]
fn parse_null() {
    expect(
        "{\"1\":[\"\"],\"a\":null,\"b\":\"1\"}",
        &[
            JsonToken::ObjectStart,
            JsonToken::Key("1".to_string()),

            JsonToken::ArrayStart,
            JsonToken::String("".to_string()),
            JsonToken::ArrayEnd,

            JsonToken::Key("a".to_string()),
            JsonToken::Null,
            JsonToken::Key("b".to_string()),
            JsonToken::String("1".to_string()),
            JsonToken::ObjectEnd
        ])
}

#[test]
fn parse_dict() {
    expect(
        "{\"b\":\"1\"}",
        &[
            JsonToken::ObjectStart,
            JsonToken::Key("b".to_string()),
            JsonToken::String("1".to_string()),
            JsonToken::ObjectEnd
        ])
}

#[test]
fn parse_array() {
    expect(
        "[\"1\",{},\"2\"]",
        &[
            JsonToken::ArrayStart,
            JsonToken::String("1".to_string()),
            JsonToken::ObjectStart,
            JsonToken::ObjectEnd,
            JsonToken::String("2".to_string()),
            JsonToken::ArrayEnd
        ]
    )
}

#[test]
fn parse_true() {
    expect(
        "true",
        &[
            JsonToken::True
        ]
    )
}

#[test]
fn parse_null_only() {
    expect(
        "null",
        &[
            JsonToken::Null
        ]
    )
}

#[test]
fn parse_false() {
    expect(
        "false",
        &[
            JsonToken::False
        ]
    )
}

#[test]
fn parse_number() {
    expect(
        "12.5",
        &[
            JsonToken::Number("12.5".to_string())
        ]
    )
}

#[test]
fn parse_negative_number() {
    expect(
        "-12.5",
        &[
            JsonToken::Number("-12.5".to_string())
        ]
    )
}

#[test]
fn parse_invalid_number() {
    let mut buf = Cursor::new("12.5.6");
    let mut r = ReaderLexer::new(&mut buf);

    assert_eq!(r.next().unwrap().unwrap(), JsonToken::Number("12.5".to_string()));
    let err = r.next().unwrap();

    match err {
        Ok(_) => panic!("Got ok, was expecting an error"),
        Err(_) => {}
    }
}