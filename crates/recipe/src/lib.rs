use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

type GuestPath = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct Recipe {
	pub files: BTreeMap<GuestPath, FileSource>,
	pub executable_path: String,
	pub stdin_path: String,
	pub arg0: Option<String>,
	#[serde(default)]
	pub arguments: Vec<ArgumentSource>,
	#[serde(default)]
	pub environment: Environment,
}

impl Recipe {
	pub fn deserialize_from(bytes: &[u8]) -> Result<Self, RecipeError> {
		let string = std::str::from_utf8(bytes)?;
		let mut ret: Self = serde_json::from_str(string).map_err(|recipe_err| {
			match serde_json::from_str::<'_, serde_json::Value>(string) {
				Ok(_) => RecipeError::NotSyntacticRecipe(recipe_err),
				Err(json_err) => RecipeError::NotJson(json_err),
			}
		})?;

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
		for env_value in &mut ret.environment.add.values_mut() {
			match env_value {
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
	NotSyntacticRecipe(serde_json::Error),
	NotSemanticRecipe(String),
}

impl From<std::str::Utf8Error> for RecipeError {
	fn from(inner: std::str::Utf8Error) -> Self {
		Self::NotUtf8(inner)
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FileSource {
	Host(String),
	/// Symbolic files are fixed-length, containing arbitrary bytes
	SymbolicContent {
		seed: String,
		#[serde(default)]
		symbolic: Vec<SymbolicRange>,
	},
	/// Symbolic files are fixed-length, containing arbitrary bytes
	SymbolicHost {
		host_path: String,
		#[serde(default)]
		symbolic: Vec<SymbolicRange>,
	},
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ArgumentSource {
	Concrete(String),
	/// Symbolic arguments are fixed-length, containing bytes 1-255
	Symbolic {
		seed: String,
		#[serde(default)]
		symbolic: Vec<SymbolicRange>,
	},
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Environment {
	pub inherit: bool,
	#[serde(default)]
	pub remove: Vec<String>,
	#[serde(default)]
	pub add: BTreeMap<String, EnvVarSource>,
}

impl Default for Environment {
	fn default() -> Self {
		Self {
			inherit: true,
			remove: Vec::new(),
			add: BTreeMap::new(),
		}
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EnvVarSource {
	Concrete(String),
	/// Symbolic env vars are fixed-length, containing bytes 1-255
	Symbolic {
		value: String,
		symbolic: Vec<SymbolicRange>,
	},
}

#[derive(Serialize, Deserialize, Debug)]
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

	pub fn len(&self) -> u64 {
		match self {
			Self::Index(_) => 1,
			Self::Begin(_, ()) => u64::MAX,
			Self::Range(a, b) => b - a,
		}
	}
}
