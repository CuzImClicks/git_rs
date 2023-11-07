

#[derive(Debug)]
pub enum GitObject {
    Blob { raw_data: Vec<u8> },
    Commit { 
        raw_data: Vec<u8>, 
        tree: String, 
        parent: Vec<String>, 
        authur: String,
        committer: String,
        gpgsig: Option<String>,
        message: String 
    },
    Tree {
        raw_data: Vec<u8>
    },
    Tag {
        raw_data: Vec<u8>
    }
}

impl GitObject {
    fn new(data: Vec<u8>) -> GitObject {
        
        match &*String::from_utf8(data.clone()).unwrap() {
            "blob" => GitObject::Blob { raw_data: data },
            "tree" => GitObject::Tree { raw_data: data },
            "tag" => GitObject::Tag { raw_data: data },
            _ => GitObject::Tag { raw_data: data }
        }
    }

    fn new_commit(data: Vec<u8>) -> C {
        todo!()
    }
}

fn main() {
    println!("{:?}", GitObject::new("tree".as_bytes().to_vec()));
}