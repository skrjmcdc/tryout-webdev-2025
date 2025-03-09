use crate::Error;

use urlencoding;

pub struct HtmlFormData(Vec<HtmlFormField>);

pub struct HtmlFormField {
	name: String,
	value: String,
}

impl HtmlFormData {
	pub fn from_url_encoded_post_body(body: &str) -> Result<Self, Error> {

		fn parse_item(item: &str) -> Result<HtmlFormField, Error> {
			let item = item.split_once('=').ok_or(Error::Other)?;
			let name = urlencoding::decode(item.0)?.into_owned();
			let value = urlencoding::decode(item.1)?.into_owned();
			Ok(HtmlFormField::new(name, value))
		}

		Ok(Self(body.split('&')
			.map(parse_item)
			.collect::<Result<_, _>>()?
		))
	}
}

impl IntoIterator for HtmlFormData {

	type Item = HtmlFormField;
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl HtmlFormField {
	fn new(name: String, value: String) -> Self {
		Self {
			name,
			value,
		}
	}
}

impl Into<(String, String)> for HtmlFormField {
	fn into(self) -> (String, String) {
		(self.name, self.value)
	}
}

pub fn lololol() -> u32 {
	4
}
