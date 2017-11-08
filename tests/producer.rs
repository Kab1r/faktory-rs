extern crate faktory;
extern crate mockstream;
extern crate serde_json;
extern crate url;

mod mock;

use faktory::*;

#[test]
fn hello() {
    let mut s = mock::Stream::default();

    let p = Producer::connect_with(s.clone(), None).unwrap();
    let written = s.pop_bytes_written(0);
    assert!(written.starts_with(b"HELLO {"));
    let written: serde_json::Value = serde_json::from_slice(&written[b"HELLO ".len()..]).unwrap();
    let written = written.as_object().unwrap();
    assert_eq!(written.get("hostname"), None);
    assert_eq!(written.get("wid"), None);
    assert_eq!(written.get("pid"), None);
    assert_eq!(written.get("labels"), None);
    assert_eq!(written.get("v").and_then(|h| h.as_i64()), Some(2));

    drop(p);
    let written = s.pop_bytes_written(0);
    assert_eq!(written, b"END\r\n");
}

#[test]
fn hello_pwd() {
    let mut s = mock::Stream::with_salt(1545, "365a858149c6e2d1");

    let c = Producer::connect_with(s.clone(), Some("test".to_string())).unwrap();
    let written = s.pop_bytes_written(0);
    assert!(written.starts_with(b"HELLO {"));
    let written: serde_json::Value = serde_json::from_slice(&written[b"HELLO ".len()..]).unwrap();
    let written = written.as_object().unwrap();
    assert_eq!(
        written.get("pwdhash").and_then(|h| h.as_str()),
        Some("dfad6c5f3a0daa4faea5788d1a913240b74fe9200f32bc48663aad4707a87056")
    );

    drop(c);
}

#[test]
fn enqueue() {
    let mut s = mock::Stream::default();
    let mut p = Producer::connect_with(s.clone(), None).unwrap();
    s.ignore(0);

    s.ok(0);
    p.enqueue(Job::new("foobar", vec!["z"])).unwrap();

    let written = s.pop_bytes_written(0);
    assert!(written.starts_with(b"PUSH {"));
    let written: serde_json::Value = serde_json::from_slice(&written[b"PUSH ".len()..]).unwrap();
    let written = written.as_object().unwrap();
    assert_eq!(written.get("jid").map(|h| h.is_string()), Some(true));
    assert_eq!(
        written.get("queue").and_then(|h| h.as_str()),
        Some("default")
    );
    assert_eq!(
        written.get("jobtype").and_then(|h| h.as_str()),
        Some("foobar")
    );
    assert_eq!(
        written
            .get("args")
            .and_then(|h| h.as_array())
            .and_then(|a| {
                assert_eq!(a.len(), 1);
                a[0].as_str()
            }),
        Some("z")
    );
    assert_eq!(
        written.get("reserve_for").and_then(|h| h.as_u64()),
        Some(600)
    );
    assert_eq!(written.get("retry").and_then(|h| h.as_u64()), Some(25));
    assert_eq!(written.get("priority").and_then(|h| h.as_u64()), Some(5));
    assert_eq!(written.get("backtrace").and_then(|h| h.as_u64()), Some(0));
}
