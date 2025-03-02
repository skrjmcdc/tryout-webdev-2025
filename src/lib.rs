#[derive(Debug)]
pub struct Tryout {
	questions: Vec<Question>,
}

#[derive(Debug)]
enum Question {
	TrueOrFalse,
}

impl Tryout {
	pub fn from_raw_bytes(data: &[u8]) -> Result<Self, ()> {
		let size = data.len();
		if size < 2 {
			return Err(());
		}
		let version = data[0];
		let num_questions = data[1] as usize;
		if size < (num_questions) + 2 {
			return Err(());
		}
		let mut questions = Vec::with_capacity(num_questions);
		for i in data.iter().skip(2).take(num_questions) {
			let question = match i {
				0 => Question::TrueOrFalse,
				_ => return Err(()),
			};
			questions.push(question);
		}
		Ok(Self {
			questions,
		})
	}
}
