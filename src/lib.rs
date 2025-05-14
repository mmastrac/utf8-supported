#![doc = include_str!("../README.md")]

#[cfg(unix)]
use std::ffi::OsStr;

const LOCALE_ENVIRONMENT_VARIABLES: &[&str] = &["LC_ALL", "LC_CTYPE", "LANG"];

#[cfg(unix)]
enum LocaleSignal {
    Unknown,
    UTF8,
    ASCII,
    NonUTF8,
}

#[cfg(unix)]
fn strstr_ignore_case(haystack: &OsStr, needle: &[u8]) -> bool {
    use std::os::unix::ffi::OsStrExt;

    debug_assert!(
        needle.iter().all(|c| c.to_ascii_lowercase() == *c),
        "needle must contain only ASCII lowercase characters"
    );

    let hlen = haystack.len();
    let nlen = needle.len();
    if hlen < nlen {
        return false;
    }
    let haystack = haystack.as_bytes();

    'outer: for i in 0..(hlen - nlen + 1) as usize {
        if haystack[i].to_ascii_lowercase() == needle[0] {
            for j in 1..nlen {
                if haystack[i + j].to_ascii_lowercase() != needle[j] {
                    continue 'outer;
                }
            }
            return true;
        }
    }
    false
}

#[cfg(unix)]
fn get_locale_signal(value: &OsStr) -> LocaleSignal {
    if value.is_empty() {
        LocaleSignal::Unknown
    } else if value == "C" || value == "c" || value == "POSIX" || value == "posix" {
        LocaleSignal::ASCII
    } else if strstr_ignore_case(value, b"utf-8") || strstr_ignore_case(value, b"utf8") {
        LocaleSignal::UTF8
    } else {
        LocaleSignal::NonUTF8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Utf8Support {
    /// The locale is unset, or at least as far as we can tell.
    #[default]
    Unknown,
    /// The locale is ASCII.
    ASCII,
    /// The locale is Latin-1 (Windows only).
    Latin1,
    /// The locale is UTF-8.
    UTF8,
    /// The locale is set, and not ASCII or UTF-8.
    Other,
}

/// Determine the UTF-8 support of the current locale.
pub fn utf8_supported() -> Utf8Support {
    #[cfg(unix)]
    return utf8_supported_unix();
    #[cfg(windows)]
    return utf8_supported_windows();
    #[cfg(not(any(unix, windows)))]
    return Utf8Support::Unset;
}

/// Determine the UTF-8 support of the current locale by locating the first
/// signal.
#[cfg(unix)]
fn utf8_supported_unix() -> Utf8Support {
    let mut signal = Utf8Support::Unknown;
    for &var in LOCALE_ENVIRONMENT_VARIABLES {
        let locale = std::env::var_os(var).unwrap_or_default();
        match get_locale_signal(locale.as_os_str()) {
            LocaleSignal::UTF8 => return Utf8Support::UTF8,
            LocaleSignal::ASCII => return Utf8Support::ASCII,
            LocaleSignal::NonUTF8 => signal = Utf8Support::Other,
            LocaleSignal::Unknown => {}
        }
    }
    signal
}

#[cfg(windows)]
fn utf8_supported_windows() -> Utf8Support {
    use windows_sys::Win32::System::Console::*;
    match unsafe { GetConsoleOutputCP() } {
        65001 => Utf8Support::UTF8,
        20127 => Utf8Support::ASCII,
        1252 => Utf8Support::Latin1,
        0 => Utf8Support::Unknown,
        // Should we expose this?
        437 => Utf8Support::Other,
        _ => Utf8Support::Other,
    }
}

/// A trait for setting the locale of a subprocess to C.
pub trait CommandUtf8Ext {
    /// Ensure that a child subprocess runs with the C locale (Unix only).
    #[cfg(unix)]
    fn set_c_locale(&mut self);
}

impl CommandUtf8Ext for std::process::Command {
    #[cfg(unix)]
    fn set_c_locale(&mut self) {
        self.env("LANG", "C");
        self.env("LC_ALL", "C");
        self.env("LC_CTYPE", "C");
    }
}

#[derive(Debug, Default)]
#[cfg(windows)]
pub struct CodePageHandle(u32, u32);

#[cfg(windows)]
impl Drop for CodePageHandle {
    fn drop(&mut self) {
        use windows_sys::Win32::System::Console::*;
        unsafe {
            if self.0 != 0 {
                SetConsoleOutputCP(self.0);
            }
            if self.1 != 0 {
                SetConsoleCP(self.1);
            }
        }
    }
}

/// Set the console code page to UTF-8 (Windows only).
///
/// This function returns a handle that will reset the console code page to the
/// original value when dropped.
///
/// # Example
///
/// ```rust
/// let handle = set_console_utf8().unwrap_or_default();
/// // Use UTF-8 here...
/// drop(handle);
/// // ...and restore the original code page when done.
/// ```
#[must_use = "The returned CodePageHandle resets the console code page to the original value when dropped"]
#[cfg(windows)]
pub fn set_console_utf8() -> Result<CodePageHandle, std::io::Error> {
    use windows_sys::Win32::Globalization::*;
    use windows_sys::Win32::System::Console::*;

    unsafe {
        let original_output_cp = GetConsoleOutputCP();
        let original_input_cp = GetConsoleCP();

        // Returns a nonzero value if the code page is valid, or 0 if the code page is invalid
        if IsValidCodePage(65001) != 0 {
            SetConsoleOutputCP(65001);
            SetConsoleCP(65001);
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "UTF-8 codepage (65001) is not available",
            ));
        }

        Ok(CodePageHandle(original_output_cp, original_input_cp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn test_strstr() {
        assert!(strstr_ignore_case(OsStr::new("UTF-8"), b"utf-8"));
        assert!(strstr_ignore_case(OsStr::new("XUTF-8"), b"utf-8"));
        assert!(strstr_ignore_case(OsStr::new("UTF-8X"), b"utf-8"));
        assert!(strstr_ignore_case(OsStr::new("utf-8x"), b"utf-8"));
        assert!(strstr_ignore_case(OsStr::new("utf-8"), b"utf-8"));
        assert!(strstr_ignore_case(OsStr::new("xutf-8"), b"utf-8"));
        assert!(!strstr_ignore_case(OsStr::new("UTF-16"), b"utf-8"));
        assert!(!strstr_ignore_case(OsStr::new("utf16"), b"utf-8"));
        assert!(!strstr_ignore_case(OsStr::new("utf8"), b"utf-8"));
    }
}
