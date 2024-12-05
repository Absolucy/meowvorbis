// SPDX-License-Identifier: 0BSD
use color_eyre::eyre::{Result, WrapErr};
use oxipng::{InFile, Options, OutFile};
use std::path::Path;

pub fn optimize_dmi(path: impl AsRef<Path>, fast: bool) -> Result<isize> {
	let path = path.as_ref();
	let original_size = std::fs::metadata(path)
		.wrap_err("failed to read metadata")?
		.len() as isize;
	let infile = InFile::Path(path.to_path_buf());
	let outfile = OutFile::Path {
		path: None,
		preserve_attrs: true,
	};
	let opts = Options {
		fast_evaluation: true,
		optimize_alpha: true,
		..Options::from_preset(if fast { 1 } else { 4 })
	};
	oxipng::optimize(&infile, &outfile, &opts).wrap_err("failed to optimize png")?;
	let optimized_size = std::fs::metadata(path)
		.wrap_err("failed to read metadata")?
		.len() as isize;
	Ok(original_size - optimized_size)
}
