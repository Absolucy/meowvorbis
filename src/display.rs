// SPDX-License-Identifier: 0BSD
use crate::SizeStats;
use console::{style, StyledObject};
use indicatif::HumanCount;
use number_prefix::NumberPrefix;
use std::{fmt, sync::atomic::Ordering};

// literally just indicatif's HumanBytes but it's a signed integer.
pub struct HumanBytes(pub i64);

impl fmt::Display for HumanBytes {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match NumberPrefix::binary(self.0 as f64) {
			NumberPrefix::Standalone(number) => write!(f, "{number:.0} B"),
			NumberPrefix::Prefixed(prefix, number) => write!(f, "{number:.2} {prefix}B"),
		}
	}
}

fn good_or_bad(num: i64) -> StyledObject<String> {
	let padded = format!("{:>7}", HumanBytes(num));
	match num {
		-1024..=1024 => style(padded).dim(),
		..-1024 => style(padded).red(),
		1025.. => style(padded).green(),
	}
}

pub(super) fn render_message(stats: Option<&SizeStats>) -> String {
	static DEFAULT: SizeStats = SizeStats::const_new();
	let stats = stats.unwrap_or(&DEFAULT);
	let (success, failed, diff) = (
		stats.success.load(Ordering::SeqCst),
		stats.failed.load(Ordering::SeqCst),
		stats.diff.load(Ordering::SeqCst),
	);
	let average = diff.checked_div(success as i64).unwrap_or(0);

	let success_str = style(format!("{:>7}", HumanCount(success))).green().bold();
	let failed_str = style(format!("{:>3}", HumanCount(failed))).red().bold();

	format!(
		"├─ Optimized: {success:>12}\n├─ Failed:    {failed:>12}\n├─ Saved:     {diff:>12}\n└─ \
		 Average:   {average:>12}",
		success = success_str,
		failed = failed_str,
		diff = good_or_bad(diff),
		average = good_or_bad(average),
	)
}
