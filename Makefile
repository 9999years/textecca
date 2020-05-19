.PHONY: src/lex/ucd_tables
src/lex/ucd_tables:
	nix-build -A ucd-tables -o $@
