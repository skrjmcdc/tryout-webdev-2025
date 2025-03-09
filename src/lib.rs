use urlencoding;
use std::io::{Read, BufReader};

pub mod server;
use server::HtmlFormData;

#[derive(Debug)]
pub struct Tryout {
	title: String,
	description: String,
	questions: Vec<Question>,
}

#[derive(Debug)]
struct Question {
	kind: QuestionKind,
	prompt: String,
}

#[derive(Debug)]
enum QuestionKind {
	TrueOrFalse(Choice, Choice),
	MultipleChoice(Vec<Choice>),
	Essay,
}

#[derive(Debug)]
struct Choice {
	name: String,
	correct: bool,
}

impl Tryout {
	pub fn from_bytes(data: &[u8]) -> Result<Self, Error> {
		Self::from_chunks(Chunks::from_bytes(data)?)
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
		Err(Error::Other)
	}

	pub fn from_chunks(chunks: Chunks) -> Result<Self, Error> {
		Err(Error::Other)
	}

	pub fn from_form_data(data: HtmlFormData) -> Result<Self, Error> {
		Err(Error::Other)
	}
}

struct Chunks(Vec<Chunk>);

struct Chunk {
	size: u32,
	type_index: u32,
	data: Vec<u8>,
}

impl Chunk {
	fn new(type_index: u32, data: &[u8]) -> Result<Self, Error> {
		let size = data.len() as u32;
		if size < u32::MAX - 8 {
			Ok(Self{
				size,
				type_index,
				data: data.to_vec(),
			})
		} else {
			Err(Error::ChunkTooLarge)
		}
	}

	fn to_owned_bytes(&self) -> Vec<u8> {
		let mut result = Vec::with_capacity((self.size as usize) + 8);
		result.extend_from_slice(&self.size.to_be_bytes());
		result.extend_from_slice(&self.type_index.to_be_bytes());
		result.extend_from_slice(&self.data);
		result
	}
}

impl Chunks {
	fn from_bytes(mut bytes: &[u8]) -> Result<Self, Error> {

		let mut temp = [0u8; 4];
		let mut chunks = Vec::new();

		loop {

			let n = bytes.read(&mut temp)?;
			if n == 0 {
				break;
			} else if n < 4 {
				return Err(Error::UnexpectedEOF);
			}
			let size  = u32::from_be_bytes(temp[..].try_into().unwrap());

			let n = bytes.read(&mut temp)?;
			if n < 4 {
				return Err(Error::UnexpectedEOF);
			}
			let type_index = u32::from_be_bytes(temp[..].try_into().unwrap());

			let mut data = Vec::with_capacity(size as usize);
			let n = bytes.read(&mut data)?;
			if (n as u32) < size {
				return Err(Error::UnexpectedEOF);
			}

			chunks.push(Chunk {
				size,
				type_index,
				data,
			});
		}

		Ok(Self(chunks))
	}
}

#[derive(Debug)]
pub enum Error {
	ChunkTooLarge,
	UnexpectedEOF,
	UnknownTypeIndex,
	Other,
}

impl From<std::io::Error> for Error {
	fn from(value: std::io::Error) -> Self {
		Self::Other
	}
}

impl From<std::string::FromUtf8Error> for Error {
	fn from(value: std::string::FromUtf8Error) -> Self {
		Self::Other
	}
}
