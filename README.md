# utf8-supported

A Rust library for determining the UTF-8 support of the current terminal locale.

Example:

```rust
use std::io::Write;

match utf8_supported::utf8_supported() {
    utf8_supported::Utf8Support::UTF8 => println!("is_utf8: UTF-8 ✅"),
    utf8_supported::Utf8Support::ASCII => println!("is_utf8: ASCII"),
    utf8_supported::Utf8Support::Latin1 => std::io::stdout()
        .write_all(b"is_utf8: Latin 1: \xb2\xb3\xb9\n")
        .unwrap(),
    utf8_supported::Utf8Support::Other => println!("is_utf8: Other"),
    utf8_supported::Utf8Support::Unknown => println!("is_utf8: Unknown"),
}
```

This library can also be used to ensure that a child process runs with the C locale:

```rust
use std::process::Command;
use utf8_supported::CommandUtf8Ext;

let mut cmd = Command::new("ls");
if utf8_supported::utf8_supported() != utf8_supported::Utf8Support::UTF8 {
    cmd.set_c_locale();
}
cmd.output().unwrap();
```
