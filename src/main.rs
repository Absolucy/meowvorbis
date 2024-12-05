// SPDX-License-Identifier: 0BSD
#![warn(
	clippy::correctness,
	clippy::suspicious,
	clippy::complexity,
	clippy::perf,
	clippy::style
)]

pub mod args;
pub mod display;
pub mod optimize;
pub mod select;

use crate::{args::CliArgs, select::TargetedData};
use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
	io::Write,
	sync::atomic::{AtomicI64, AtomicU64, Ordering},
	time::Instant,
};

#[derive(Default)]
struct SizeStats {
	success: AtomicU64,
	failed: AtomicU64,
	diff: AtomicI64,
}

impl SizeStats {
	pub const fn const_new() -> Self {
		Self {
			success: AtomicU64::new(0),
			failed: AtomicU64::new(0),
			diff: AtomicI64::new(0),
		}
	}
}

fn main() -> Result<()> {
	color_eyre::install()?;
	let args = CliArgs::parse();

	rayon::ThreadPoolBuilder::new()
		.num_threads(args.threads)
		.build_global()
		.wrap_err("failed to build global rayon pool")?;

	let start = Instant::now();
	let files = select::get_target_files_from_args(&args);
	let total = TargetedData {
		dmis: files.dmis.len(),
		oggs: files.oggs.len(),
	};
	let stats = TargetedData::<SizeStats>::default();

	let multi = MultiProgress::new();

	// Create progress bars with custom styles
	let progress_style = ProgressStyle::with_template(
		"[{prefix:^9}] {bar:40.green/red.dim} {pos:>7}/{len:7} [{elapsed_precise}]\n{msg}",
	)?
	.progress_chars("█▓▒░");
	let finished_progress_style = ProgressStyle::with_template(
		"[{prefix:^9.green.bold}] {bar:40.green/red.dim} {pos:>7}/{len:7}\n{msg}",
	)?
	.progress_chars("█▓▒░");

	let default_msg = display::render_message(None);

	let dmi_progress = multi
		.add(ProgressBar::new(total.dmis as u64))
		.with_style(progress_style.clone())
		.with_prefix("dmi/png")
		.with_message(default_msg);

	let ogg_progress = multi
		.add(ProgressBar::new(total.oggs as u64))
		.with_style(progress_style)
		.with_prefix("ogg")
		.with_message(display::render_message(None));

	rayon::scope(|scope| {
		if args.targets.dmi {
			scope.spawn(|_| {
				let SizeStats {
					success,
					failed,
					diff,
				} = &stats.dmis;
				files.dmis.par_iter().for_each(|file| {
					match optimize::dmi(file, args.fast) {
						Ok(bytes) => {
							success.fetch_add(1, Ordering::SeqCst);
							diff.fetch_add(bytes as i64, Ordering::SeqCst);
						}
						Err(err) => {
							failed.fetch_add(1, Ordering::SeqCst);
							let mut stderr = std::io::stderr().lock();
							let _ = writeln!(
								stderr,
								"Failed to optimize {file}: {err:?}",
								file = file.display(),
								err = err
							);
						}
					}
					dmi_progress.inc(1);
					dmi_progress.set_message(display::render_message(Some(&stats.dmis)));
				});
				dmi_progress.set_style(finished_progress_style.clone());
				dmi_progress.finish();
			});
		}

		if args.targets.ogg {
			scope.spawn(|_| {
				let SizeStats {
					success,
					failed,
					diff,
				} = &stats.oggs;
				files.oggs.par_iter().for_each(|file| {
					match optimize::ogg(file) {
						Ok(bytes) => {
							success.fetch_add(1, Ordering::SeqCst);
							diff.fetch_add(bytes as i64, Ordering::SeqCst);
						}
						Err(err) => {
							failed.fetch_add(1, Ordering::SeqCst);
							let mut stderr = std::io::stderr().lock();
							let _ = writeln!(
								stderr,
								"Failed to optimize {file}: {err:?}",
								file = file.display(),
								err = err
							);
						}
					}
					ogg_progress.inc(1);
					ogg_progress.set_message(display::render_message(Some(&stats.oggs)));
				});
				ogg_progress.set_style(finished_progress_style.clone());
				ogg_progress.finish();
			});
		}
	});
	println!(
		"Optimizations took {time}",
		time = HumanDuration(start.elapsed()),
	);

	Ok(())
}
