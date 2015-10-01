all: star-probe star-collect

clean:
	cargo clean

test:
	cargo test

star-probe:
	cargo build --bin star-probe

star-collect:
	cargo build --bin star-collect
