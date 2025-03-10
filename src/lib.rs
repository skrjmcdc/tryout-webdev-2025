use std::{
	io::{Read, BufReader},
	mem,
};

pub mod server;
use server::HtmlFormData;
pub mod db;
use db::{Chunks, Chunk};

use urlencoding;
use uuid::Uuid;
use base62;

#[derive(Debug)]
pub struct Tryout {
	id: Option<Uuid>,
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

	pub fn set_id() {
		
	}

	pub fn from_bytes(data: &[u8]) -> Result<Self, Error> {
		Self::from_chunks(Chunks::from_bytes(data)?)
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
		Ok(self.to_chunks()?.to_bytes())
	}

	pub fn from_chunks(chunks: Chunks) -> Result<Self, Error> {
		let mut title: Option<String> = None;
		let mut description: Option<String> = None;
		let mut questions = Vec::new();
		let mut id: Option<Option<Uuid>> = None;
		for chunk in chunks.into_iter() {
			match chunk.get_type() {
				1..= 15 => questions.push(Question::from_chunk(chunk)?),
				16 => match id {
					None => {
						let result = base62::decode(String::from_utf8_lossy(chunk.get_data()).to_string());
						match result {
							Ok(i) => id = Some(Some(Uuid::from_u128(i))),
							Err(base62::DecodeError::EmptyInput) => id = Some(None),
							Err(_) => return Err(Error::InvalidIdFormat),
						}
					}
					Some(_) => return Err(Error::DuplicateField),
				}
				17 => match title {
					None => title = Some(String::from_utf8_lossy(chunk.get_data()).to_string()),
					Some(_) => return Err(Error::DuplicateField),
				}
				18 => match description {
					None => description = Some(String::from_utf8_lossy(chunk.get_data()).to_string()),
					Some(_) => return Err(Error::DuplicateField),
				}
				_ => continue,
			}
		}
		Ok(Self {
			id: id.flatten(),
			title: title.unwrap_or(String::new()),
			description: description.unwrap_or(String::new()),
			questions,
		})
	}

	pub fn to_chunks(&self) -> Result<Chunks, Error> {
		let mut chunks: Vec<Chunk> = Vec::new();
		let id = match self.id {
			None => Vec::new(),
			Some(i) => i.as_u128().to_be_bytes().to_vec(),
		};
		chunks.push(Chunk::new(16, &id[..])?);

		chunks.push(Chunk::new(17, &self.title.as_bytes())?);
		chunks.push(Chunk::new(18, &self.description.as_bytes())?);

		for question in &self.questions {
			chunks.push(question.to_chunk()?);
		}

		let result = Ok(Chunks::from_vec(chunks));
		result
	}

	pub fn from_form_data(data: HtmlFormData) -> Result<Self, Error> {

		let mut current_title: Option<String> = None;
		let mut current_description: Option<String> = None;
		let mut questions = Vec::new();
		let mut current_id: Option<Uuid> = None;

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
			} else if name == "a" {
				match current_id {
					None => {
						if value == "" {
							current_id = None;
						} else {
							match base62::decode(&value) {
								Ok(i) => current_id = Some(Uuid::from_u128(i)),
								Err(_) => return Err(Error::InvalidIdFormat),
							}
						}
					}
					Some(_) => return Err(Error::DuplicateField),
				}
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

		questions.push(build_question(
			&mut current_prompt,
			&mut current_type,
			&mut current_choices,
		)?);

		Ok(Self{
			id: current_id,
			title: current_title.unwrap_or(String::new()),
			description: current_description.unwrap_or(String::new()),
			questions,
		})
	}

	pub fn get_id(&self) -> Option<&Uuid> {
		self.id.as_ref()
	}
}

impl Question {
	fn from_chunk(chunk: Chunk) -> Result<Self, Error> {
		let inner_chunks = Chunks::from_bytes(&chunk.get_data())?;
		let mut prompt: Option<String> = None;
		let mut choices = Vec::new();
		let question_type = chunk.get_type();
		if question_type < 1 || question_type > 3 {
			return Err(Error::UnknownTypeIndex);
		}
		for chunk in inner_chunks.iter() {
			match chunk.get_type() {
				0 => match prompt {
					None => prompt = Some(String::from_utf8_lossy(chunk.get_data()).to_string()),	
					Some(_) => return Err(Error::DuplicateField),
				},
				1 => {
					if question_type == 3 {
						return Err(Error::TooManyChoices);
					}
					let inner_inner_chunks = Chunks::from_bytes(&chunk.get_data())?;
					for chunk in inner_inner_chunks.iter() {
						choices.push(Choice {
							name: String::from_utf8_lossy(chunk.get_data()).to_string(),
							correct: match chunk.get_type() {
								0 => false,
								1 => true,
								_ => return Err(Error::UnknownTypeIndex),
							}
						});
					}
				},
				_ => return Err(Error::UnknownTypeIndex),
			}
		}
		let kind = match question_type {
			1 => {
				if choices.len() < 2 {
					return Err(Error::NotEnoughChoices);
				}
				let mut drain = choices.drain(..2);
				QuestionKind::TrueOrFalse(drain.next().unwrap(), drain.next().unwrap())
			},
			2 => {
				if choices.len() < 1 {
					return Err(Error::NotEnoughChoices);
				}
				QuestionKind::MultipleChoice(choices)
			},
			3 => QuestionKind::Essay,
			_ => unreachable!(),
		};
		Ok(Self {
			kind,
			prompt: prompt.unwrap_or(String::new()),
		})
	}

	fn to_chunk(&self) -> Result<Chunk, Error> {
		let mut data = Vec::new();
		data.extend_from_slice(&Chunk::new(0, self.prompt.as_bytes())?.to_owned_bytes());
		match &self.kind {
			QuestionKind::TrueOrFalse(a, b) => {
				let mut inner_chunks = Vec::new();
				inner_chunks.extend_from_slice(&Chunk::new(
					a.correct as u32,
					a.name.as_bytes(),
				)?.to_owned_bytes());
				inner_chunks.extend_from_slice(&Chunk::new(
					b.correct as u32,
					b.name.as_bytes(),
				)?.to_owned_bytes());
				data.extend_from_slice(&Chunk::new(1, &inner_chunks)?.to_owned_bytes());
			}
			QuestionKind::MultipleChoice(choices) => {
				return Err(Error::Other);
			}
			QuestionKind::Essay => (),
		}
		let type_index = match self.kind {
			QuestionKind::TrueOrFalse(_, _) => 1,
			QuestionKind::MultipleChoice(_) => 2,
			QuestionKind::Essay => 3,
		};
		Chunk::new(type_index, &data)
	}
}

#[derive(Debug)]
pub enum Error {
	ChunkTooLarge,
	DuplicateField,
	InvalidIdFormat,
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
