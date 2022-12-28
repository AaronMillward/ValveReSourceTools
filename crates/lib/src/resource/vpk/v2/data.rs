//! Pure data types used to read file sections

use serde::{Serialize, Deserialize};
use crate::resource::vpk::data::VPK_SIGNATURE;
use super::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct HeaderV2 {
	/// Should be `VPK_SIGNATURE`
	pub signature : u32,
	/// Should be 2
	pub version : u32,

	/// The size, in bytes, of the directory tree
	pub tree_size : u32,
	/// How many bytes of file content are stored in this VPK file (0 in CSGO)
	pub file_data_section_size : u32,
	/// The size, in bytes, of the section containing MD5 checksums for external archive content
	pub archive_md5_section_size : u32,
	/// The size, in bytes, of the section containing MD5 checksums for content in this file (should always be 48)
	pub other_md5_section_size : u32,
	/// The size, in bytes, of the section containing the public key and signature. This is either 0 (CSGO & The Ship) or 296 (HL2, HL2:DM, HL2:EP1, HL2:EP2, HL2:LC, TF2, DOD:S & CS:S)
	pub signature_section_size : u32,
}

impl HeaderV2 {
	pub const SIZE : usize = std::mem::size_of::<HeaderV2>();

	pub(super) const fn get_tree_start(&self)  -> usize { Self::SIZE }
	pub(super) fn get_data_start(&self)        -> usize { self.get_tree_start()        + self.tree_size as usize }
	pub(super) fn get_archive_md5_start(&self) -> usize { self.get_data_start()        + self.file_data_section_size as usize }
	pub(super) fn get_other_md5_start(&self)   -> usize { self.get_archive_md5_start() + self.archive_md5_section_size as usize }
	pub(super) fn get_signature_start(&self)   -> usize { self.get_other_md5_start()   + self.other_md5_section_size as usize }

	pub(super) fn is_valid(&self) -> Result<(), ErrorKind> {
		if self.signature != VPK_SIGNATURE { return Err(ErrorKind::MalformedData("Signature".to_string())); }
		if self.version != 2 { return Err(ErrorKind::MalformedData("Major Version".to_string())); }
		if self.other_md5_section_size != 48 { return Err(ErrorKind::MalformedData("OtherMD5".to_string())); }
		if self.signature_section_size != 296 && self.signature_section_size != 0  { return Err(ErrorKind::MalformedData("Signature".to_string())); }
		Ok(())
	}
}

impl Default for HeaderV2 {
	fn default() -> Self {
		Self {
			signature: VPK_SIGNATURE,
			version: 2,
			tree_size: Default::default(),
			file_data_section_size: Default::default(),
			archive_md5_section_size: Default::default(),
			other_md5_section_size: Default::default(),
			signature_section_size: Default::default(),
		}
	}
}

/// Represents the data structure following each filename in the VPK directory
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DirectoryEntryData {
	/// A 32bit CRC of the file's data.
	pub crc : u32,
	/// The number of bytes contained in the index file.
	pub preload_bytes_size : u16,
	
	/// A zero based index of the archive this file's data is contained in.
	/// If 0x7fff, the data follows the directory.
	pub archive_index : u16,

	/// If ArchiveIndex is 0x7fff, the offset of the file data relative to the end of the directory (see the header for more details).
	/// Otherwise, the offset of the data from the start of the specified archive.
	pub data_offset : u32,

	/// If zero, the entire file is stored in the preload data.
	/// Otherwise, the number of bytes stored starting at EntryOffset.
	pub data_length : u32,

	/// Should always be 0xffff
	pub terminator : u16,
}

impl DirectoryEntryData {
	pub const SIZE : usize = 18;
	pub const TERMINATOR : u16 = 0xffff;
	pub const DATA_IN_DIRECTORY_ARCHIVE_INDEX : u16 = 0x7fff;

	pub(super) fn total_data_size(&self) -> u32 {
		self.data_length + u32::from(self.preload_bytes_size)
	}

	pub(super) fn is_valid(&self) -> Result<(), ErrorKind> {
		if self.terminator != Self::TERMINATOR { return Err(ErrorKind::MalformedData("Directory Entry Terminator".to_string())) }
		Ok(())
	}

	pub(super) fn is_in_directory_archive(&self) -> bool {
		self.archive_index == Self::DATA_IN_DIRECTORY_ARCHIVE_INDEX
	}

	pub(super) fn is_preload_only(&self) -> bool {
		self.data_length == 0
	}

	pub(super) fn has_preload(&self) -> bool {
		self.preload_bytes_size != 0
	}
}
