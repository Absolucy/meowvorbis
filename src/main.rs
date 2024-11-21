// SPDX-License-Identifier: 0BSD

use atomic_write_file::AtomicWriteFile;
use color_eyre::eyre::{Result, WrapErr};
use indicatif::ParallelProgressIterator;
use optivorbis::{OggToOgg, Remuxer};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
	ffi::OsStr,
	io::{BufWriter, Cursor, Seek, SeekFrom, Write},
	path::{Path, PathBuf},
	sync::atomic::{AtomicIsize, Ordering},
};
use walkdir::WalkDir;

pub fn optimize(path: impl AsRef<Path>) -> Result<isize> {
	let path = path.as_ref();
	let mut original_ogg = std::fs::read(path)
		.map(Cursor::new)
		.wrap_err_with(|| format!("Failed to read {}", path.display()))?;
	let mut optimized_ogg = AtomicWriteFile::open(path)
		.map(BufWriter::new)
		.wrap_err_with(|| format!("Failed to create file {}", path.display()))?;
	match OggToOgg::new_with_defaults()
		.remux(&mut original_ogg, &mut optimized_ogg)
		.wrap_err_with(|| format!("Failed to optimize {}", path.display()))
	{
		Ok(_) => (),
		Err(err) => {
			let _ = optimized_ogg.into_inner()?.discard();
			return Err(err);
		}
	}
	let original_size = original_ogg.seek(SeekFrom::End(0))? as isize;
	let optimized_size = optimized_ogg.seek(SeekFrom::End(0))? as isize;
	optimized_ogg.flush()?;
	let atomic_file = optimized_ogg.into_inner()?;
	atomic_file.commit()?;
	Ok(original_size - optimized_size)
}

fn main() {
	let mut ogg_files = Vec::<PathBuf>::new();
	for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
		let path = entry.path();
		if !path.is_file() {
			continue;
		}
		match path.extension().and_then(OsStr::to_str) {
			Some(ext) if ext.eq_ignore_ascii_case("ogg") => ogg_files.push(path.to_path_buf()),
			_ => continue,
		}
	}
	let total_oggs = ogg_files.len() as u64;
	println!("Optimizing {total_oggs} files...");
	let saved_bytes = AtomicIsize::new(0);
	let min_diff = AtomicIsize::new(0);
	let max_diff = AtomicIsize::new(0);
	ogg_files
		.par_iter()
		.progress_count(total_oggs)
		.for_each(|file| match optimize(file) {
			Ok(bytes) => {
				saved_bytes.fetch_add(bytes, Ordering::Relaxed);
				min_diff.fetch_min(bytes, Ordering::Relaxed);
				max_diff.fetch_max(bytes, Ordering::Relaxed);
			}
			Err(err) => {
				let mut stderr = std::io::stderr().lock();
				let _ = writeln!(stderr, "Failed to optimize {}: {:?}", file.display(), err);
			}
		});
	let saved_bytes = saved_bytes.load(Ordering::Relaxed);
	let min_diff = min_diff.load(Ordering::Relaxed);
	let max_diff = max_diff.load(Ordering::Relaxed);
	println!("Saved {saved_bytes} bytes (min {min_diff}, max {max_diff})");
}
