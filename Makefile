.PHONY: src/lex/ucd_general_category.rs
src/lex/ucd_general_category.rs:
	nix-build -A ucd-general-category -o $@
