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


pub fn crlf_to_lf<T: AsRef<[u8]>>(data: &T) -> Vec<u8> {
    let mut result = Vec::new();
    let mut data = data.as_ref().to_vec();
    while !data.is_empty() {
        if data[0] == 13 && data[1] == 10 {
            result.push(10);
            data.remove(0);
            data.remove(0);
        } else {
            result.push(data[0]);
            data.remove(0);
        }
    }
    result
}
