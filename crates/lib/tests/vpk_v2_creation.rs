use std::io::Read;

mod common;
use common::*;

#[test]
/// Creates a vpk with every possible entry type then opens the vpk to ensure it reads properly.
fn create_with_every_entry_permutation() {
	use valve_resource_tools::resource::vpk::v2::*;
	use valve_resource_tools::resource::vpk::prelude::*;

	const DIR_NAME : &str = "testing-folder";

	/* Contains all usefully distinct permutations of EntryPrototypeV2 */
	let mut ents = vec![
		/* Preload only */          EntryPrototypeV2::new(false, 26, DIR_NAME.to_string(), "PreloadOnly".to_string(),        "txt".to_string(), Box::new(get_example_data("PreloadOnly.txt"))),
		/* Data Archive only */     EntryPrototypeV2::new(false,  0, DIR_NAME.to_string(), "ArchiveOnly".to_string(),        "txt".to_string(), Box::new(get_example_data("ArchiveOnly.txt"))),
		/* Embedded Archive Only */ EntryPrototypeV2::new(true,   0, DIR_NAME.to_string(), "EmbededArchiveOnly".to_string(), "txt".to_string(), Box::new(get_example_data("EmbededArchiveOnly.txt"))),
		/* Preload and Archive */   EntryPrototypeV2::new(false, 21, DIR_NAME.to_string(), "PreloadAndArchive".to_string(),  "txt".to_string(), Box::new(get_example_data("PreloadAndArchive.txt"))),
	];

	let tmp_dir = get_tmp_dir();
	eprintln!("Creating files in {}", tmp_dir.to_string_lossy());
	VPKv2::create(&tmp_dir, "vpk_test", &mut ents).expect("Create failed");

	/* Check the file opens correctly */ {
		let mut path = tmp_dir.to_owned();
		path.push("./vpk_test_dir.vpk");
		let vpk = VPKv2::open_from_path(&path).expect("Couldn't open VPK");
		if !vpk.validate_archive().is_empty() { panic!("Validation failed")}
		vpk.validate_other().expect("Other validation section failed");

		/* Read entries and check they are correct */
		for f in &[ /* Should match `ents` above */
			"PreloadOnly.txt",
			"PreloadAndArchive.txt",
			"ArchiveOnly.txt",
			"EmbededArchiveOnly.txt",
		] {
			let path = &(DIR_NAME.to_owned() + "/" + f);
			let mut buf = Vec::<u8>::new();
			let mut res = Vec::<u8>::new();
			vpk.get_entry_from_path(&path).expect(&format!("{} does not exist", path))
				.read_to_end(&mut buf).expect(&format!("Couldn't read whole entry at \"{}\"", path));
			get_example_data(f).read_to_end(&mut res).expect("Couldn't read example data");
			if !do_vecs_match(&buf, &res) { panic!("File entry \"{}\" does not match original content", path) }
		}
	}
}

/* Tests TODO:
 * Single large file (>maximum split)
 * Single large file (>data_offset)
 * 0 Length File
 */