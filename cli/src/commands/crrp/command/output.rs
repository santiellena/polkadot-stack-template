use std::fmt::Display;

use crate::commands::crrp::model::Backend;

pub(super) fn backend_label(backend: Backend) -> &'static str {
	if backend == Backend::Mock {
		"mock"
	} else {
		"rpc"
	}
}

pub(super) fn line(message: impl Display) {
	println!("{message}");
}

pub(super) fn kv(label: &str, value: impl Display) {
	println!("{label}: {value}");
}

pub(super) fn steps(title: &str, items: &[&str]) {
	println!("{title}");
	for (idx, item) in items.iter().enumerate() {
		println!("{}. {}", idx + 1, item);
	}
}
