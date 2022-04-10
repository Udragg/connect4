main:
	cargo build --release
	screen -S connect4 sudo ./target/release/connect4
	# sudo ./target/release/connect4
