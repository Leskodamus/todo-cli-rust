use std::env;
use todo::{Todo, help};

fn main() {
    let todo = Todo::new().expect("couldn't create todo instance");

    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() >= 1 {
        let command = &args[0];

        match &command[..] {
            "list" => todo.list(),
            "add" => todo.add(&args[1..]),
            "rm" => todo.remove(&args[1..]),
            "done" => todo.done(&args[1..]),
            "undone" => todo.undone(&args[1..]),
            "sort" => todo.sort(),
            "raw" => todo.raw(&args[1..]),
            "help" | "--help" | "-h" | _ => help(),
        }
    } else {
        todo.list();
    }
}
