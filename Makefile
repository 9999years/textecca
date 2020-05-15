.PHONY: src/ucd_general_category.rs
src/ucd_general_category.rs:
	nix-build -A ucd-general-category -o $@
