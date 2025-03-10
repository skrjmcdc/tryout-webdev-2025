use std::{
    env,
	fs,
	io::{
		prelude::{Read, Write},
		BufReader
	},
	net::{TcpListener, TcpStream},
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

		handle_connection(&config, stream);
	}
}

fn handle_connection(config: &Config, mut stream: TcpStream) {
	let mut buffer = [0; 65536];
	let result = stream.read(&mut buffer);
	let n = match result {
		Ok(n) => {
			if n == 65536 {
				return;
			}
			n
		}
		Err(_) => return,
	};

	let request = &buffer[..n];
	let mut i = 0;
	let mut d = 0;
	let mut request_line = None;
	let mut headers = Vec::new();
	let mut body = None;

	while i < n {
		if request[i] == b'\r' {
			d = 1;
		} else if request[i] == b'\n' && d == 1 {
			request_line = Some(&request[0..i-1]);
			i += 1;
			d = 2;
			break;
		} else {
			d = 0;
		}
		i += 1;
	}

	let request_line = match request_line {
		Some(x) => String::from_utf8_lossy(x),
		None => return,
	};

	let mut start = i;
	while i < n {
		if request[i] == b'\r' {
			d |= 1;
		} else if request[i] == b'\n' && d % 2 == 1 {
			d += 1;
			if d == 2 {
				headers.push(&request[start..i-1]);
				start = i + 1;
			}
			if d == 4 {
				body = Some(&request[i+1..n]);
				break;
			}
		} else {
			d = 0;
		}
		i += 1;
	}

	let headers: Vec<_> = headers.iter().map(|x| String::from_utf8_lossy(x)).collect();
	println!("Request: {}", request_line);
	println!("======== Headers ========");
	for header in headers {
		println!("{}", header);
	}

	println!("======== Body ========");
	println!("{:?}", String::from_utf8_lossy(body.unwrap_or(&[])));

	let mut drain = request_line.split(' ');
	let method = match drain.next() {
		Some(x) => x,
		None => return,
	};
	let uri = match drain.next() {
		Some(x) => x,
		None => return,
	};
	let version = match drain.next() {
		Some(x) => x,
		None => return,
	};

	if version != "HTTP/1.1" {
		return;
	}

	let (status_line, filename, redirect) = match (method, uri) {
		("GET", "/") => ("HTTP/1.1 200 OK", Some("index.html"), None),
		("GET", "/style.css") => ("HTTP/1.1 200 OK", Some("style.css"), None),
		("GET", "/edit") => {
			("HTTP/1.1 200 OK", Some("edit.html"), None)
		},
		("GET", uri) => {
			if uri.starts_with("/details/") {
				let id = uri.strip_prefix("/details/").unwrap_or(&"");
				if is_legal_id(id) {
					//TODO: Display tryout details
					("HTTP/1.1 404 Not Found", Some("404.html"), None)
				} else {
					("HTTP/1.1 404 Not Found", Some("404.html"), None)
				}
			} else {
				("HTTP/1.1 404 Not Found", Some("404.html"), None)
			}
		},
		("POST" , "/submit") => {
			let tryout = parse_tryout_from_raw_post_body(&String::from_utf8_lossy(body.unwrap())).unwrap();
			let destination = store_tryout(&config.path_to_tryouts, tryout).unwrap();
			let mut path = String::from("/details/");
			path.push_str(&destination);
			("HTTP/1.1 303 See Other", None, Some(path))
		},
		_ => ("HTTP/1.1 404 Not Found", Some("404.html"), None),
	};

	let content = match filename {
		Some(x) => String::from_utf8_lossy(&fs::read(config.path_to_pages.join(x)).unwrap()).to_string(),
		None => String::from(""),
	};

	let response = format!("{}{}{}\r\n\r\n{}",
		status_line,
		match status_line {
			"HTTP/1.1 303 See Other" => format!("\r\nLocation: {}", redirect.unwrap()),
			_ => String::from(""),
		},
		format!("\r\nContent-Length: {}", content.len()),
		content,
	);

	println!("Response:\n{}", response);
	stream.write_all(response.as_bytes()).unwrap();
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

fn is_legal_id(id: &str) -> bool {
	base62::decode(id).is_ok()
}
