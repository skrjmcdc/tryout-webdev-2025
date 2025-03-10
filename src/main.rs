use std::{
    env,
	fs,
	io::{
		prelude::{Read, Write},
		BufReader
	},
	net::{TcpListener},
	path::PathBuf,
};

use urlencoding;
use uuid;
use base62;

use tryout::Tryout;
use tryout::Error;

use tryout::server::{lololol, HtmlFormData};

#[derive(Debug)]
struct Config {

	work_dir: PathBuf,
	path_to_pages: PathBuf,
	path_to_tryouts: PathBuf,
}

fn main() {

	let config = parse_config();
	println!("{:?}", config);

    let listener = TcpListener::bind("127.0.0.1:12345").unwrap();

	let mut connections: usize = 0;

	println!("{}", lololol());

	for stream in listener.incoming() {
		connections += 1;
		println!("Connection established! #{connections}");

		let mut stream = stream.unwrap();

		let mut buffer = [0; 1024];
		let mut buf_reader = BufReader::new(&mut stream);
		let result = buf_reader.read(&mut buffer);
		println!("{:?}", result);

		let request = String::from_utf8_lossy(&buffer);
		println!("===Request: ===\n{request}\n===End Request===");

		let (request_line, rest) = request
            .split_once("\r\n")
            .unwrap_or(("", ""));

        let (headers, body) = rest
            .split_once("\r\n\r\n")
            .unwrap_or(("", ""));

		let content_length: Option<usize> = headers
			.split("\r\n")
			.find(|x| x.starts_with("Content-Length"))
			.map(|x| x.split_once(":"))
			.flatten()
			.map(|x| x.1.trim().parse().ok())
			.flatten();
		
		let body = &body[..content_length.unwrap_or(0)];

		let (response_line, filename) = match request_line {
			"GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html"),
			"GET /contoh-tryout HTTP/1.1" =>
                ("HTTP/1.1 200 OK", "details.html"),
            "GET /edit HTTP/1.1" => ("HTTP/1.1 200 OK", "edit.html"),
			"GET /style.css HTTP/1.1" => ("HTTP/1.1 200 OK", "style.css"),
			"POST /submit HTTP/1.1" => {
				let tryout = parse_tryout_from_raw_post_body(&body[..]);
				if let Ok(tryout) = tryout {
					let result = store_tryout(&config.path_to_tryouts, tryout);
				}
				("HTTP/1.1 404 Not Found", "404.html")
			},
			_ => ("HTTP/1.1 404 Not Found", "404.html"),
		};

		println!("Response: {}", response_line);

		let content: String = fs::read_to_string(
			config.path_to_pages.join(filename)
		).unwrap();

		let response = format!(
			"{}\r\nContent-Length: {}\r\n\r\n{}",
			response_line,
			content.len(),
			content,
		);

		stream.write_all(response.as_bytes()).unwrap();
	}
}

fn parse_config() -> Config {
	let work_dir = env::current_dir().unwrap();
	Config {
		work_dir,
		path_to_pages: ["pages"].iter().collect(),
		path_to_tryouts: ["data", "tryouts"].iter().collect(),
	}
}

fn fetch_tryout(id: &str, path_to_tryouts: &PathBuf) -> Result<Tryout, Error> {
    let data = fs::read(path_to_tryouts.join(id));
	match data {
		Err(_) => Err(Error::Other),
		Ok(data) => Tryout::from_bytes(&data[..]),
	}
}

fn store_tryout(path_to_tryouts: &PathBuf, tryout: Tryout) -> Result<String, Error> {
	let id = match tryout.get_id() {
		None => uuid::Uuid::new_v4(),
		Some(i) => *i,
	};
	let id = base62::encode(id.as_u128());
	fs::write(path_to_tryouts.join(&id), tryout.to_bytes()?)?;
	Ok(id)
}

fn parse_tryout_from_raw_post_body(body: &str) -> Result<Tryout, Error> {
	Tryout::from_form_data(HtmlFormData::from_url_encoded_post_body(body)?)
}
