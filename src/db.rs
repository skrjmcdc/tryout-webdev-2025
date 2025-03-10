use std::io::Read;

use crate::Error;

#[derive(Debug)]
pub struct Chunks(Vec<Chunk>);

#[derive(Debug)]
pub struct Chunk {
	size: u32,
	type_index: u32,
	data: Vec<u8>,
}

impl Chunk {
	pub fn new(type_index: u32, data: &[u8]) -> Result<Self, Error> {
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

	pub fn to_owned_bytes(&self) -> Vec<u8> {
		let mut result = Vec::with_capacity((self.size as usize) + 8);
		result.extend_from_slice(&self.size.to_be_bytes());
		result.extend_from_slice(&self.type_index.to_be_bytes());
		result.extend_from_slice(&self.data);
		result
	}

	pub fn get_type(&self) -> u32 {
		self.type_index
	}

	pub fn get_data(&self) -> &[u8] {
		&self.data
	}
}

impl Chunks {
	pub fn from_vec(chunks: Vec<Chunk>) -> Self {
		Self(chunks)
	}
	pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, Error> {

		let mut temp = [0u8; 4];
		let mut chunks = Vec::new();

		loop {

			let n = bytes.read(&mut temp)?;
			if n == 0 {
				break;
			} else if n < 4 {
				return Err(Error::UnexpectedEOF);
			}
			let mut size  = u32::from_be_bytes(temp[..].try_into().unwrap());

			let n = bytes.read(&mut temp)?;
			if n < 4 {
				return Err(Error::UnexpectedEOF);
			}
			let type_index = u32::from_be_bytes(temp[..].try_into().unwrap());

			let mut data = Vec::with_capacity(size as usize);
			let mut buffer = [0u8; 1024];
			while size > 1024 {
				if let Err(_) = bytes.read_exact(&mut buffer) {
					return Err(Error::Other);
				}
				data.extend_from_slice(&buffer);
				size -= 1024;
			}
			if let Err(_) = bytes.read_exact(&mut buffer[0..size as usize]) {
				return Err(Error::Other);
			}
			data.extend_from_slice(&buffer[0..size as usize]);

			chunks.push(Chunk {
				size,
				type_index,
				data,
			});
		}

		Ok(Self(chunks))
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		let mut size = 0;
		for chunk in &self.0 {
			size += (chunk.size as usize) + 8;
		}
		let mut bytes = Vec::with_capacity(size);
		for chunk in &self.0 {
			bytes.extend_from_slice(&chunk.to_owned_bytes());
		}
		bytes
	}

	pub fn iter(&self) -> std::slice::Iter<'_, Chunk> {
		self.0.iter()
	}

	pub fn into_iter(self) -> std::vec::IntoIter<Chunk> {
		self.0.into_iter()
	}
}
