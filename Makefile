bin/process-minion-out: process-minion-out/src/main.rs
	cargo install --path process-minion-out --root .

configure: bin/process-minion-out