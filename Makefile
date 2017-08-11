
CARGO_ENV=CARGO_INCREMENTAL=1

default:
	$(CARGO_ENV) cargo check

build:
	$(CARGO_ENV) cargo build --verbose

release:
	rustup run stable cargo build --release

# nocaptest:
# 	rm -f test_version*
# 	cargo rustc --verbose -- --test -o test_version
# 	./test_version* --nocapture

pretty:
	$(CARGO_ENV) cargo rustc -- -Z unstable-options --pretty=expanded

bt:
	$(CARGO_ENV) RUST_BACKTRACE=1 cargo test

run:
	$(CARGO_ENV) cargo test
