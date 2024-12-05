// SPDX-License-Identifier: 0BSD
use clap::{Args, Parser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
	#[command(flatten)]
	pub targets: Targets,

	/// Number of threads.
	/// Defaults to number of logical cores - 1.
	#[arg(short, long, default_value_t = default_threads())]
	pub threads: usize,

	/// Optimize dmi and png files faster, albeit with possibly larger file
	/// sizes.
	#[arg(short, long)]
	pub fast: bool,

	/// Files or folders to compress.
	/// Folders will be searched recursively.
	pub files: Vec<PathBuf>,
}

#[derive(Args)]
#[group(required = true, multiple = true)]
pub struct Targets {
	/// Optimize dmi and png files (using oxipng)
	#[arg(short, long)]
	pub dmi: bool,

	/// Optimize ogg files (using optivorbis)
	#[arg(short, long)]
	pub ogg: bool,
}

fn default_threads() -> usize {
	num_cpus::get().saturating_sub(1).max(1)
}
