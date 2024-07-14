test *args='':
	cargo test {{args}} --target x86_64-unknown-linux-gnu --target i686-unknown-linux-gnu --target aarch64-unknown-linux-gnu
