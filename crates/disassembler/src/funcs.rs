// NOTE: This is something we will be using, but here for reference. This is just some example
// code using gimli.
use std::fs;

use object::Object;

pub fn foo() {
	// https://docs.rs/gimli/latest/gimli/read/index.html
	// Read the DWARF sections with whatever object loader you're using.
	// These closures should return a `Reader` instance (e.g. `EndianSlice`).
	let bin_data = fs::read("../../demos/hello").unwrap();
	let file = object::File::parse(&*bin_data).unwrap();
	let endian = if file.is_little_endian() {
		gimli::RunTimeEndian::Little
	} else {
		gimli::RunTimeEndian::Big
	};

	fn load_section<'data: 'file, 'file, O, Endian>(
		id: gimli::SectionId,
		file: &'file O,
		endian: Endian,
	) -> Result<gimli::EndianRcSlice<Endian>, gimli::Error>
	where
		O: object::Object<'data, 'file>,
		Endian: gimli::Endianity,
	{
		use object::ObjectSection;

		let data = file
			.section_by_name(id.name())
			.and_then(|section| section.uncompressed_data().ok())
			.unwrap_or(std::borrow::Cow::Borrowed(&[]));
		Ok(gimli::EndianRcSlice::new(
			std::rc::Rc::from(&*data),
			endian,
		))
	}

	let loader = |section: gimli::SectionId| load_section(section, &file, endian);
	// let sup_loader = |section: gimli::SectionId| get_sup_file_section_reader(section.name());
	let dwarf = gimli::Dwarf::load(loader).unwrap();
	// dwarf.load_sup(sup_loader).unwrap();

	// Iterate over all compilation units.
	let mut iter = dwarf.units();
	while let Some(header) = iter.next().unwrap() {
		// Parse the abbreviations and other information for this compilation unit.
		let unit = dwarf.unit(header).unwrap();

		// Iterate over all of this compilation unit's entries.
		let mut entries = unit.entries();
		while let Some((_, entry)) = entries.next_dfs().unwrap() {
			// If we find an entry for a function, print it.
			if entry.tag() == gimli::DW_TAG_subprogram {
				println!("Found a function: {:#?}", entry);
			}
		}
		let mut locations = dwarf
			.raw_locations(&unit, gimli::LocationListsOffset(0))
			.unwrap();
		while let Some(_loc) = locations.next().unwrap() {
			// let mut eval = expression.evaluation(unit.encoding());
			// let mut result = eval.evaluate().unwrap();
		}
	}
}
