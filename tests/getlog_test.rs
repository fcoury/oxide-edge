use oxide::command::getlog;

mod common;

#[test]
fn test_getlog() {
    let response = getlog::run().unwrap();
    assert_eq!(response.get_f64("ok").unwrap(), 1.0);
}
