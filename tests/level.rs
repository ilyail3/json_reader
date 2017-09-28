extern crate json_reader;

use std::io::Cursor;
use json_reader::LevelReader;

#[test]
fn level_reader(){
    let mut buf = Cursor::new("{\"Records\":[{},{}]}");
    let mut r = LevelReader::new(&mut buf,2);

    assert_eq!(r.next().unwrap().unwrap(), "{}".as_bytes());
    assert_eq!(r.next().unwrap().unwrap(), "{}".as_bytes());
    assert!(r.next().is_none());
}