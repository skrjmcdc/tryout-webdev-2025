use std::{
    env,
	fs,
	io::{
		prelude::{Read, Write},
		BufReader
	},
	net::{TcpListener},
};

use tryout::Tryout;

fn main() {
	
    let listener = TcpListener::bind("127.0.0.1:12345").unwrap();

	let mut connections: usize = 0;

	for stream in listener.incoming() {
		connections += 1;
		println!("Connection established! #{connections}");

		let mut stream = stream.unwrap();

		let mut buffer = [0; 1024];
		let mut buf_reader = BufReader::new(&mut stream);
		let result = buf_reader.read(&mut buffer);
		println!("{:?}", result);

		let request = String::from_utf8_lossy(&buffer);
		println!("{request}");

		let (header, _) = match request.split_once("\r\n") {
			Some((a, b)) => (a, b),
			None => ("", ""),
		};

		let (response_line, filename) = match header {
			"GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html"),
			"GET /contoh-tryout HTTP/1.1" =>
                ("HTTP/1.1 200 OK", "details.html"),
            "GET /edit HTTP/1.1" => ("HTTP/1.1 200 OK", "edit.html"),
			_ => ("HTTP/1.1 404 Not Found", "404.html"),
		};

		let content: String = fs::read_to_string(filename).unwrap();

		let response = format!(
			"{}\r\nContent-Length: {}\r\n\r\n{}",
			response_line,
			content.len(),
			content,
		);

		stream.write_all(response.as_bytes()).unwrap();
	}
}

fn fetch_tryout(id: &str) -> Result<Tryout, ()> {
	let data_path = {
        let mut path = env::current_dir().unwrap();
        path.push("data");
        path.push(id);
        path
    };
    let data = fs::read(data_path);
	match data {
		Err(_) => Err(()),
		Ok(data) => Tryout::from_raw_bytes(&data[..]),
	}
}
