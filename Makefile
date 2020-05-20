.PHONY: src/lex/ucd_tables
src/lex/ucd_tables:
	rm -rf $@
	- mkdir $@
	nix-build -A ucd-tables
	cp -r result/* $@/
	rm result

