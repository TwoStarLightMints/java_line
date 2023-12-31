use std::{
    env, fs,
    io::{Read, Write},
    path::PathBuf,
};

use serde::Deserialize;
use toml;

#[derive(Debug, Clone)]
struct NotJavaLineProject;

impl std::fmt::Display for NotJavaLineProject {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Not inside a valid java_line project")
    }
}

#[derive(Deserialize, Debug)]
struct JavaLineConfig {
    name: String,
}

fn init() {
    //! Initialize the root directory for the java project
    let thing = fs::DirBuilder::new();
    match thing.create(".java_line") {
        Ok(_) => println!("New root created"),
        Err(_) => println!("Root already exists"),
    }
}

// An option is used here so as to allow recursion within the function, passing in None implies this is the first traversal of directories
fn find_root(dir: Option<PathBuf>) -> Option<PathBuf> {
    //! Find the root directory of the current project, must be inside of root directory or one of its children
    //! Returns the parent directory of the .java_line directory (which marks a directory and its children as a java_line project)
    match dir {
        Some(dir_path) => {
            // This arm is typically taken after one trasversal
            if let Some(file) = fs::read_dir(dir_path.clone())
                .unwrap()
                .map(|e| {
                    let entry = e.unwrap().path();
                    fs::canonicalize(entry)
                })
                .filter(|e| e.is_ok()) // This arm is used to prevent errors with strange directories that I faced during initial testing
                .map(|e| {
                    let binding = e.unwrap();
                    let entry = binding.as_path();
                    fs::canonicalize(entry).unwrap()
                })
                .find(|e| e.file_name().unwrap().to_str().unwrap() == ".java_line")
            // This is the actual test to find the .java_line directory
            {
                // This branch handles having found the .java_line directory
                Some(file.parent().unwrap().to_path_buf()) // Return the path to the .java_line directory
            } else {
                // This branch handles not having found the .java_line directory within the currently searched directory
                let parent_dir = dir_path.parent(); // Get the parent directory
                match parent_dir {
                    Some(parent) => find_root(Some(parent.to_path_buf())), // If the directory has a parent, pass it in to continue recursion and return the result of that traversal to return at end
                    None => None,                                          // If there is no parent
                }
            }
        }
        None => {
            // This arm is typically taken as the first traversal
            let pwd = env::current_dir().unwrap();
            if let Some(file) = fs::read_dir(pwd.clone())
                .unwrap()
                .map(|e| {
                    let entry = e.unwrap().path();
                    fs::canonicalize(entry).unwrap()
                })
                .find(|e| e.file_name().unwrap().to_str().unwrap() == ".java_line")
            {
                Some(file.parent().unwrap().to_path_buf())
            } else {
                find_root(Some(pwd.parent().unwrap().to_path_buf()))
            }
        }
    }
}

fn is_java_line_project() -> bool {
    //! Returns true if the directory is a child of a java_line project or if it is the root directory
    //! Returns false in all other cases
    match find_root(None) {
        Some(_) => true,
        None => false,
    }
}

fn is_java_line_root_dir() -> bool {
    //! Checks if the pwd is the root directory of the java_line project
    match find_root(None) {
        Some(path) => path == env::current_dir().unwrap(),
        None => false,
    }
}

fn create_class(
    class_file_name: &String,
    parent_dir: Option<&String>,
    package_info: Option<String>,
) {
    //! Creates the Java file for a new class with name class_file_name
    //! If provided, creates the class within the given parent directory in parent_dir

    match parent_dir {
        Some(dir) => {
            let mut c = class_file_name.chars();

            let class_name = match c.next() {
                None => String::new(),
                Some(l) => l.to_uppercase().collect::<String>() + c.as_str(),
            };

            let mut new_class = fs::File::create(format!("{dir}/{class_name}.java")).unwrap();

            match package_info {
                Some(info) => {
                    let file_content = [
                        &format!("import {info};"),
                        "\n",
                        &format!("class {class_name} {{"),
                        "\n",
                        "\tpublic static void main(String[] args) {",
                        "\n",
                        "\t}",
                        "\n",
                        "}",
                    ];

                    new_class
                        .write_all(file_content.join("\n").as_bytes())
                        .unwrap();
                }
                None => {
                    let file_content = [
                        &format!("class {class_name} {{"),
                        "\n",
                        "\tpublic static void main(String[] args) {",
                        "\n",
                        "\t}",
                        "\n",
                        "}",
                    ];

                    new_class
                        .write_all(file_content.join("\n").as_bytes())
                        .unwrap();
                }
            }
        }
        None => {
            // The below solution to capitalizing the first character of class_file_name found on github
            // https://stackoverflow.com/questions/38406793/why-is-capitalizing-the-first-letter-of-a-string-so-convoluted-in-rust
            let mut c = class_file_name.chars();

            let class_name = match c.next() {
                None => String::new(),
                Some(l) => l.to_uppercase().collect::<String>() + c.as_str(),
            };

            let mut new_class = fs::File::create(format!("{class_name}.java")).unwrap();

            let file_content = [
                &format!("class {class_name} {{"),
                "\n",
                "\tpublic static void main(String[] args) {",
                "\n",
                "\t}",
                "\n",
                "}",
            ];

            new_class
                .write_all(file_content.join("\n").as_bytes())
                .unwrap();
        }
    }
}

// java_line add class parent_dir class_name
// java_line add class class_name
fn add_class(
    class_file_name: &String,
    parent_dir: Option<&String>,
) -> Result<(), NotJavaLineProject> {
    //! Creates a new Java class if the user is currently inside of a java_line project
    //! This is a wrapper for create_class, and should be used instead of that class
    if is_java_line_project() {
        match parent_dir {
            Some(parent) => {
                let package_info = get_package_info(&parent);

                if package_info.is_empty() {
                    create_class(class_file_name, Some(parent), None)
                } else {
                    create_class(class_file_name, Some(parent), Some(package_info));
                }
            }
            None => create_class(class_file_name, None, None),
        }
        Ok(())
    } else {
        Err(NotJavaLineProject)
    }
}

fn new_package(package_name: &String) {
    if is_java_line_project() {
        let new_pack = fs::DirBuilder::new();

        match new_pack.create(&package_name) {
            Ok(_) => (),
            Err(_) => {
                println!("Package already exists");
                return;
            }
        }

        let mut pack_decl = fs::File::create(package_name.clone() + "/pack_def.toml").unwrap();

        let pack_decl_content = format!("name=\"{}\"", package_name);

        pack_decl.write(pack_decl_content.as_bytes()).unwrap();
    } else {
        println!("You are not in a java_line project");
    }
}

fn get_package_info(target_dir: &String) -> String {
    // target_dir will be in form of path/to/dest, there should NOT be a / after dest
    let mut pack_def = match fs::File::open(format!("{target_dir}/pack_def.toml")) {
        Ok(tf) => tf,
        Err(_) => return "".to_string(),
    };

    let mut buf = String::new();

    pack_def.read_to_string(&mut buf).unwrap();

    let info: JavaLineConfig = toml::from_str(buf.as_str()).unwrap();

    info.name
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args[1] == "init" {
        // Initialize branch
        init();
    } else if args[1] == "add" {
        // Add branch
        // Class
        if args[2] == "class" {
            // Add a class
            if args.len() > 4 {
                // With source directory specified
                add_class(&args[3], Some(&args[4])).unwrap();
            } else {
                // Without source directory specified
                add_class(&args[3], None).unwrap();
            }
        }
        // Package
        if args[2] == "package" {
            // Add a package
            new_package(&args[3]);
        }
    }
}
