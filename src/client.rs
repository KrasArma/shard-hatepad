use serde::{Deserialize, Serialize};
use std::io::{self, Write, Read};
use std::net::TcpStream;
use chrono::Utc; 

#[derive(Serialize, Deserialize, Debug)]
struct Note {
    title: String,
    content: String,
    created_at: String,
}

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    println!("Подключено к серверу");

    loop {
        println!("Введите команду (add, list, get, edit, delete, exit):");
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();

        if command == "exit" {
            break;
        }

        match command {

            "add" => {
                println!("Введите заголовок заметки:");
                let mut title = String::new();
                io::stdin().read_line(&mut title).unwrap();
                let title = title.trim();
        
                println!("Введите содержимое заметки:");
                let mut content = String::new();
                io::stdin().read_line(&mut content).unwrap();
        
                let note = Note {
                    title: title.to_string(),
                    content: content.trim().to_string(),
                    created_at: Utc::now().to_string(),
                };

                let request = serde_json::to_string(&note).unwrap();
                let request_str = format!("add,{}", request);
                stream.write_all(request_str.as_bytes()).unwrap();

                let mut buffer = [0; 1024];
                let bytes_read = stream.read(&mut buffer).unwrap();
                let response_str = String::from_utf8_lossy(&buffer[..bytes_read]);
        
                println!("{}", response_str);
            }
            "list" => {
                let request = "list,"; 
                stream.write_all(request.as_bytes()).unwrap();

                let mut buffer = [0; 1024];
                let bytes_read = stream.read(&mut buffer).unwrap();
                let response_str = String::from_utf8_lossy(&buffer[..bytes_read]);

                println!("Список заметок:\n{}", response_str);
            }
            "get" => {
                println!("Введите заголовок заметки:");
                let mut title = String::new();
                io::stdin().read_line(&mut title).unwrap();
                let title = title.trim();

                let request_str = format!("get,{}", title);
                stream.write_all(request_str.as_bytes()).unwrap();

                let mut buffer = [0; 1024];
                let bytes_read = stream.read(&mut buffer).unwrap();
                let response_str = String::from_utf8_lossy(&buffer[..bytes_read]);

                println!("{}", response_str);
            }
            "edit" => {
                println!("Введите заголовок заметки для редактирования:");
                let mut title = String::new();
                io::stdin().read_line(&mut title).unwrap();
                let title = title.trim();

                println!("Введите новое содержимое заметки:");
                let mut content = String::new();
                io::stdin().read_line(&mut content).unwrap();

                let request_str = format!("edit,{},{},", title, content.trim());
                stream.write_all(request_str.as_bytes()).unwrap();

                let mut buffer = [0; 1024];
                let bytes_read = stream.read(&mut buffer).unwrap();
                let response_str = String::from_utf8_lossy(&buffer[..bytes_read]);

                println!("{}", response_str);
            }
            "delete" => {
                println!("Введите заголовок заметки для удаления:");
                let mut title = String::new();
                io::stdin().read_line(&mut title).unwrap();
                let title = title.trim();

                let request_str = format!("delete,{}", title);
                stream.write_all(request_str.as_bytes()).unwrap();

                let mut buffer = [0; 1024];
                let bytes_read = stream.read(&mut buffer).unwrap();
                let response_str = String::from_utf8_lossy(&buffer[..bytes_read]);

                println!("{}", response_str);
            }
            _ => {
                println!("Неверная команда.");
            }
        }
    }
    println!("Соединение закрыто.");
}