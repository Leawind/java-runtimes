use crate::{detector, JavaRuntime};

#[test]
fn test_extract_version() {
    assert_eq!(
        JavaRuntime::extract_version("17.0.4.1").unwrap(),
        "17.0.4.1"
    );
    assert_eq!(
        JavaRuntime::extract_version("\"17.0.4.1\"").unwrap(),
        "17.0.4.1"
    );
    assert_eq!(
        JavaRuntime::extract_version("1.8.0_333").unwrap(),
        "1.8.0_333"
    );
    assert_eq!(
        JavaRuntime::extract_version("\"1.8.0_333\"").unwrap(),
        "1.8.0_333"
    );

    let output: String = String::from(
        r#"java version "1.8.0_333"
Java(TM) SE Runtime Environment (build 1.8.0_333-b02)
Java HotSpot(TM) 64-Bit Server VM (build 25.333-b02, mixed mode)"#,
    );
    assert_eq!(JavaRuntime::extract_version(&output).unwrap(), "1.8.0_333");

    let output: String = String::from(
        r#"java version "17.0.4.1"
Java(TM) SE Runtime Environment (build 1.8.0_333-b02)
Java HotSpot(TM) 64-Bit Server VM (build 25.333-b02, mixed mode)"#,
    );
    assert_eq!(JavaRuntime::extract_version(&output).unwrap(), "17.0.4.1");
}

#[test]
fn test_detector() {
    let runtimes = detector::detect_java_in_environments();
    println!("Detected {} java runtimes in environments", runtimes.len());

    if runtimes.is_empty() {
        println!("Please specify java runtime path in environments.");
        println!("See [`java_runtimes::detector::detect_java_from_environments`]");
        assert!(false);
    }

    for (i, runtime) in runtimes.iter().enumerate() {
        println!("Java Runtime[{}]: {:#?}", i, runtime);
        let runtime = runtime.clone();
        assert!(runtime.is_same_os());
        assert!(runtime.is_available());
    }
}
