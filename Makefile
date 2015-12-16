CARGO := cargo rustc --
OUT_FILE_OPT := -o rusat

FLAGS = 
OPTIMISE_FLAG = -C lto -C opt-level=3

all:
	$(CARGO) $(FLAGS) $(OUT_FILE_OPT)

optimise:
	$(CARGO) $(OPTIMISE_FLAG) $(OUT_FILE_OPT)
