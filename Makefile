CARGO ?= cargo
TARGET ?= debug
CARGO_FLAGS ?=
BUILDDIR := build
PREFIX ?= /usr/local
LIBEXECDIR ?= $(PREFIX)/libexec
INSTALL ?= install
AR ?= ar
ARCH_TARGET ?= $(shell gcc -dumpmachine)
#
#ifeq ($(TARGET),release)
#  CARGO_FLAGS += --release
#endif
#
#
#$(BUILDDIR):
#	mkdir -p $(BUILDDIR)
#
#PLATFORM_FILES = start.s syscalls.c jmp_ld.s breakpoint.s
#PLATFORM_SRC = $(addprefix src/$(ARCH_TARGET)/,$(PLATFORM_FILES))
#PLATFORM_OBJS = $(addsuffix .o,$(addprefix $(BUILDDIR)/,$(PLATFORM_FILES)))
#LIB_OBJS = $(filter-out $(BUILDDIR)/start.s.o, $(PLATFORM_OBJS))
#CRT_OBJS = $(BUILDDIR)/start.s.o
#
#$(BUILDDIR)/%.s.o: src/$(ARCH_TARGET)/%.s
#	$(CC) -c -o $@ $<
#$(BUILDDIR)/%.c.o: src/$(ARCH_TARGET)/%.c
#	$(CC) -c -o $@ $<
#
#ALL_OBJS = $(addprefix obj/, $(filter-out $(REPLACED_OBJS), $(sort $(BASE_OBJS) $(ARCH_OBJS))))
#
#$(BUILDDIR)/$(TARGET)/libnix_ld.a: $(BUILDDIR)
#	$(CARGO) build --target-dir $(BUILDDIR) $(CARGO_FLAGS)
#
#$(BUILDDIR)/nix-ld: $(BUILDDIR)/$(TARGET)/libnix_ld.a $(PLATFORM_OBJS) $(BUILDDIR)
#	$(LD) -pie -o $@ $(CRT_OBJS) $< $(LIB_OBJS)
#
#install:
#	$(INSTALL) -D -m755 $(BUILDDIR)/nix-ld $(LIBEXECDIR)/nix-ld
#
#clean:
#	rm -rf "$(BUILDDIR)"
#
#.PHONY: clean all install

all: $(BUILDDIR)/nix-ld

$(BUILDDIR)/x86_64-unknown-linux-musl/$(TARGET)/libnix_ld.a: $(BUILDDIR)
	xargo build --target x86_64-unknown-linux-musl --target-dir $(BUILDDIR) $(CARGO_FLAGS)

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

$(BUILDDIR)/nix-ld: $(BUILDDIR)/x86_64-unknown-linux-musl/$(TARGET)/libnix_ld.a $(BUILDDIR)
	$(LD) -static -pie -o $@ $<

install:
	$(INSTALL) -D -m755 $(BUILDDIR)/nix-ld $(LIBEXECDIR)/nix-ld

clean:
	rm -rf "$(BUILDDIR)"

.PHONY: clean all install
