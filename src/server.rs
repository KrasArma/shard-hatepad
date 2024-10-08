use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt}; 

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Note {
    title: String,
    content: String,
    created_at: String,
}

struct AppState {
    notes: Mutex<HashMap<String, Note>>, 
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        notes: Mutex::new(HashMap::new()),
    });

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Сервер запущен на 127.0.0.1:8080");

    while let Ok((socket, _)) = listener.accept().await {
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            handle_client(socket, state).await;
        });
    }
}

async fn handle_client(mut socket: TcpStream, state: Arc<AppState>) {
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = match socket.read(&mut buffer).await {
            Ok(0) => return, 
            Ok(n) => n,
            Err(e) => {
                eprintln!("Ошибка чтения из сокета: {}", e);
                return;
            }
        };

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        eprintln!("Получен запрос: {}", request);

        if let Some(pos) = request.find(',') {
            let command = &request[..pos];
            let json_data = &request[pos + 1..];

            match command {
                "add" => {
                    let note: Result<Note, serde_json::Error> = serde_json::from_str(json_data);
                    match note {
                        Ok(note) => {
                            let mut notes = state.notes.lock().await;
                            notes.insert(note.title.clone(), note);
                            socket.write_all("Заметка добавлена\n".as_bytes()).await.unwrap();
                        }
                        Err(e) => {
                            eprintln!("Ошибка десериализации JSON: {}", e);
                            socket.write_all("Ошибка обработки запроса\n".as_bytes()).await.unwrap();
                        }
                    }
                }
                "list" => {
                    let notes = state.notes.lock().await;
                    let mut notes_list = String::new();
                    for (title, note) in notes.iter() {
                        notes_list.push_str(&format!("Заголовок: {}, Содержимое: {}, Создана: {}\n", title, note.content, note.created_at));
                    }
                    socket.write_all(notes_list.as_bytes()).await.unwrap();
                }
                "get" => {
                    if json_data.is_empty() {
                        socket.write_all("Неверный формат запроса\n".as_bytes()).await.unwrap();
                        continue;
                    }
                    let title = json_data;
                    let notes = state.notes.lock().await;
                    if let Some(note) = notes.get(title) {
                        let response = serde_json::to_string(&note).unwrap();
                        socket.write_all(response.as_bytes()).await.unwrap();
                    } else {
                        socket.write_all("Заметка не найдена\n".as_bytes()).await.unwrap();
                    }
                }
                "edit" => {
                    let parts: Vec<&str> = json_data.split(',').collect();
                    if parts.len() != 2 {
                        socket.write_all("Неверный формат запроса\n".as_bytes()).await.unwrap();
                        continue;
                    }
                    let title = parts[0];
                    let content = parts[1];
                    let mut notes = state.notes.lock().await;
                    if let Some(note) = notes.get_mut(title) {
                        note.content = content.to_string();
                        socket.write_all("Заметка изменена\n".as_bytes()).await.unwrap();
                    } else {
                        socket.write_all("Заметка не найдена\n".as_bytes()).await.unwrap();
                    }
                }
                "delete" => {
                    if json_data.is_empty() {
                        socket.write_all("Неверный формат запроса\n".as_bytes()).await.unwrap();
                        continue;
                    }
                    let title = json_data;
                    let mut notes = state.notes.lock().await;
                    if notes.remove(title).is_some() {
                        socket.write_all("Заметка удалена\n".as_bytes()).await.unwrap();
                    } else {
                        socket.write_all("Заметка не найдена\n".as_bytes()).await.unwrap();
                    }
                }
                _ => {
                    socket.write_all("Неверная команда\n".as_bytes()).await.unwrap();
                }
            }
        } else {
            socket.write_all("Неверный формат запроса\n".as_bytes()).await.unwrap();
        }
    }
}
