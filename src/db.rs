use std::io::Read;

use crate::Error;

pub struct Chunks(Vec<Chunk>);

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
