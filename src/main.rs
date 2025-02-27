use std::{
	io::prelude::*,
	net::{TcpListener},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:12345").unwrap();

	let mut connections: usize = 0;
	for stream in listener.incoming() {
		connections += 1;
		println!("Connection established! #{connections}");
		let mut stream = stream.unwrap();

		let response = "HTTP/1.1 200 OK\r\n\
			Content-Length: 10\r\n\r\n\
			Halo Dunia";
		stream.write_all(response.as_bytes()).unwrap();
	}
}
