use std::{convert::TryInto, convert::TryFrom, collections::HashMap, io::SeekFrom};
use std::io::prelude::*;

/// The size at which a new data file is created
const DATA_SPLIT_BYTE : u64 = 100 * 1000 * 1000; /* 100MB */

use super::*;

/// An incomplete entry for the user to apply settings to.
pub struct EntryPrototype {
	///How much of this entry is stored in preload data
	preload_size : u16,
	///If this entry's data is stored in an indexed archive or the embedded archive
	store_in_directory : bool,

	extension : String,
	filename : String,
	path : String,

	data : Box<dyn ReadSeek>,

	raw : data::DirectoryEntryData,
}

impl EntryPrototype {
	pub fn new(
		store_in_directory : bool,
		preload_size : u16,
		path : String,
		filename : String,
		extension : String,
		data : Box<dyn ReadSeek>,
	) -> Self {
		EntryPrototype {
			preload_size,
			store_in_directory,
			extension,
			filename,
			path,
			data,
			raw: data::DirectoryEntryData::default(),
		}
	}
}

impl VPKv2 {
	/// Creates new VPK dir and data files.
	/// 
	/// # Arguments
	/// * `directory_path` - The directory to create the files in.
	/// * `filename` - Base name for the VPKs, e.g. `hl2_misc` -> `hl2_misc_dir.vpk`
	/// * `entries` - A vector containing all of the entries to be packed.
	pub fn create(directory_path : &Path, filename : &str, entries : &mut [EntryPrototype]) -> Result<(), ErrorKind> {
		let mut file_dir = {
			let mut path = directory_path.to_path_buf();
			path.push("./".to_owned() + filename + "_dir.vpk");
			File::create(path)?
		};
		
		/* Set up as much of the entry as possible */
		for e in entries.iter_mut() {
			let data_len = e.data.seek(SeekFrom::End(0))?;
			e.raw.preload_bytes_size = e.preload_size;
			e.raw.data_length = u32::try_from(data_len).expect("Data length is greater than a u32") - u32::from(e.preload_size);
			e.raw.terminator = data::DirectoryEntryData::TERMINATOR;
		}
		
		/* TODO: Revise sizing concerns
		 * u32 adresses allows up to 4.2GB
		 * The data file division occurs every `DATA_SPLIT_BYTE` but what happens if a single 4.2GB file is inserted?
		 */

		let mut current_file_index : i32 = -1;
		let mut file_data_option : Option<File> = None;
		let mut embeded_data = Vec::<u8>::new();
		let mut archivemd5 = Vec::<common_data::ArchiveMD5SectionEntry>::new();
		
		/* Loop to write the actual entry data to the appropriate place */
		for e in entries.iter_mut() {
			if file_data_option.is_none() {
				current_file_index += 1;
				let mut path = directory_path.to_path_buf();
				path.push("./".to_string() + filename + "_" + &format!("{:0>3}", current_file_index).to_string() + ".vpk");
				file_data_option = Some(File::create(path)?)
			}
			let mut file_data = file_data_option.as_ref().unwrap(); /* Okay because of the create above */

			if e.raw.is_preload_only() {
				continue; /* Preload data will be written later when the directory is written */
			}

			e.data.seek(SeekFrom::Start(e.preload_size.into())).expect("Entry data reader can't be seeked");
			if e.store_in_directory {
				e.raw.archive_index = data::DirectoryEntryData::DATA_IN_DIRECTORY_ARCHIVE_INDEX;
				e.raw.data_offset = embeded_data.len().try_into().expect("Embeded data is larger than maximum address");

				let mut buf = vec![0u8; e.raw.data_length.try_into().unwrap()];/*  Vec::<u8>::with_capacity(e.raw.data_length.try_into().unwrap()); */
				e.data.read_exact(&mut buf)?;

				embeded_data.append(&mut buf);
			} else { /* Write to data file */
				e.raw.archive_index = current_file_index.try_into().expect("File index is too large");
				e.raw.data_offset = file_data.seek(SeekFrom::Current(0))?.try_into().expect("Data in file is larger than maximum address");
				
				let mut buf = vec![0u8; e.raw.data_length.try_into().unwrap()];
				e.data.read_exact(&mut buf)?;

				/* Calculate MD5 Entry */ {
					archivemd5.push(
						common_data::ArchiveMD5SectionEntry {
							archive_index: current_file_index.try_into().unwrap(),
							starting_offset: e.raw.data_offset,
							count: e.raw.data_length,
							md5_checksum: md5::compute(&buf).0,
						}
					)
				}

				file_data.write_all(&buf)?;

				if e.data.seek(SeekFrom::Current(0))? > DATA_SPLIT_BYTE { /* File is over the size limit */
					file_data.flush()?;
					file_data_option = None;
				}
			}
		}
	
		let dir_data = { /* Create entry directory */
			let mut maps = HashMap::<String /* Extension */, HashMap<String /* Path */, HashMap<String /* Filename */, &mut EntryPrototype>>>::new();

			/* Populate all hash maps with entries */
			for e in entries.iter_mut() {
				let filenames = maps
					.entry(e.extension.clone()).or_default() /* Get paths */
					.entry(e.path.clone()).or_default(); /* Get filenames */

				if filenames.contains_key(&e.filename) {
					return Err(ErrorKind::AlreadyExists(format!("File at {}/{}.{} already exists", e.path, e.filename, e.extension)))
				} else {
					filenames.insert(e.filename.clone(), e);
				}
			}

			let mut data = Vec::<u8>::new();

			fn write_null_terminated_string(buf : &mut Vec<u8>, s : &String) -> Result<(), ErrorKind> {
				if !s.is_ascii() { return Err(ErrorKind::MalformedData(format!("String \"{}\" is not ASCII", s)))}
				buf.write_all(s.as_bytes())?; /* FIXME: Might work? might need a cusor */
				buf.write_all(&[0])?; /* Null terminator */
				Ok(())
			}

			/* Iterate hash maps to write directory tree */
			for (ext, paths_map) in maps {
				write_null_terminated_string(&mut data, &ext)?;
				for (path, filenames_map) in paths_map {
					write_null_terminated_string(&mut data, &path)?;
					for (filename, e) in filenames_map {
						write_null_terminated_string(&mut data, &filename)?;
						
						/* Write entry */ {
							let e_bytes = bincode::serialize(&e.raw).expect("Entry can't be serialized");
							data.write_all(&e_bytes)?;
						}

						/* Write preload data */ {
							e.data.seek(SeekFrom::Start(0))?;
							let mut buf = vec![0u8; e.preload_size.into()];
							e.data.read_exact(&mut buf)?;
							data.write_all(&buf)?;
						}
					}
					data.write_all(&[0])?;
				}
				data.write_all(&[0])?;
			}
			data.write_all(&[0])?;

			data
		};

		/* Write dir file */ {
			/* header */ {
				let head = data::HeaderV2 {
					tree_size : u32::try_from(dir_data.len()).expect("tree size is greater than a u32"),
					file_data_section_size : u32::try_from(embeded_data.len()).expect("file data size is greater than a u32"),
					archive_md5_section_size : (common_data::ArchiveMD5SectionEntry::SIZE * archivemd5.len()).try_into().unwrap(),
					other_md5_section_size : 48,
					..Default::default()
				};
				let bytes = bincode::serialize(&head)?;
				file_dir.write_all(&bytes)?;
			}

			file_dir.write_all(&dir_data)?;
			file_dir.write_all(&embeded_data)?;

			let archive_md5_checksum = { /* ArchiveMD5 */
				let mut buf = Vec::<u8>::new();
				for e in archivemd5 {
					let mut bytes = bincode::serialize(&e)?;
					buf.append(&mut bytes);
				}
				file_dir.write_all(&buf)?;
				md5::compute(buf)
			};

			/* OtherMD5 */ {
				let other = common_data::OtherMD5Section {
					tree_checksum : md5::compute(&dir_data).0,
					archive_md5_section_checksum : *archive_md5_checksum,
					..Default::default()
				};

				let bytes = bincode::serialize(&other)?;
				file_dir.write_all(&bytes)?;
			}

			file_dir.flush()?;
		}

		Ok(())
	}

	/// Rewrites the `_dir` VPK appending new entries without rewriting the data files.
	pub fn append_entries(&self) {
		todo!()
	}
}

/* TODO: Tests */
/* Single Large File (>4.2GB) */
