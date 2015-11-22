all:
	$(MAKE) optimise

optimise:
	cargo rustc -- -C lto -C opt-level=3 -o rusat
