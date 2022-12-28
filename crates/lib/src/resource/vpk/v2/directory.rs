use std::convert::TryInto;
use std::rc::Rc;

use crate::resource::error::ErrorKind;
use super::data as V2Data;
use super::Reader;

#[derive(Default, Clone)]
pub(super) struct Directory {
	pub(super) entries : Vec<std::rc::Rc<Handle>>,
	pub(super) map : std::collections::HashMap<String, std::rc::Rc<Handle>>,
}

/// Reads the standard directory tree seen in most VPK formats.
/// 
/// # Arguments
/// * `input` - The bytes begining at the tree's offset and ending at tree offset + length.
/// * `offset` - Used to determine `preload_data_position` relative to the start of the VPK.
/// * `directory_archive_offset` - Used to determine `preload_data_position` relative to the start of the VPK.
pub(super) fn read_directory_tree(input: &[u8], offset : u64, directory_archive_offset : u64) -> Result<Directory, ErrorKind> {
	fn read_string(input: &[u8], cursor : &mut u64) -> String {
		let mut string = "".to_string();
		loop {
			let c = input[*cursor as usize];
			*cursor += 1;
			if c == 0 { break; }
			string.push(c.into());
		}
		string
	}
	
	let mut directory = Directory::default();
	let mut cursor: u64 = 0;
	loop {
		let extension = read_string(input, &mut cursor);
		if extension.is_empty() { break; }
		loop {
			let path = read_string(input, &mut cursor);
			if path.is_empty() { break; }
			loop {
				let filename = read_string(input, &mut cursor);
				if filename.is_empty() { break; }
				
				let buf = &input[cursor as usize..(cursor as usize + V2Data::DirectoryEntryData::SIZE)];
				let entry = bincode::deserialize::<V2Data::DirectoryEntryData>(buf)?;

				entry.is_valid()?;

				cursor += u64::try_from(V2Data::DirectoryEntryData::SIZE).unwrap();

				let handle = Rc::new(Handle {
					entry,
					preload_data_position: offset + cursor,
					directory_archive_data_start_position: directory_archive_offset,
				});
				
				cursor += u64::from(handle.entry.preload_bytes_size);
				
				directory.entries.push(handle.clone());
				directory.map.insert(path.clone() + "/" + &filename + "." + &extension, handle.clone());
			}
		}
	}

	Ok(directory)
}

#[derive(Clone)]
pub struct Handle {
	pub(super) entry : V2Data::DirectoryEntryData,
	pub(super) preload_data_position : u64,
	/// Where the embedded archive data is located in the VPK.
	pub(super) directory_archive_data_start_position : u64,
}

pub struct EntryReader {
	handle : Handle,
	/// The VPK directory file.
	dir : Reader,
	/// The data file in which the archive data resides. might not exist if the data is embedded in `dir`
	data : Option<Reader>,
	cursor : u32,
}

impl EntryReader {
	/// # Panics
	/// If `data` is `None` when `handle.entry.is_in_directory_archive()` is `false`
	pub(super) fn new(handle : Handle, dir : Reader, data : Option<Reader>) -> EntryReader {
		if handle.entry.is_in_directory_archive() == false && data.is_none() { panic!("Entry data is external but no external reader given") }
		EntryReader {
			dir,
			data,
			handle,
			cursor : 0,
		}
	}
}

impl std::io::Seek for EntryReader {
	fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
		match pos {
			std::io::SeekFrom::Start(c) => {
				let pos = std::cmp::min(self.handle.entry.total_data_size(), c.try_into().unwrap_or(u32::MAX));
				self.cursor = pos.into();
			},
			std::io::SeekFrom::End(c) => {
				let c : u32 = c.clamp(0, self.handle.entry.total_data_size().into()).try_into().unwrap();
				let pos : u32 = self.handle.entry.total_data_size() - c;
				self.cursor = pos;
			},
			std::io::SeekFrom::Current(c) => {
				let pos = (i64::from(self.cursor) + c).clamp(0,self.handle.entry.total_data_size().into());
				self.cursor = pos.try_into().unwrap();
			},
		}
		return Ok(self.cursor.into());
	}
}

impl std::io::Read for EntryReader {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		use std::io::Seek;

		let mut bytes_read : usize = 0;

		if self.cursor >= self.handle.entry.total_data_size() { return Ok(0); } /* EOF */

		let buf_data = if self.cursor < self.handle.entry.preload_bytes_size.into() {
			/* Seek dir to preload data */ {
				let pos = self.handle.preload_data_position + u64::from(self.cursor);
				self.dir.borrow_mut().seek(std::io::SeekFrom::Start(pos))?;
			}

			let remaining_preload_bytes : usize = (u32::from(self.handle.entry.preload_bytes_size) - self.cursor).try_into().unwrap();
			let (preload, data) = buf.split_at_mut(std::cmp::min(remaining_preload_bytes, buf.len()));
			self.dir.borrow_mut().read_exact(preload)?;
			bytes_read += preload.len();
			self.seek(std::io::SeekFrom::Current(bytes_read.try_into().unwrap()))?;

			data
		} else {
			buf
		};

		if buf_data.is_empty() {
			return Ok(bytes_read);
		}

		/* We don't need to check the cursor position past here because we've eliminated other conditions */

		{
			/* Seek real cursor and get the reader */
			let reader = if self.handle.entry.is_in_directory_archive() {
				let cursor_offset_into_archive_entry_data = u64::from(self.cursor - u32::from(self.handle.entry.preload_bytes_size));
				let pos = self.handle.directory_archive_data_start_position + u64::from(self.handle.entry.data_offset) + cursor_offset_into_archive_entry_data;
				self.dir.borrow_mut().seek(std::io::SeekFrom::Start(pos))?;
				&self.dir
			} else {
				let data_file = self.data.as_ref().unwrap();
				let cursor_offset_into_archive_entry_data = u64::from(self.cursor - u32::from(self.handle.entry.preload_bytes_size));
				let pos = u64::from(self.handle.entry.data_offset) + cursor_offset_into_archive_entry_data;
				data_file.borrow_mut().seek(std::io::SeekFrom::Start(pos))?;
				data_file
			};

			let remaining_data : usize = u64::from(
				self.handle.entry.data_length - (self.cursor - u32::from(self.handle.entry.preload_bytes_size))
			).try_into().unwrap();
			
			let (data, _rest) = buf_data.split_at_mut(std::cmp::min(remaining_data, buf_data.len()));
			reader.borrow_mut().read_exact(data)?;
			bytes_read += data.len();
			self.seek(std::io::SeekFrom::Current(data.len().try_into().unwrap()))?;
		}

		return Ok(bytes_read);
	}
}