// SPDX-License-Identifier: 0BSD
use atomic_write_file::AtomicWriteFile;
use color_eyre::eyre::{Result, WrapErr};
use optivorbis::{OggToOgg, Remuxer};
use std::{
	io::{BufWriter, Cursor, Seek, SeekFrom, Write},
	path::Path,
};

pub fn optimize_ogg(path: impl AsRef<Path>) -> Result<isize> {
	let path = path.as_ref();
	let original_metadata: std::fs::Metadata =
		std::fs::metadata(path).wrap_err("failed to read metadata")?;
	let original_size = original_metadata.len() as isize;
	let mut original_ogg = std::fs::read(path)
		.map(Cursor::new)
		.wrap_err("failed to read file")?;
	let mut optimized_ogg = AtomicWriteFile::open(path)
		.map(BufWriter::new)
		.wrap_err("failed to create atomic output file")?;
	match OggToOgg::new_with_defaults()
		.remux(&mut original_ogg, &mut optimized_ogg)
		.wrap_err("failed to optimize file with optivorbis")
	{
		Ok(_) => (),
		Err(err) => {
			optimized_ogg
				.into_inner()
				.wrap_err("failed to unwrap bufwriter")?
				.discard()
				.wrap_err("failed to discard temporary file")?;
			return Err(err);
		}
	}
	let optimized_size = optimized_ogg
		.seek(SeekFrom::End(0))
		.wrap_err("failed to get optimized file size")? as isize;
	optimized_ogg.flush().wrap_err("failed to flush buffer")?;
	let atomic_file = optimized_ogg
		.into_inner()
		.wrap_err("failed to unwrap atomic bufwriter")?;
	atomic_file
		.commit()
		.wrap_err("failed to commit atomic file")?;
	std::fs::set_permissions(path, original_metadata.permissions())
		.wrap_err("failed to ensure metadata is preserved")?;
	Ok(original_size - optimized_size)
}
