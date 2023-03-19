use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

type GuestPath = String;

#[derive(Serialize, Deserialize)]
pub struct Recipe {
	pub files: BTreeMap<GuestPath, FileSource>,
	pub executable_path: String,
	pub stdin_path: String,
	pub arg0: Option<String>,
	arguments: Vec<ArgumentSource>,
	environment: Environment,
}
impl Recipe {
	pub fn deserialize_from(bytes: &[u8]) -> Result<Self, RecipeError> {
		let string = std::str::from_utf8(bytes)?;
		let mut ret: Self = match serde_json::from_str(string) {
			Ok(ret) => ret,
			Err(recipe_err) => match serde_json::from_str::<'_, serde_json::Value>(string) {
				Ok(_) => return Err(RecipeError::NotRecipe(recipe_err)),
				Err(json_err) => return Err(RecipeError::NotRecipe(json_err)),
			},
		};
		for file in ret.files.values_mut() {
			match file {
				FileSource::Host(_) => {}
				FileSource::SymbolicContent { symbolic, .. }
				| FileSource::SymbolicHost { symbolic, .. } => SymbolicRange::normalize(symbolic),
			}
		}
		for argument in &mut ret.arguments {
			match argument {
				ArgumentSource::Concrete(_) => {}
				ArgumentSource::Symbolic { symbolic, .. } => SymbolicRange::normalize(symbolic),
			}
		}
		for env in &mut ret.environment.add {
			match env {
				EnvVarSource::Concrete(_) => {}
				EnvVarSource::Symbolic { symbolic, .. } => SymbolicRange::normalize(symbolic),
			}
		}
		Ok(ret)
	}
}
#[derive(Debug)]
pub enum RecipeError {
	NotUtf8(std::str::Utf8Error),
	NotJson(serde_json::Error),
	NotRecipe(serde_json::Error),
}
impl From<std::str::Utf8Error> for RecipeError {
	fn from(inner: std::str::Utf8Error) -> Self {
		Self::NotUtf8(inner)
	}
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum FileSource {
	Host(String),
	/// Symbolic files are fixed-length, containing arbitrary bytes
	SymbolicContent {
		seed: String,
		symbolic: Vec<SymbolicRange>,
	},
	/// Symbolic files are fixed-length, containing arbitrary bytes
	SymbolicHost {
		host_path: String,
		symbolic: Vec<SymbolicRange>,
	},
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum ArgumentSource {
	Concrete(String),
	/// Symbolic arguments are fixed-length, containing bytes 1-255
	Symbolic {
		seed: String,
		symbolic: Vec<SymbolicRange>,
	},
}

#[derive(Serialize, Deserialize)]
struct Environment {
	inherit: bool,
	remove: Vec<String>,
	add: Vec<EnvVarSource>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum EnvVarSource {
	Concrete(String),
	/// Symbolic env vars are fixed-length, containing bytes 1-255
	Symbolic {
		key: String,
		value: String,
		symbolic: Vec<SymbolicRange>,
	},
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum SymbolicRange {
	Index(u64),
	Begin(u64, ()),
	Range(u64, u64),
}

#[allow(clippy::len_without_is_empty)]
impl SymbolicRange {
	fn normalize(ranges: &mut Vec<SymbolicRange>) {
		let mut v: Vec<[u64; 2]> = ranges.iter().map(Self::range).collect();
		v.sort_unstable_by_key(|&[a, b]| (a, u64::MAX - b));

		let mut ret: Vec<[u64; 2]> = Vec::new();
		v.into_iter()
			.for_each(|range| match (ret.last_mut(), range) {
				(Some([a0, b0]), [a, b]) if a <= *b0 => {
					*a0 = (*a0).min(a);
					*b0 = (*b0).max(b);
				}
				(_, range) => ret.push(range),
			});
		ranges.clear();
		ranges.extend(ret.into_iter().map(|[a, b]| Self::Range(a, b)));
	}

	fn range(&self) -> [u64; 2] {
		match *self {
			Self::Index(a) => [a, a + 1],
			Self::Begin(a, ()) => [a, u64::MAX],
			Self::Range(a, b) => [a, b],
		}
	}

	pub fn start(&self) -> u64 {
		self.range()[0]
	}

	pub fn len(&self) -> Option<u64> {
		match self {
			Self::Index(_) => Some(1),
			Self::Begin(_, ()) => None,
			Self::Range(a, b) => Some(b - a),
		}
	}
}
