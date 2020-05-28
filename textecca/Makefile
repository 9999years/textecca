.PHONY: parse/ucd_tables
parse/ucd_tables:
	rm -rf $@
	- mkdir $@
	nix-build -A ucd-tables
	cp -r result/* $@/
	rm result

