use colored::*;
use std::ffi::OsString;
use std::fs::{OpenOptions, self, File};
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::path::PathBuf;
use std::{process, env, path::Path};

type Tasks = Vec<String>;

pub struct Todo {
    pub tasks: Tasks,
    pub n_tasks: usize,
    pub file_path: PathBuf,
}

impl Todo {
    pub fn new() -> Result<Self, String> {
        let home = match env::consts::OS {
            "linux" | "macos" => {
                env::var_os("XDG_DATA_HOME").unwrap_or(
                    match env::var_os("HOME") {
                        Some(dir) => dir,
                        None => return Err("failed to read $XDG_DATA_HOME or $HOME".into()),
                    })
            },
            "windows" => {
                match env::var_os("USERPROFILE") {
                    Some(dir) => dir,
                    None => return Err("failed to read %USERPROFILE%".into()),
                }
            },
            _ => return Err("unsupported operating system".into()),
        };

        let file_path = Path::new(&home).join(".todo");
        let todo_file = match 
            OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .open(&file_path) 
        {
            Ok(f) => f,
            Err(e) => return Err(format!("failed to open todo file: {}", e)),
        };

        let tasks: Tasks = BufReader::new(todo_file)
            .lines()
            .map(|task| task.expect("failed to read todo file"))
            .collect();

        let n_tasks = tasks.len();

        Ok(Self { tasks, n_tasks, file_path })
    }

    pub fn list(&self) -> () {
        for (idx, task) in self.tasks.iter().enumerate() {
            let idx = (idx + 1).to_string().bold();

            if task.len() <= 4 { 
                eprintln!("{} corrupt todo file: task with wrong format", "warning:".red());
                process::exit(1);
            }

            let symbol = &task[..4];
            let task = &task[4..];

            match symbol {
                "[*] " => println!("{} {}", idx, task.strikethrough()), /* DONE */
                "[ ] " => println!("{} {}", idx, task),                 /* NOT DONE */
                _ => {
                    eprintln!("{} corrupt todo file: task with wrong symbol", "warning:".red());
                    process::exit(1);
                }
            }
        }
    }

    pub fn raw(&self, arg: &[String]) {
        if arg.len() > 1 {
            eprintln!("todo raw takes only 1 argument, not {}", arg.len())
        } else if arg.len() < 1 {
            eprintln!("todo raw needs 1 argument [done|undone]");
        } else {
            for task in self.tasks.iter() {
                if task.len() > 4 {
                    let symbol = &task[..4];
                    let task = &task[4..];
    
                    match symbol {
                        "[ ] " if arg[0] == "undone" => println!("{}", task),   /* DONE */
                        "[*] " if arg[0] == "done" => println!("{}", task),     /* NOT DONE */
                        "[ ] " | "[*] " => (),  /* do nothing; only show for appropriate arg */
                        _ => {
                            eprintln!("{} corrupt todo file: task with wrong symbol", "warning:".red());
                            process::exit(1);
                        }                    }
                } else {
                    eprintln!("{} corrupt todo file: task with wrong format", "warning:".red());
                    process::exit(1);
                }
            }
        }
    }

    fn write_to_file(&self, append: bool) -> Result<(), String> {
        let mut begin = 0;
        let mut file_opts = OpenOptions::new(); 

        if append {
            if self.tasks.len() <= self.n_tasks {
                return Err("cannot append to file because there is nothing to append".into());
            }
            begin = self.n_tasks;
            file_opts.create(true).append(true);
        } else {
            file_opts.create(true).write(true).truncate(true);
        }

        let mut todo_file = match file_opts.open(&self.file_path) {
            Ok(f) => f,
            Err(e) => return Err(format!("failed to open todo file: {}", e)),
        };

        for task in &self.tasks[begin..] {
            match todo_file.write_fmt(format_args!("{task}\n")) {
                Ok(_) => (),
                Err(e) => return Err(format!("failed to write todo file: {}", e)),
            };
        }

        Ok(())
    }

