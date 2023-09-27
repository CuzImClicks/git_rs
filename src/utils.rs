use std::path::Path;
use std::string::FromUtf8Error;

use sha1::{Digest, Sha1};
use sha1::digest::FixedOutput;

#[cfg(not(target_os = "windows"))]
pub fn adjust_canonicalization<P: AsRef<Path>>(p: &P) -> String {
    p.as_ref().canonicalize().unwrap().display().to_string()
}


#[cfg(target_os = "windows")]
pub fn adjust_canonicalization<P: AsRef<Path>>(p: &P) -> String {
    const VERBATIM_PREFIX: &str = r#"\\?\"#;
    let p = p.as_ref().canonicalize().unwrap().display().to_string();
    if p.starts_with(VERBATIM_PREFIX) {
        p[VERBATIM_PREFIX.len()..].to_string()
    } else {
        p
    }
}


fn sha1<T: AsRef<str>>(data: T) -> Result<String, FromUtf8Error> {
    let mut hasher = Sha1::new();
    hasher.update(data.as_ref().as_bytes());
    let sha = hasher.finalize_fixed().to_vec();
    String::from_utf8(sha)
}
