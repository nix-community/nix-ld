CARGO ?= cargo
TARGET ?= debug
CARGO_FLAGS ?=
BUILDDIR := build
PREFIX ?= /usr/local
LIBEXECDIR ?= $(PREFIX)/libexec
INSTALL ?= install

ifeq ($(TARGET),release)
  CARGO_FLAGS += --release
endif

all: $(BUILDDIR)/nix-ld

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

$(BUILDDIR)/$(TARGET)/libnix_ld.a: $(BUILDDIR)
	$(CARGO) build --target-dir $(BUILDDIR) $(CARGO_FLAGS)

$(BUILDDIR)/nix-ld: $(BUILDDIR)/$(TARGET)/libnix_ld.a $(BUILDDIR)
	$(LD) -o $@ $<

install:
	$(INSTALL) -D -m755 $(BUILDDIR)/nix-ld $(LIBEXECDIR)/nix-ld

clean:
	rm -rf "$(BUILDDIR)"

.PHONY: clean all install