    pub fn add(&mut self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo add needs at least 1 argument");
            process::exit(1);
        } else {
            for arg in args {
                self.tasks.push(format!("[ ] {}", arg))
            }
            match self.write_to_file(true) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(1);
                }
            }
        }
    }

    pub fn remove(&mut self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo rm needs at least 1 argument");
            process::exit(1);
        } else {
            let mut decr_next = false;
            for arg in args {
                let mut idx = arg.parse::<usize>().unwrap() - 1;
                if decr_next && idx >= 1 { idx -= 1; }
                if idx < self.tasks.len() {
                    let _ = self.tasks.remove(idx);
                    decr_next = true;
                } else {
                    decr_next = false;
                }
            }
            match self.write_to_file(false) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(1);
                }
            }
        }
    }

    pub fn done(&self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo done needs at least 1 argument");
            process::exit(1);
        } else {
            let todo_file = match 
                OpenOptions::new()
                    .write(true)
                    .open(&self.file_path)
            {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("failed to open todo file: {}", e);
                    process::exit(1);
                },
            };

            let mut buf = BufWriter::new(todo_file);
            for (idx, task) in self.tasks.iter().enumerate() {
                if task.len() > 4 {
                    let fmt_task;
                    if args.contains(&(idx + 1).to_string()) {
                        fmt_task = format!("[*] {}\n", &task[4..]);
                    } else {
                        fmt_task = format!("{}\n", task);
                    }
                    match buf.write_all(fmt_task.as_bytes()) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("failed to write todo file: {}", e);
                            process::exit(1);
                        },
                    };
                } else {
                    eprintln!("{} corrupt todo file: task with wrong format", "warning:".red());
                    process::exit(1);
                }
            }
        }
    }

    pub fn undone(&self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo undone needs at least 1 argument");
            process::exit(1);
        } else {
            let todo_file = match 
                OpenOptions::new()
                    .write(true)
                    .open(&self.file_path)
            {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("failed to open todo file: {}", e);
                    process::exit(1);
                },
            };

            let mut buf = BufWriter::new(todo_file);
            for (idx, task) in self.tasks.iter().enumerate() {
                if task.len() > 4 {
                    let fmt_task;
                    if &task[..4] == "[*] " && args.contains(&(idx + 1).to_string()) {
                        fmt_task = format!("[ ] {}\n", &task[4..]);
                    } else {
                        fmt_task = format!("{}\n", task);
                    }

                    match buf.write_all(fmt_task.as_bytes()) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("failed to write todo file: {}", e);
                            process::exit(1);
                        },
                    };
                } else {
                    eprintln!("{} corrupt todo file: task with wrong format", "warning:".red());
                    process::exit(1);
                }
            }
        }
    }

    pub fn sort(&self) {
        let mut sorted_tasks = Tasks::new();

        for task in self.tasks.iter() {
            if task.len() > 4 {
                match &task[..4] {
                    "[ ] " => sorted_tasks.insert(0, format!("{}\n", task)),    /* Add undone task to beginning of vec */
                    "[*] " => sorted_tasks.push(format!("{}\n", task)),         /* Add done task to end of vec */
                    _ => {
                        eprintln!("{} corrupt todo file: task with wrong symbol", "warning:".red());
                        process::exit(1);
                    }
                }
            } else {
                eprintln!("{} corrupt todo file: task with wrong format", "warning:".red());
                process::exit(1);
            }
        }

        let mut todo_file = match 
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&self.file_path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("failed to open todo file: {}", e);
                process::exit(1);
            },
        };

        for task in sorted_tasks {
            match todo_file.write_all(task.as_bytes()) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("failed to write todo file: {}", e);
                    process::exit(1);
                },
            };
        }
    }
}

const TODO_HELP: &str = "Usage: todo [COMMAND] [ARGUMENTS]
todo is a super fast and simple tasks organizer written in rust

Available commands:
    - add [TASK/s] 
        adds new task/s
        Example: todo add \"read a book\"
    - list
        lists all tasks
        Example: todo list
    - done [INDEX]
        marks task with INDEX as done
        Example: todo done 2 3 (marks second and third tasks as completed)
    - undone [INDEX]
        reverts done task with INDEX to undone
        Example: todo undone 3 (no longer marks the third task as completed)
    - rm [INDEX] 
        removes a task
        Example: todo rm 4 
    - sort
        sorts completed and uncompleted tasks
        Example: todo sort 
    - raw [done|undone]
        prints nothing but done/undone tasks in plain text, useful for scripting
        Example: todo raw done
";

pub fn help() {
    println!("{}", TODO_HELP);
}
