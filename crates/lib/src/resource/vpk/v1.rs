// use std::{fs::File, path::Path};
// use super::{EntryDirectorySection, VPKOpen, };

// /// Pure data types used to read file sections.
// mod data {
// 	#[repr(C, packed)]
// 	pub struct HeaderV1 {
// 		pub signature : u32, /* Should remain const VPK_SIGNATURE*/
// 		pub version : u32,   /* Should remain const 1 */
// 		pub tree_size : u32, /* The size, in bytes, of the directory tree */
// 	}
// 	pub const HEADER_SIZE : usize = std::mem::size_of::<HeaderV1>();
// }



// ///Used in the following titles
// /// - Alien Swarm
// /// - Dota 2
// /// - Left 4 Dead
// /// - Left 4 Dead 2
// /// - Portal 2
// /// - Source Filmmaker

// pub struct VPKv1 {
// 	directory : super::directory::Directory<>,
// 	dir_file : File,
// 	data_file : Vec<File>,
// }

// impl VPKOpen for VPKv1 {
// 	fn open(path : &Path) -> Result<Self, crate::resource::error::Error> {
// 		/* Open files */ 
// 		let (dir_file, data_file) = {
// 			let base_path = super::helpers::get_base_path(path);
			
// 			let dir_file = File::open(base_path.clone() + "dir.vpk")?;
			
// 			let mut data_file = Vec::<File>::new();
// 			let mut i = 0;
// 			loop {
// 				let s = base_path.clone() + &format!("{:0>3}", i).to_string() + ".vpk";
// 				let p = std::path::Path::new(&s);
// 				if !p.exists() {
// 					break;
// 				}
// 				data_file.push(File::open(p)?);
// 				i += 1;
// 			}

// 			(dir_file, data_file)
// 		};


// 		let header = {
// 			let mut buf = [0u8; data::HEADER_SIZE];
// 			dir_file.read_exact_at(&mut buf, 0)?;
// 			crate::resource::read_from_bytes::<data::HeaderV1>(buf.as_slice())
// 		};

// 		let directory = {
// 			let mut buf = Vec::<u8>::with_capacity(header.tree_size as usize);
// 			buf.resize(header.tree_size as usize, 0);
// 			dir_file.read_exact_at(&mut buf, data::HEADER_SIZE.try_into().unwrap())?;
// 			super::helpers::read_directory_tree(buf.as_slice(), data::HEADER_SIZE.try_into().unwrap())
// 		};

// 		return Ok(VPKv1 {
// 			dir_file,
// 			data_file,
// 			directory,
// 		})
// 	}
// }

// impl EntryDirectorySection for VPKv1 {
// 	type EntryHandle = DirectoryEntry;

// 	fn read_entry_data(&self, entry : &Self::EntryHandle) -> Result<Vec<u8>, crate::resource::error::Error> {
// 		let mut full_data = Vec::<u8>::new();
// 		full_data.resize(entry.entry_data.preload_bytes_size as usize + entry.entry_data.data_length as usize, 0);
// 		let (preload_data, archive_data) = full_data.split_at_mut(entry.entry_data.preload_bytes_size as usize);
		
// 		/* Read preload data */
// 		if entry.entry_data.preload_bytes_size != 0 {
// 			(&self.dir_file).read_exact_at(preload_data, entry.preload_data_position.into())?;
// 		}

// 		if entry.entry_data.data_length == 0 {
// 			return Ok(full_data);
// 		}

// 		if entry.entry_data.archive_index == super::data::DIRECTORYENTRY_DATA_IN_DIRECTORY_ARCHIVE_INDEX {
// 			todo!()
// 		} else {
// 			(&self.data_file[entry.entry_data.archive_index as usize])
// 				.read_exact_at(archive_data, entry.entry_data.data_offset.into())?;
// 		}

// 		return Ok(full_data);
// 	}

// 	fn get_entries<'a>(&'a self) -> &'a Vec<std::rc::Rc<Self::EntryHandle>> {
// 		return &self.directory.entries;
// 	}

// 	fn get_entry_from_path(&self, path : String) -> Option<&Self::EntryHandle> {
// 		match self.directory.map.get(&path) {
// 			Some(x) => Some(x.as_ref()),
// 			None => None,
// 		}
// 	}
// }