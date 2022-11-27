use colored::*;
use std::fs::OpenOptions;
use std::io::{BufReader, Write, BufRead};
use std::path::PathBuf;
use std::{process, env, path::Path};

type Tasks = Vec<String>;

pub struct Todo {
    pub tasks: Tasks,       /* vector containing all tasks */
    pub n_tasks: usize,     /* initial number of tasks */
    pub file_path: PathBuf, /* path of todo file  */
}

impl Todo {
    /* Create a new todo instance which creates the todo file 
     * TODO: use config file and/or env var for file path */
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
            .enumerate()
            .map(|(idx, task)| task.expect(
                format!("failed to read todo file while reading line {}", idx+1).as_str()
            ))
            .collect();

        let n_tasks = tasks.len();

        Ok(Self { tasks, n_tasks, file_path })
    }

    /* List all tasks */
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

    /* List either done or undone tasks without any formatting */
    pub fn raw(&self, arg: &[String]) {
        if arg.len() > 1 {
            eprintln!("todo raw takes only 1 argument, not {}", arg.len())
        } else if arg.len() < 1 {
            eprintln!("todo raw needs 1 argument [done|undone]");
        } else {
            for task in &self.tasks {
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

    /* Write content of tasks vector to todo file 
     * If append is true, append the file if the vector size has increased */
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

    /* Add new task to todo and save to file */
    pub fn add(&mut self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo add needs at least 1 argument");
            process::exit(1);
        } else {
            for task in args {
                self.tasks.push(format!("[ ] {}", task))
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

    /* Remove a task from todo */
    pub fn remove(&mut self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo rm needs at least 1 argument");
            process::exit(1);
        } else {
            /* If (not the last) element gets removed, all remaining indices 
             * have to get reduced by one for each removal */
            let mut decr_next = 0;
            for arg in args {
                let idx = arg.parse::<usize>().unwrap() - 1 - decr_next;
                if idx < self.tasks.len() {
                    /* Decrease following indices only if the 
                     * removed index was not the last item */
                    if idx < self.tasks.len() - 1 {
                        decr_next += 1;
                    }
                    let _ = self.tasks.remove(idx);
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

    /* Mark a task as done */
    pub fn done(&mut self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo done needs at least 1 argument");
            process::exit(1);
        } else {
            for arg in args {
                let idx = arg.parse::<usize>().unwrap() - 1;
                if idx < self.tasks.len() {
                    let task = &self.tasks[idx];
                    if task.len() > 4 {
                        self.tasks[idx] = format!("[*] {}", &task[4..]);
                    } else {
                        eprintln!("{} corrupt todo file: task with wrong format", "warning:".red());
                        process::exit(1);
                    }
                }
                continue;
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

    /* Mark a done task as undone */
    pub fn undone(&mut self, args: &[String]) {
        if args.len() < 1 {
            eprintln!("todo undone needs at least 1 argument");
            process::exit(1);
        } else {
            for arg in args {
                let idx = arg.parse::<usize>().unwrap() - 1;
                if idx < self.tasks.len() {
                    let task = &self.tasks[idx];
                    if task.len() > 4 {
                        self.tasks[idx] = format!("[ ] {}", &task[4..]);
                    } else {
                        eprintln!("{} corrupt todo file: task with wrong format", "warning:".red());
                        process::exit(1);
                    }
                }
                continue;
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

    /* Sort tasks by their status: 
     * done tasks get placed at the bottom */
    pub fn sort(&mut self) {
        for idx in 0..self.tasks.len() {
            if self.tasks[idx].len() > 4 {
                match &self.tasks[idx][..4] {
                    "[ ] " => {
                        let task = self.tasks.remove(idx);
                        self.tasks.insert(0, task);
                    },
                    "[*] " => (),
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

        match self.write_to_file(false) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        }
    }
}

const TODO_HELP: &str = "Usage: todo [COMMAND] [ARGUMENTS]
todo is a super fast and simple tasks organizer written in rust

Available commands:
    - add [TASK]
        adds new task(s)
        Example: todo add \"read a book\" \"do homework\"
    - list
        lists all tasks
    - done [INDEX]
        marks task(s) with INDEX as done
        Example: todo done 2 3 (marks second and third task as completed)
    - undone [INDEX]
        reverts done task(s) with INDEX to undone
        Example: todo undone 3 (no longer marks the third task as completed)
    - rm [INDEX] 
        removes task(s) with INDEX 
        Example: todo rm 4 1 (removes first and fourth task)
    - sort
        sorts completed and uncompleted tasks
    - raw [done|undone]
        prints nothing but done/undone tasks in plain text, useful for scripting
        Example: todo raw done
";

pub fn help() {
    println!("{}", TODO_HELP);
}
