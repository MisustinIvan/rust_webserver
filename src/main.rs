use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

struct Server {
    listener: TcpListener,
}

impl Server {
    fn new(address: &str) -> Result<Self, String> {
        TcpListener::bind(address)
            .map_err(|e| e.to_string())
            .map(|listener| Server { listener })
    }

    fn start(&self) {
        for connection in self.listener.incoming() {
            match connection {
                Ok(stream) => {
                    self.handle_connection(stream);
                }
                Err(e) => {
                    println!("Error accepting connection: {}", e);
                }
            }
        }
    }

    fn handle_connection(&self, mut connection: TcpStream) {
        let data = self.read_request(&connection);
        if data.is_empty() {
            return;
        }

        println!("Received request:\n{:#?}", data);

        let path = self.parse_request(&data);
        let path = match path.as_str() {
            "/" => "./srv/index.html".to_string(),
            path if path.starts_with("/api") => format!("./srv{}", path),
            _ => format!("./srv{}", path),
        };

        let content = self.read_file_bytes(&path);
        let length = content.len();
        let content_type = self.filename_to_content_type(&path);
        // ------- logging -------
        println!("Sending response");
        println!("Path: {}", path);
        println!("Content-Type: {}", content_type);
        println!("Content-Length: {}", length);

        let response_headers = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            content_type, length,
        );

        if let Err(e) = connection.write_all(response_headers.as_bytes()) {
            println!("Error sending response headers: {}", e);
            return;
        }

        if let Err(e) = connection.write_all(&content) {
            println!("Error sending content: {}", e);
        }
    }

    pub fn read_request(&self, stream: &TcpStream) -> Vec<String> {
        let reader = BufReader::new(stream);
        reader
            .lines()
            .map(|line| line.map_err(|e| e.to_string()).unwrap())
            .take_while(|line| !line.is_empty())
            .collect()
    }

    fn parse_request(&self, request: &[String]) -> String {
        let path_line = request.get(0).unwrap_or(&String::new()).to_string();
        let path = path_line
            .split_whitespace()
            .nth(1)
            .unwrap_or("/")
            .to_string();
        path
    }

    fn read_file_bytes(&self, path: &str) -> Vec<u8> {
        fs::read(path).unwrap_or_else(|e| self.read_file_error(e))
    }

    fn read_file_error(&self, error: std::io::Error) -> Vec<u8> {
        format!("<p>Error: {}</p>", error.to_string())
            .bytes()
            .collect()
    }

    fn filename_to_content_type(&self, filename: &str) -> &str {
        match filename.split('.').last() {
            Some("gif") => "image/gif",
            Some("html") => "text/html",
            Some("css") => "text/css",
            Some("js") => "text/javascript",
            Some("jpg") => "image/jpeg",
            Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            _ => "text/plain",
        }
    }
}

fn main() {
    let server = Server::new("localhost:6969").unwrap();
    server.start();
}
