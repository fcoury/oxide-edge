use oxide::command::getlog;

mod common;

#[test]
fn test_getlog() {
    let response = getlog::run().unwrap();
    assert_eq!(response.get_f64("ok").unwrap(), 1.0);
    assert_eq!(response.get_i32("totalLinesWritten").unwrap(), 8);
}
