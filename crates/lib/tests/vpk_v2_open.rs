mod common;
use common::*;

#[test]
fn open() {
	use valve_resource_tools::resource::vpk::v2::*;
	use valve_resource_tools::resource::vpk::prelude::*;

	VPKv2::open_from_path(std::path::Path::new(&get_example_path("vpk_test_dir.vpk"))).unwrap();
}

#[test]
fn open_read_entry() {
	use valve_resource_tools::resource::vpk::v2::*;
	use valve_resource_tools::resource::vpk::prelude::*;

	let vpk = open_test_vpk();
	vpk.get_entry_from_path("PreloadAndArchive");
}