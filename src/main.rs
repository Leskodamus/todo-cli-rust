use std::env;
use todo::{Todo, help};

fn main() {
    let todo = Todo::new().expect("couldn't create the todo instance");

    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() > 1 {
        let command = &args[0];

        match &command[..] {
            "list" => todo.list(),
            "add" => todo.add(&args[2..]),
            "rm" => todo.remove(&args[2..]),
            "done" => todo.done(&args[2..]),
            "raw" => todo.raw(&args[2..]),
            "sort" => todo.sort(),
            "help" | "--help" | "-h" | _ => help(),
        }
    } else {
        todo.list();
    }
}
