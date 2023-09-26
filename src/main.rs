use std::env;
mod repository;

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() == 0 {
        println!("git_rs");
        return;
    }
    match &*args[0] {
        "add" => {

        }
        "cat-file" => {
            
        }
        "check-ignore" => {
            
        }
        "checkout" => {
            
        }
        "commit" => {
            
        }
        "hash-object" => {
            
        }
        "init" => {
            
        }
        "log" => {
            
        }
        "ls-files" => {
            
        }
        "ls-tree" => {
            
        }
        "rev-parse" => {
            
        }
        "rm" => {
            
        }
        "show-ref" => {
            
        }
        "status" => {
            
        }
        "tag" => {
            
        }
        _ => {
            println!("Invalid argument provided!");
            return;
        }
    }
}
