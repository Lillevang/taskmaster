# Makefile for TaskMaster

.PHONY: all build test clean lint format

all: build

build:
	cargo build

test:
	cargo test

run:
	cargo run

clean:
	cargo clean

lint:
	cargo clippy -- -D warnings

format:
	cargo fmt

check: lint format test
	echo "All checks passed"
