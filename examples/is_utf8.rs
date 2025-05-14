use std::io::Write;

pub fn main() {
    match utf8_supported::utf8_supported() {
        utf8_supported::Utf8Support::UTF8 => println!("is_utf8: UTF-8 ✅"),
        utf8_supported::Utf8Support::ASCII => println!("is_utf8: ASCII"),
        utf8_supported::Utf8Support::Latin1 => std::io::stdout()
            .write_all(b"is_utf8: Latin 1: \xb2\xb3\xb9\n")
            .unwrap(),
        utf8_supported::Utf8Support::Other => println!("is_utf8: Other"),
        utf8_supported::Utf8Support::Unknown => println!("is_utf8: Unknown"),
    }
}
