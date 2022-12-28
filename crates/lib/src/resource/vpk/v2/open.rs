use std::convert::TryInto;

use super::*;

impl VPKv2 {
	fn open(dir : Reader, data : Vec<Reader>) -> Result<Self, ErrorKind> {
		let header : data::HeaderV2 = {
			let mut buf = [0u8; data::HeaderV2::SIZE];
			dir.borrow_mut().seek(std::io::SeekFrom::Start(0))?;
			dir.borrow_mut().read_exact(&mut buf)?;
			bincode::deserialize(&buf)?
		};

		header.is_valid()?;

		let directory = {
			let mut buf = vec![0u8; header.tree_size.try_into().unwrap()];
			dir.borrow_mut().seek(std::io::SeekFrom::Start(data::HeaderV2::SIZE.try_into().unwrap()))?;
			dir.borrow_mut().read_exact(&mut buf)?;
			directory::read_directory_tree(
				buf.as_slice(),
				header.get_tree_start().try_into().unwrap(), /* data::HeaderV2::SIZE.try_into().unwrap(), */
				header.get_data_start().try_into().unwrap()
			)?
		};

		let archive_md5 = {
			let start = data::HeaderV2::SIZE + ( header.tree_size + header.file_data_section_size) as usize;

			let mut buf = vec![0u8; header.archive_md5_section_size.try_into().unwrap()];
			dir.borrow_mut().seek(std::io::SeekFrom::Start(start.try_into().unwrap()))?;
			dir.borrow_mut().read_exact(&mut buf)?;

			helpers::read_section::<common_data::ArchiveMD5SectionEntry>(buf.as_slice())
		};

		let other_md5 = {
			let start = data::HeaderV2::SIZE + ( header.tree_size + header.file_data_section_size + header.archive_md5_section_size) as usize;

			let mut buf = vec![0u8; header.other_md5_section_size.try_into().unwrap()];
			dir.borrow_mut().seek(std::io::SeekFrom::Start(start.try_into().unwrap()))?;
			dir.borrow_mut().read_exact(&mut buf)?;
			crate::resource::read_from_bytes::<common_data::OtherMD5Section>(buf.as_slice())
		};

		Ok(VPKv2 {
			raw_header : header,
			dir,
			data,
			directory,
			archive_md5,
			other_md5,
		})
	}
}

impl Open for VPKv2 {
	fn open_from_path(path : &Path) -> Result<Self, crate::resource::error::ErrorKind> {
		/* Open files */ 
		let (dir_file, data_file) = {
			let base_path = helpers::get_base_path(path);
			
			let dir_file : Reader = Rc::new(RefCell::new(Box::new(File::open(base_path.clone() + "dir.vpk")?)));
			
			let mut data_file = Vec::<Reader>::new();
			let mut i = 0;
			loop {
				let s = base_path.clone() + &format!("{:0>3}", i).to_string() + ".vpk";
				let p = std::path::Path::new(&s);
				if !p.exists() {
					break;
				}
				data_file.push(Rc::new(RefCell::new(Box::new(File::open(p)?))));
				i += 1;
			}

			(dir_file, data_file)
		};

		VPKv2::open(dir_file, data_file)
	}
}