use std::cell::RefCell;
use std::{fs::File, path::Path};
use std::io::prelude::*;
use super::*;
use crate::resource::vpk::data as common_data;

mod data;
mod directory;
mod create;
mod open;

pub use directory::Handle      as EntryHandleV2;
pub use directory::EntryReader as EntryReaderV2;
pub use create::EntryPrototype as EntryPrototypeV2;

pub trait ReadSeek : Read + Seek {}
impl ReadSeek for File {}
impl ReadSeek for std::io::Cursor<&[u8]> {}
type Reader = Rc<RefCell<Box<dyn ReadSeek>>>;

/// VPK V2 file
/// 
/// Does *not* read Titanfall VPK's desipite their major version also being 2.
/// 
/// Used in the following titles:
/// - Counter-Strike: Global Offensive
/// - Counter-Strike: Source
/// - Day of Defeat: Source
/// - Half-Life: Source
/// - Half-Life 2
/// - Half-Life 2: Deathmatch
/// - Portal
/// - Team Fortress 2
pub struct VPKv2 {
	raw_header : data::HeaderV2,

	directory : directory::Directory,
	dir : Reader,
	data : Vec<Reader>,
	archive_md5 : Vec<common_data::ArchiveMD5SectionEntry>,
	other_md5 : common_data::OtherMD5Section,
}

impl Extract for VPKv2 {
	type EntryReader = EntryReaderV2;
	
	fn get_entry_from_path(&self, path : &str) -> Result<Self::EntryReader, ErrorKind> {
		match self.directory.map.get(path) {
			Some(x) => {
				let handle = (**x).clone();
				Ok(
					Self::EntryReader::new(
						handle,
						self.dir.clone(),
						if x.entry.is_in_directory_archive() {
							None
						} else {
							Some(self.data[usize::from(x.entry.archive_index)].clone())
						}
					)
				)
			},
			None => Err(ErrorKind::DoesNotExist("".to_string())),
		}
	}
}

impl ValidateArchive for VPKv2 {
	type Checksum = common_data::ArchiveMD5SectionEntry;
	
	fn validate_archive(&self) -> Vec<&Self::Checksum> {
		let mut failed = Vec::<&Self::Checksum>::new();
		
		for entry in &self.archive_md5 {
			let mut buf = Vec::<u8>::new();
			buf.resize(usize::try_from(entry.count).unwrap(), 0);
			
			let mut data = (&self.data[usize::try_from(entry.archive_index).unwrap()]).borrow_mut();

			data.seek(std::io::SeekFrom::Start(entry.starting_offset.into())).expect("File error when reading archive for validation");
			data.read_exact(&mut buf).expect("File error when reading archive for validation");
			
			let data_digest = md5::compute(buf.as_slice());
			if data_digest.0 != entry.md5_checksum {
				failed.push(entry);
			}
		}

		failed
	}
}

impl ValidateOther for VPKv2 {
	fn validate_other(&self) -> Result<(), ErrorKind> {
		/* tree_checksum */ {
			let mut buf = vec![0u8; self.raw_header.tree_size.try_into().unwrap()];
			self.dir.borrow_mut().seek(std::io::SeekFrom::Start(self.raw_header.get_tree_start().try_into().unwrap()))?;
			self.dir.borrow_mut().read_exact(&mut buf)?;
			
			let data_digest = md5::compute(buf.as_slice());
			if data_digest.0 != self.other_md5.tree_checksum {
				return Err(ErrorKind::ValidationFailed("tree_checksum".to_string()));
			}
		}

		/* archive_md5_section_checksum */ {
			let mut buf = vec![0u8; self.raw_header.archive_md5_section_size.try_into().unwrap()];
			self.dir.borrow_mut().seek(std::io::SeekFrom::Start(self.raw_header.get_archive_md5_start().try_into().unwrap()))?;
			self.dir.borrow_mut().read_exact(&mut buf)?;
			
			let data_digest = md5::compute(buf.as_slice());
			if data_digest.0 != self.other_md5.archive_md5_section_checksum {
				return Err(ErrorKind::ValidationFailed("archive_md5_section_checksum".to_string()));
			}
		}

		Ok(())
	}
}