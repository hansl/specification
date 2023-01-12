CDDL_FILES := $(shell find * -type f -name \*.cddl -a \( \! -path target/all.cddl \))
.PHONY: compile-cddl cddl-check clean

clean:
	rm target/all.cddl

target/all.cddl: $(CDDL_FILES)
	mkdir -p "$(@D)"
	rm target/all.cddl || true
	scripts/make_cddl.bash target/all.cddl

target/bin/cddl:
	cargo install cddl --root target/

compile-cddl:
	@scripts/make_cddl.bash

cddl-check: target/all.cddl target/bin/cddl
	target/bin/cddl compile-cddl --cddl target/all.cddl

