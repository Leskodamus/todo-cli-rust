# todo
A lightweight and super fast cli todo program written in rust

use `cargo build --release` to compile todo and copy `target/release/todo` to `/usr/bin`

## usage
```Usage: todo [COMMAND] [ARGUMENTS]
todo is a super fast and simple tasks organizer written in rust

Available commands:
    - add [TASK/s] 
        adds new task/s
        Example: todo add \"read a book\"
    - list
        lists all tasks
        Example: todo list
    - done [INDEX]
        marks task as done
        Example: todo done 2 3 (marks second and third tasks as completed)
    - undone [INDEX]
        reverts done task to undone
        Example: todo undone 3 (no longer marks the third task as completed)
    - rm [INDEX] 
        removes a task
        Example: todo rm 4 
    - sort
        sorts completed and uncompleted tasks
        Example: todo sort 
    - raw [todo|done]
        prints nothing but done/undone tasks in plain text, useful for scripting
        Example: todo raw done
```
