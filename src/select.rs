// SPDX-License-Identifier: 0BSD
use crate::args::{CliArgs, Targets};
use std::{
	collections::HashSet,
	ffi::OsStr,
	path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Copy, Clone, Default)]
pub struct TargetedData<T> {
	pub oggs: T,
	pub dmis: T, // also includes pngs to be fair
}

fn add_file(path: &Path, targets: &Targets, files: &mut TargetedData<HashSet<PathBuf>>) {
	let path = path.to_path_buf();
	match path.extension().and_then(OsStr::to_str) {
		Some(ext)
			if targets.dmi
				&& (ext.eq_ignore_ascii_case("dmi") || ext.eq_ignore_ascii_case("png")) =>
		{
			files.dmis.insert(path);
		}
		Some(ext) if targets.ogg && ext.eq_ignore_ascii_case("ogg") => {
			files.oggs.insert(path);
		}
		_ => (),
	}
}

fn add_dir(path: &Path, targets: &Targets, files: &mut TargetedData<HashSet<PathBuf>>) {
	for entry in WalkDir::new(path)
		.into_iter()
		.filter_map(|e| e.ok())
		.filter(|entry| entry.path().is_file())
	{
		add_file(entry.path(), targets, files);
	}
}

pub fn get_target_files_from_args(args: &CliArgs) -> TargetedData<Vec<PathBuf>> {
	let mut target_files = TargetedData::default();
	for file in &args.files {
		if file.is_dir() {
			add_dir(file, &args.targets, &mut target_files);
		} else {
			add_file(file, &args.targets, &mut target_files);
		}
	}
	let mut oggs = target_files.oggs.into_iter().collect::<Vec<_>>();
	let mut dmis = target_files.dmis.into_iter().collect::<Vec<_>>();
	oggs.sort();
	dmis.sort();
	TargetedData { oggs, dmis }
}
