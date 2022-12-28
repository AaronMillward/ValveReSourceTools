use std::{fs::File, path::PathBuf};

use valve_resource_tools::resource::vpk::VPKv2;
use valve_resource_tools::resource::vpk::prelude::*;

pub fn get_test_data_path() -> String {
	env!("CARGO_MANIFEST_DIR").to_owned() + "/test-data/"
}

pub fn get_example_path(path : &str) -> PathBuf {
	PathBuf::from(get_test_data_path() + path)
}

pub fn get_example_data(path : &str) -> File {
	let p = get_example_path(path);
	File::open(&p).expect(&format!("Can't load example data {}", &p.display()))
}

pub fn open_test_vpk() -> VPKv2 {
	VPKv2::open_from_path(std::path::Path::new(&get_example_path("vpk_test_dir.vpk"))).unwrap()
}

pub fn get_tmp_dir() -> PathBuf {
	let mut tmp_dir = std::env::temp_dir();
	tmp_dir.push("./vrst-test".to_owned() + &format!("-{:x}", rand::random::<u32>()));
	std::fs::create_dir(&tmp_dir).expect(&format!("Couldn't create tmp directory at {}", &tmp_dir.display()));
	return tmp_dir;
}

pub fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
	let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
	matching == a.len() && matching == b.len()
}