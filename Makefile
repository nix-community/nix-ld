all: nix-ld

PATCHELF ?= patchelf
PREFIX ?= /usr/local/
CP ?= cp
INSTALL = install
CFLAGS = -Wall -g
LD_CC ?= $(CC)
LD_CFLAGS = $(CFLAGS) -fPIC -static-pie

libexample.so:
	echo '' | $(CC) $(CFLAGS) -shared -o $@ -x c -

example-binary: libexample.so
	echo 'int main() { return 0; }' | $(CC) $(CFLAGS) -L. -lexample -o $@ -x c -

real-ld: example-binary
	$(PATCHELF) --print-interpreter $< > $@

ld-name: real-ld
	basename $(shell cat real-ld) > ld-name

nix-ld: nix-ld.c real-ld
	$(LD_CC) $(LD_CFLAGS) -DREAL_LD=\"$(shell cat real-ld)\" -o $@ $<

patched-example-binary: example-binary nix-ld
	$(CP) example-binary patched-example-binary
	$(PATCHELF) --set-rpath "" --set-interpreter $(shell readlink -f ./nix-ld) ./patched-example-binary

check: patched-example-binary
	LD_DEBUG=libs NIX_LD_LIBRARY_PATH=. ./patched-example-binary

install: nix-ld ld-name
	$(INSTALL) -D -m755 nix-ld $(PREFIX)/lib/nix-ld.so
	$(INSTALL) -D -m440 ld-name $(PREFIX)/nix-support/ld-name

CLEAN_TARGETS = patched-example-binary example-binary libexample.so nix-ld real-ld

.PHONY: clean install
clean:
	$(RM) -f $(CLEAN_TARGETS)
