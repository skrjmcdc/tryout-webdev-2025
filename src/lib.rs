use urlencoding;
use std::{
	io::{Read, BufReader},
	mem,
};
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

	pub const MAX_MULTIPLE_CHOICE_AMOUNT: usize = 10;

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

		let mut current_title: Option<String> = None;
		let mut current_description: Option<String> = None;
		let mut questions = Vec::new();

		let mut current_question_id: Option<u32> = None;
		let mut current_prompt: Option<String> = None;
		let mut current_type: Option<u32> = None;
		let mut current_choices = Vec::new();

		let mut build_question = |
			p: &mut Option<String>,
			t: &mut Option<u32>,
			c: &mut Vec<Choice>,
		| -> Result<Question, Error> {
			let p = p.take().ok_or(Error::MissingQuestionPrompt)?;
			let t = t.take().ok_or(Error::MissingQuestionType)?;
			let kind = match t {
				1 => {
					if c.len() > 2 {
						return Err(Error::TooManyChoices);
					}
					if c.len() < 2 {
						return Err(Error::NotEnoughChoices);
					}
					let mut iter = c.drain(..);
					QuestionKind::TrueOrFalse(iter.next().unwrap(), iter.next().unwrap())
				}
				2 => {
					if c.len() > Self::MAX_MULTIPLE_CHOICE_AMOUNT {
						return Err(Error::TooManyChoices);
					}
					if c.len() < 1 {
						return Err(Error::NotEnoughChoices);
					}
					let mut iter = c.drain(..);
					QuestionKind::MultipleChoice(iter.collect())
				}
				3 => QuestionKind::Essay,
				_ => return Err(Error::UnknownQuestionType),
			};
			Ok(Question{
				kind,
				prompt: p,
			})
		};

		for field in data.into_iter() {

			let (name, value) = field.into();

			if name == "t" {
				match current_title {
					None => current_title = Some(value),
					Some(_) => return Err(Error::Other),
				}
				continue;
			} else if name == "d" {
				match current_description {
					None => current_description = Some(value),
					Some(_) => return Err(Error::Other),
				}
				continue;
			}

			let (id, name) = name.split_once('_').ok_or(Error::Other)?;
			let id = match id.parse::<u32>() {
				Ok(i) => i,
				Err(_) => return Err(Error::InvalidQuestionId),
			};

			match current_question_id {
				None => current_question_id = Some(id),
				Some(i) => {
					if i != id {
						questions.push(build_question(
							&mut current_prompt,
							&mut current_type,
							&mut current_choices,
						)?);
					}
					current_question_id = Some(id);
				}
			}

			let prefix = name.chars().next().ok_or(Error::Other)?;
			match prefix {
				'q' => match current_prompt {
					None => current_prompt = Some(value),
					Some(_) => return Err(Error::Other),
				}
				't' => match current_type {
					None => match value.parse::<u32>() {
						Ok(i) => current_type = Some(i),
						Err(_) => return Err(Error::Other),
					},
					Some(_) => return Err(Error::Other),
				}
				'o' | 'c' => match current_type {
					None => return Err(Error::MissingQuestionType),
					Some(i) => match i {
						1 => {
							if current_choices.len() >= 2 {
								return Err(Error::TooManyChoices);
							} else {
								current_choices.push(Choice {
									name: value,
									correct: prefix == 'c',
								});
							}
						},
						2 => {
							if current_choices.len() >= Self::MAX_MULTIPLE_CHOICE_AMOUNT {
								return Err(Error::TooManyChoices);
							} else {
								current_choices.push(Choice {
									name: value,
									correct: prefix == 'c',
								});
							}
						},
						3 => return Err(Error::Other),
						_ => return Err(Error::UnknownQuestionType),
					}
				},
				_ => return Err(Error::Other),
			}
		}

		Ok(Self{
			title: current_title.unwrap_or(String::new()),
			description: current_description.unwrap_or(String::new()),
			questions,
		})
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
	InvalidQuestionId,
	MissingCorrectAnswer,
	MissingQuestionPrompt,
	MissingQuestionType,
	NotEnoughChoices,
	TooManyChoices,
	UnexpectedEOF,
	UnknownQuestionType,
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
