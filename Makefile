release:
	cd bytesto4t/src-tauri
	echo "Releasing x86_64-pc-windows-gnu"
	make release-x86_64-pc-windows-gnu
	echo "Releasing x86_64-unknown-linux-gnu"
	make release-x86_64-unknown-linux-gnu

release-x86_64-pc-windows-gnu:
	cross build --release --target x86_64-pc-windows-gnu

release-x86_64-unknown-linux-gnu:
	cross build --release --target x86_64-unknown-linux-gnu

install-deps:
	echo "Installing dependencies..."
	cargo install cross
