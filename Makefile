DEV_TESTER=test

default:
	cargo build

devtest:
	rustc --test -o ${DEV_TESTER} src/main.rs
	./${DEV_TESTER} --nocapture

run:
	cargo test
