DEV_TESTER=test

default:
	cargo build

devtest:
	rustc --test -o ${DEV_TESTER} src/main.rs
	./${DEV_TESTER} --nocapture

pretty:
	cargo rustc -- -Z unstable-options --pretty=expanded

test:
	RUST_BACKTRACE=1 cargo test

run:
	cargo test
