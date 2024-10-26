| [Crates.io](https://crates.io/crates/java-runtimes) | [Documentation](https://docs.rs/java-runtimes/0.3.0) | [Source code](https://github.com/Leawind/java-runtimes) |
|-----------------------------------------------------|------------------------------------------------------|---------------------------------------------------------|

# java-runtimes

`java-runtimes` is a rust library that provides functionality to detect java runtimes in current system.

## Installation

```shell
cargo add java-runtimes
```

## Usage

Detect Java runtime from environment variables

```rust
use java_runtimes::detector;

fn main() {
    let runtimes = detector::detect_java_in_environments();
    println!("Detected Java runtimes: {:?}", runtimes);
}
```

Detect Java runtimes recursively within multiple paths

```rust
use java_runtimes::detector;

fn main() {
    let runtimes = detector::detect_java_in_paths(&[
        "/usr".as_ref(),
        "/opt".as_ref(),
    ], 2);
    println!("Detected Java runtimes in multiple paths: {:?}", runtimes);
}
```
