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

pub struct Set<T> {
    items: Vec<T>
}

impl <T>Set<T> {
    pub fn remove(&mut self, index: usize) -> T {
        self.items.remove(index)
    }

    pub fn add(&mut self, item: T)
        where T: PartialEq {
        if self.contains(&item) {
            return;
        }
        self.items.push(item);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn contains(&self, item: &T) -> bool
        where T: PartialEq {
        self.items.contains(item)
    }

    pub fn new() -> Self {
        Self {
            items: Vec::new()
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.items.iter_mut()
    }

    pub fn remove_item(&mut self, item: &T) -> Option<T>
        where T: PartialEq {
        let index = self.items.iter().position(|x| x == item);
        index.map(|index| self.remove(index))
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn append<G: AsRef<Vec<T>>>(&mut self, other: &G)
        where T: Clone + PartialEq {
        for item in other.as_ref().iter() {
            self.add(item.clone());
        }
    }
}
