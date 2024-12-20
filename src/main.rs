use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    id: u32,
    description: String,
    completed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TaskList {
    username: String,
    tasks: Vec<Task>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AppData {
    task_lists: Vec<TaskList>,
    users: HashMap<String, User>,
}

impl AppData {
    fn new() -> Self {
        AppData {
            task_lists: Vec::new(),
            users: HashMap::new(),
        }
    }

    fn add_task(&mut self, username: &str, description: String) {
        let task_list = self.task_lists.iter_mut().find(|list| list.username == username);
        match task_list {
            Some(list) => {
                let id = list.tasks.len() as u32 + 1;
                let task = Task {
                    id,
                    description,
                    completed: false,
                };
                list.tasks.push(task);
            },
            None => {
                let task_list = TaskList {
                    username: username.to_string(),
                    tasks: vec![Task {
                        id: 1,
                        description,
                        completed: false,
                    }],
                };
                self.task_lists.push(task_list);
            }
        }
    }

    fn remove_task(&mut self, username: &str, task_id: u32) {
        if let Some(list) = self.task_lists.iter_mut().find(|list| list.username == username) {
            list.tasks.retain(|task| task.id != task_id);
        }
    }

    fn edit_task(&mut self, username: &str, task_id: u32, new_description: String) {
        if let Some(list) = self.task_lists.iter_mut().find(|list| list.username == username) {
            if let Some(task) = list.tasks.iter_mut().find(|task| task.id == task_id) {
                task.description = new_description;
            }
        }
    }

    fn mark_completed(&mut self, username: &str, task_id: u32) {
        if let Some(list) = self.task_lists.iter_mut().find(|list| list.username == username) {
            if let Some(task) = list.tasks.iter_mut().find(|task| task.id == task_id) {
                task.completed = true;
            }
        }
    }

    fn save(&self) -> io::Result<()> {
        let task_file = OpenOptions::new().create(true).write(true).open("tasks.json")?;
        serde_json::to_writer(task_file, &self.task_lists)?;

        let user_file = OpenOptions::new().create(true).write(true).open("users.json")?;
        let users: Vec<User> = self.users.values().cloned().collect();
        serde_json::to_writer(user_file, &users)?;

        Ok(())
    }

    fn load() -> io::Result<Self> {
        let mut app_data = AppData::new();

        let path = Path::new("tasks.json");
        if path.exists() {
            let file = File::open(path)?;
            app_data.task_lists = serde_json::from_reader(file)?;
        }

        let path = Path::new("users.json");
        if path.exists() {
            let file = File::open(path)?;
            let users: Vec<User> = serde_json::from_reader(file)?;
            for user in users {
                app_data.users.insert(user.username.clone(), user);
            }
        }

        Ok(app_data)
    }

    fn register_user(&mut self, username: String, password: String) -> io::Result<()> {
        if self.users.contains_key(&username) {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "User already exists"));
        }

        let hashed_password = hash(password, DEFAULT_COST).unwrap();
        let user = User { username, password: hashed_password };
        self.users.insert(user.username.clone(), user);

        Ok(())
    }

    fn authenticate(&self, username: &str, password: &str) -> bool {
        if let Some(user) = self.users.get(username) {
            verify(password, &user.password).unwrap_or(false)
        } else {
            false
        }
    }

    fn display_tasks(&self, username: &str) {
        if let Some(list) = self.task_lists.iter().find(|list| list.username == username) {
            println!("Tasks for {}:", username);
            for task in &list.tasks {
                let status = if task.completed { "Completed" } else { "Pending" };
                println!("ID: {}, Description: {}, Status: {}", task.id, task.description, status);
            }
        } else {
            println!("No tasks found for {}", username);
        }
    }
}

fn main() {
    let mut app_data = AppData::load().unwrap_or_else(|_| AppData::new());

    println!("Enter 1 to register a new user or anything else to log in: ");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    let choice = choice.trim();

    if choice == "1" {
        println!("Enter username for new user: ");
        let mut new_user_username = String::new();
        io::stdin().read_line(&mut new_user_username).unwrap();
        let new_user_username = new_user_username.trim();

        println!("Enter password: ");
        let mut new_user_password = String::new();
        io::stdin().read_line(&mut new_user_password).unwrap();
        let new_user_password = new_user_password.trim();

        if let Err(e) = app_data.register_user(new_user_username.to_string(), new_user_password.to_string()) {
            println!("Error: {}", e);
        } else {
            println!("User successfully registered!");
        }
    }

    println!("Enter username: ");
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim();

    println!("Enter password: ");
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    let password = password.trim();

    if app_data.authenticate(username, password) {
        println!("Authentication successful!");

        loop {
            println!("\nMenu:");
            println!("1. View tasks");
            println!("2. Add task");
            println!("3. Remove task");
            println!("4. Edit task");
            println!("5. Mark task as completed");
            println!("6. Save and exit");

            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();
            let choice = choice.trim();

            match choice {
                "1" => {
                    app_data.display_tasks(username);
                }
                "2" => {
                    println!("Enter task description:");
                    let mut description = String::new();
                    io::stdin().read_line(&mut description).unwrap();
                    app_data.add_task(username, description.trim().to_string());
                }
                "3" => {
                    println!("Enter task ID to remove:");
                    let mut task_id_str = String::new();
                    io::stdin().read_line(&mut task_id_str).unwrap();
                    let task_id: u32 = task_id_str.trim().parse().unwrap();
                    app_data.remove_task(username, task_id);
                }
                "4" => {
                    println!("Enter task ID to edit:");
                    let mut task_id_str = String::new();
                    io::stdin().read_line(&mut task_id_str).unwrap();
                    let task_id: u32 = task_id_str.trim().parse().unwrap();

                    println!("Enter new task description:");
                    let mut new_description = String::new();
                    io::stdin().read_line(&mut new_description).unwrap();
                    app_data.edit_task(username, task_id, new_description.trim().to_string());
                }
                "5" => {
                    println!("Enter task ID to mark as completed:");
                    let mut task_id_str = String::new();
                    io::stdin().read_line(&mut task_id_str).unwrap();
                    let task_id: u32 = task_id_str.trim().parse().unwrap();
                    app_data.mark_completed(username, task_id);
                }
                "6" => {
                    app_data.save().unwrap();
                    println!("Data saved. Exiting...");
                    break;
                }
                _ => println!("Invalid choice, please try again."),
            }
        }
    } else {
        println!("Authentication failed.");
    }
}
