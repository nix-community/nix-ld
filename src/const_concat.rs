//! Const concatenation utilities
//!
//! This module provides compile-time string and byte slice concatenation
//! functionality to replace the external `constcat` crate dependency.

/// Concatenate const string expressions into a static string slice.
///
/// This macro works similarly to `std::concat!` but supports const variables
/// and expressions, not just literals.
#[macro_export]
macro_rules! const_concat {
    ($($expr:expr),* $(,)?) => {{
        const fn concat_str_len(strings: &[&str]) -> usize {
            let mut total_len = 0;
            let mut i = 0;
            while i < strings.len() {
                let s = strings[i];
                let mut j = 0;
                while j < s.len() {
                    total_len += 1;
                    j += 1;
                }
                i += 1;
            }
            total_len
        }

        const fn concat_str(_strings: &[&str]) -> &'static str {
            const STRINGS: &[&str] = &[$($expr),*];

            const LEN: usize = concat_str_len(STRINGS);
            const fn inner() -> [u8; LEN] {
                let mut result = [0u8; LEN];
                let mut pos = 0;
                let mut i = 0;
                while i < STRINGS.len() {
                    let s = STRINGS[i];
                    let bytes = s.as_bytes();
                    let mut j = 0;
                    while j < bytes.len() {
                        result[pos] = bytes[j];
                        pos += 1;
                        j += 1;
                    }
                    i += 1;
                }
                result
            }
            const BYTES: [u8; LEN] = inner();
            unsafe { core::str::from_utf8_unchecked(&BYTES) }
        }
        concat_str(&[$($expr),*])
    }};
}

/// Concatenate const byte slice expressions into a static byte slice.
///
/// This macro concatenates const `&[u8]` expressions and literals into a static
/// byte slice, supporting both literals and const variables.
#[macro_export]
macro_rules! const_concat_slices {
    ([$type:ty]: $($expr:expr),* $(,)?) => {{
        const fn concat_slices_len<T>(slices: &[&[T]]) -> usize {
            let mut total_len = 0;
            let mut i = 0;
            while i < slices.len() {
                total_len += slices[i].len();
                i += 1;
            }
            total_len
        }

        const fn concat_slices<T: Copy, const N: usize>(slices: &[&[T]]) -> [T; N] {
            let mut result = [slices[0][0]; N]; // Initialize with first element
            let mut pos = 0;
            let mut i = 0;
            while i < slices.len() {
                let slice = slices[i];
                let mut j = 0;
                while j < slice.len() {
                    result[pos] = slice[j];
                    pos += 1;
                    j += 1;
                }
                i += 1;
            }
            result
        }

        const SLICES: &[&[$type]] = &[$($expr),*];
        const LEN: usize = concat_slices_len(SLICES);
        const RESULT: [$type; LEN] = concat_slices::<$type, LEN>(SLICES);
        &RESULT
    }};
}

/// Re-export the macros with constcat-compatible names
pub use const_concat as concat;
pub use const_concat_slices as concat_slices;

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_const_concat() {
        const PREFIX: &str = "NIX_LD_";
        const SUFFIX: &str = "x86_64_linux";
        const RESULT: &str = const_concat!(PREFIX, SUFFIX);
        assert_eq!(RESULT, "NIX_LD_x86_64_linux");
    }

    #[test]
    fn test_const_concat_slices() {
        const PATH: &[u8] = b"/run/current-system/sw/share/nix-ld/lib/ld.so";
        const NULL: &[u8] = b"\0";
        const RESULT: &[u8] = const_concat_slices!([u8]: PATH, NULL);
        assert_eq!(RESULT, b"/run/current-system/sw/share/nix-ld/lib/ld.so\0");
    }

    #[test]
    fn test_multiple_concat() {
        const A: &str = "NIX_";
        const B: &str = "LD_";
        const C: &str = "LIBRARY_";
        const D: &str = "PATH";
        const RESULT: &str = const_concat!(A, B, C, D);
        assert_eq!(RESULT, "NIX_LD_LIBRARY_PATH");
    }
}
