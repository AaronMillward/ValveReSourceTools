//! VPK Handling
//! 
//! This module aims to provide a file like interface for VPK files of all types (both v1 and v2).

// See https://developer.valvesoftware.com/wiki/VPK_File_Format for more details on the format.

pub mod prelude {
	pub use super::Open;
	pub use super::Extract;
	pub use super::ValidateArchive;
	pub use super::ValidateOther;
}

use std::{path::Path, rc::Rc};
use std::io::prelude::*;
use super::{error::ErrorKind,*};

pub use v2::VPKv2;

pub trait Open where Self : Sized {
	fn open_from_path(path : &Path) -> Result<Self, ErrorKind>;
}

pub trait Extract where Self : Sized {
	type EntryReader : std::io::Read;

	fn get_entry_from_path(&self, path : &str) -> Result<Self::EntryReader, ErrorKind>;
}

/// This trait allows for VPK formats validate their resources.
/// 
/// Usually present past V1
pub trait ValidateArchive {
	type Checksum;
	/// Returns a vector of all failed checksums.
	fn validate_archive(&self) -> Vec<&Self::Checksum>;
}

pub trait ValidateOther {
	/// Returns a vector of all failed checksums.
	fn validate_other(&self) -> Result<(), ErrorKind>;
}

mod v1;
pub mod v2;

/// Pure data types used to read file sections.
mod data {
	use serde::{Serialize, Deserialize};

	/// Magic number present at the start of all VPKs.
	pub const VPK_SIGNATURE : u32 = 0x55aa1234;

	#[derive(Clone, Default, Serialize, Deserialize)]
	pub struct ArchiveMD5SectionEntry {
		pub archive_index : u32,
		/// where to start reading bytes
		pub starting_offset : u32,
		/// how many bytes to check
		pub count : u32,
		/// expected checksum
		pub md5_checksum : [u8; 16], 
	} impl ArchiveMD5SectionEntry {
		pub const SIZE : usize = 28;
	}
	
	#[derive(Clone, Default, Serialize, Deserialize)]
	pub struct OtherMD5Section {
		pub tree_checksum : [u8; 16],
		pub archive_md5_section_checksum : [u8; 16],
		pub unknown : [u8; 16],
	}
	
	#[repr(C, packed)]
	pub struct SignatureSection {
		/// always seen as 160 (0xA0) bytes
		pub public_key_size : u32, 
		pub public_key : [u8; 160],
	
		/// always seen as 128 (0x80) bytes
		pub signature_size : u32, 
		pub signature : [u8; 128],
	}
}

mod helpers {
	use super::*;

	pub(super) fn get_base_path(path : &Path) -> String {
		let s = path.to_str().expect("could not convert path to &str");
		s[0..s.len() - 7].to_owned()
	}
	
	/// Helper function for reading arrays of a type.
	/// 
	/// # Arguments
	/// * `buf` - buffer containing the whole section to be iterated.
	pub(super) fn read_section<T: Sized>(buf : &[u8]) -> Vec<T> {
		let entry_size : usize = std::mem::size_of::<T>();
		let mut section = Vec::<T>::new();
		let mut cursor: usize = 0;
		
		loop {
			if buf.len() <= cursor {
				break;
			}
			section.push(
				crate::resource::read_from_bytes::<T>(&buf[cursor..cursor + entry_size])
			);
			cursor += entry_size;
		}
	
		section
	}
}

pub enum Version {
	V1,
	V2,
	UNKNOWN,
	NOTVPK,
}

/// Reads a vpk and tries to determine the version
pub fn determine_version(path : &std::path::Path) -> Result<Version, ErrorKind> {
	let mut dir_file = std::fs::File::open(helpers::get_base_path(path) + "dir.vpk")?;

	/* Confirm file is a supported VPK and if so, open it */

	///Common between all VPK formats
	struct CommonHeader {
		signature : u32,
		version : u32,
	}

	let mut buf = [0u8; 8];
	dir_file.seek(std::io::SeekFrom::Start(0))?;
	dir_file.read_exact(&mut buf)?;
	let h = read_from_bytes::<CommonHeader>(&buf);

	if h.signature != data::VPK_SIGNATURE {
		return Ok(Version::NOTVPK);
	}
	
	Ok(
		/* TODO: TF2 also uses version 2 but has a sub version so this needs to account for that. */
		match h.version {
			1 => Version::V1,
			2 => Version::V2, 
			_ => Version::UNKNOWN,
		}
	)
}

/// Opens a VPK file for reading.
/// 
/// # Arguments
/// * `path` - The path to the VPK.
/// 
/// # Errors
/// * `FileError` - When a problem is encountered with the file io, these are progated and so could be a wide range of io errors.
/// * `InvalidHeader` - When the header is malformed.
// pub fn open(path : &std::path::Path) -> Result<VPK<R>, Error> {
// 	let dir_file = std::fs::File::open(helpers::get_base_path(path) + "dir.vpk")?;

// 	/* Confirm file is a supported VPK and if so, open it */

// 	///Common between all VPK formats
// 	struct CommonHeader {
// 		signature : u32,
// 		version : u32,
// 	}

// 	let mut buf = [0u8; 8];
// 	dir_file.seek(std::io::SeekFrom::Start(0))?;
// 	dir_file.read_exact(&mut buf)?;
// 	let h = read_from_bytes::<CommonHeader>(&buf);

// 	if h.signature != data::VPK_SIGNATURE {
// 		return Err(Error::InvalidHeader("signature does not match"));
// 	}

// 	Ok(
// 		match h.version {
// 			1 => todo!(), //VPK::V1(v1::VPKv1::open(path)?),
// 			2 => VPK::V2(v2::VPKv2::open_from_path(path)?), /* TODO: TF2 also uses version 2 but has a sub version so this needs to account for that. */
// 			_ => return Err(Error::InvalidHeader("unsupported version")),
// 		}
// 	)
// }

#[cfg(test)]
mod tests {
	#[allow(unused_imports)]
	use super::*;

	#[test]
	fn get_base_path() {
		assert_eq!(super::helpers::get_base_path(std::path::Path::new("/example/directory/file_dir.vpk")), "/example/directory/file_");
		assert_eq!(super::helpers::get_base_path(std::path::Path::new("/example/directory/file_001.vpk")), "/example/directory/file_");
	}

	/* TODO: `open` test */
}

