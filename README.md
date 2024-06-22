# java-runtimes

`java-runtimes` is a rust library that provides functionality to detect java runtimes in current system.

## Installation

```shell
cargo add java-runtimes
```

## Usage

```rust
fn main() {
    // Detect java runtimes in environments
    let mut runtimes: Vec<JavaRuntime> = detector::detect_java_in_environments();
    
    let paths = vec![
        Path::new("/usr"),
        Path::new("/opt"),
    ];
    
    // Detect java runtimes in specific paths
    detector::gather_java_in_paths(&mut runtimes, paths, 2);
    
    println!("Detected Java runtimes: {:#?}", runtimes);
}
```
