DEV_TESTER=test

CARGO_ENV=CARGO_INCREMENTAL=1

default:
	$(CARGO_ENV) cargo check

build:
	$(CARGO_ENV) cargo build --verbose

release:
	rustup run stable cargo build --release

devtest:
	rustc --test -o ${DEV_TESTER} src/main.rs
	./${DEV_TESTER} --nocapture

pretty:
	$(CARGO_ENV) cargo rustc -- -Z unstable-options --pretty=expanded

bt:
	$(CARGO_ENV) RUST_BACKTRACE=1 cargo test

run:
	$(CARGO_ENV) cargo test
