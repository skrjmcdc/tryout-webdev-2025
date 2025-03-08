use urlencoding;

#[derive(Debug)]
pub struct Tryout {
	questions: Vec<Question>,
}

#[derive(Debug)]
struct Question {
	kind: QuestionKind,
	prompt: String,
	choices: Vec<String>,
}

#[derive(Debug)]
enum QuestionKind {
	TrueOrFalse,
}

impl Tryout {
	pub fn from_raw_bytes(data: &[u8]) -> Result<Self, ()> {
		Err(())
	}

	pub fn from_post_body(body: &str) -> Result<Self, ()> {
		println!("Body: {}", body);
		let entries = body.split("&");
		let mut current_id: Option<u32> = None;
		let mut current_type: Option<u32> = None;
		let mut current_prompt: Option<String> = None;

		let mut current_questions: Vec<Question> = Vec::new();
		let mut current_choices: Vec<String> = Vec::new();
		
		let mut build_question = |t: &mut Option<u32>, p: &mut Option<String>, c: &mut Vec<String>| -> Result<Question, ()> {
			println!("Building question...");
			let t = {
				let x = t.take();
				*t = None;
				x
			};
			let p = {
				let x = p.take();
				*p = None;
				x
			};
			let c = {
				let x: Vec<String> = c.into_iter().map(|x| x.to_owned()).take(2).collect();
				(*c).clear();
				x
			};
			match t {
				None => Err(()),
				Some(1) => match p {
					None => Err(()),
					Some(p) => {
						if c.len() < 2 {
							Err(())
						} else {
							Ok(Question {
								kind: QuestionKind::TrueOrFalse,
								prompt: p,
								choices: c,
							})
						}
					}
				}
				Some(_) => Err(()),
			}
		};

		for entry in entries {
			println!("================================================================");
			println!("i={:?}; t={:?}; p={:?}; c={:?};", current_id, current_type, current_prompt, current_choices);
			println!("{}", entry);
			let (name, value) =  match entry.split_once("=") {
				Some((a, b)) => (a, b),
				None => continue,
			};
			let (id, field) =  match name.split_once("_") {
				Some((a, b)) => (a, b),
				None => continue,
			};
			let id: u32 = match id.parse() {
				Ok(i) => i,
				Err(_) => continue,
			};
			match current_id {
				None => current_id = Some(id),
				Some(i) => {
					if i != id {
						if let Ok(question) = build_question(&mut current_type, &mut current_prompt, &mut current_choices) {
							current_questions.push(question);
						}
					}
					current_id = Some(id);
				},
			}
			if field.len() < 1 {
				continue;
			}
			match field.chars().nth(0).unwrap() {
				'q' => match current_prompt { 
					None => current_prompt = Some(urlencoding::decode(&value[..].replace("+"," ")).unwrap().into_owned()),
					Some(_) => continue,
				},
				't' => match current_type {
					None => match value.parse::<u32>() {
						Ok(i) => match i {
							1 => current_type = Some(1),
							_ => continue,
						},
						Err(_) => continue,
					}
					Some(_) => continue,
				}
				'o' => {
					if field.len() < 2 {
						continue;
					}
					match field[1..].parse::<u32>() {
						Ok(i) => {
							if current_choices.len() as u32 == i {
								current_choices.push(value.to_string());
							} else {
								continue;
							}
						},
						Err(i) => continue,
					}
				}
				_ => continue,
			}
		}
		if let Ok(question) = build_question(&mut current_type, &mut current_prompt, &mut current_choices) {
			current_questions.push(question);
		}
		println!("{:?}", current_questions);
		Err(())
	}
}
