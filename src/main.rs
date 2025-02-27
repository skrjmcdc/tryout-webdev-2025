use std::{
	fs,
	io::prelude::Write,
	net::{TcpListener},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:12345").unwrap();

	let mut connections: usize = 0;

	for stream in listener.incoming() {
		connections += 1;
		println!("Connection established! #{connections}");

		let mut stream = stream.unwrap();

		let content: String = fs::read_to_string("index.html").unwrap();

		let response = format!(
			"HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
			content.len(),
			content,
		);

		stream.write_all(response.as_bytes()).unwrap();
	}
}
