#define _GNU_SOURCE
#include <assert.h>
#include <elf.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/auxv.h>
#include <sys/mman.h>
#include <unistd.h>

#include "config.h"

static inline void closep(const int *fd) { close(*fd); }
static inline void freep(void *p) { free(*(void **)p); }

#define _cleanup_(f) __attribute__((cleanup(f)))
#define _cleanup_close_ _cleanup_(closep)
#define _cleanup_free_ _cleanup_(freep)
#define _cleanup__ _cleanup_(free)

#if UINTPTR_MAX == 0xffffffff
typedef Elf32_Ehdr Ehdr;
typedef Elf32_Phdr Phdr;
#else
typedef Elf64_Ehdr Ehdr;
typedef Elf64_Phdr Phdr;
#endif

typedef struct {
  void *addr;
  size_t size;
} mmap_t;

struct ld_ctx {
  const char *prog_name;
  const char *nix_ld;
  const char *nix_ld_lib_path;
  const char *ld_lib_path;
  const char *nix_ld_env_prefix;
  const char *nix_lib_path_prefix;

  // filled out by elf_load
  unsigned long load_addr;
  unsigned long entry_point;
  mmap_t mapping;
};

static inline void munmapp(mmap_t *m) {
  if (m->size) {
    munmap(m->addr, m->size);
  }
}

static void log_error(struct ld_ctx *ctx, const char *format, ...) {
  va_list args;
  fprintf(stderr, "cannot execute %s: ", ctx->prog_name);

  va_start(args, format);
  vfprintf(stderr, format, args);
  va_end(args);

  fputc('\n', stderr);
}

struct ld_ctx init_ld_ctx(char **argv) {
  struct ld_ctx ctx = {
      .prog_name = argv[0],
      .ld_lib_path = secure_getenv("LD_LIBRARY_PATH"),
      .nix_ld = secure_getenv("NIX_LD_" NIX_SYSTEM),
      .nix_ld_lib_path = secure_getenv("NIX_LD_LIBRARY_PATH_" NIX_SYSTEM),
      .nix_lib_path_prefix = "NIX_LD_LIBRARY_PATH_" NIX_SYSTEM "=",
  };

  if (!ctx.nix_ld) {
    ctx.nix_ld = secure_getenv("NIX_LD");
  }

  if (!ctx.nix_ld_lib_path) {
    ctx.nix_lib_path_prefix = "NIX_LD_LIBRARY_PATH=";
    ctx.nix_ld_lib_path = secure_getenv("NIX_LD_LIBRARY_PATH");
  }

  return ctx;
}

static size_t total_mapping_size(Phdr *phdrs, size_t phdr_num) {
  size_t addr_min = UINTPTR_MAX;
  size_t addr_max = 0;
  for (size_t i = 0; i < phdr_num; i++) {
    Phdr *ph = &phdrs[i];
    if (ph->p_type != PT_LOAD || ph->p_memsz == 0) {
      continue;
    }
    if (ph->p_vaddr < addr_min) {
      addr_min = ph->p_vaddr;
    }
    if (ph->p_vaddr + ph->p_memsz > addr_max) {
      addr_max = ph->p_vaddr + ph->p_memsz;
    }
  }
  return addr_max - addr_min;
}

static inline unsigned long page_start(unsigned long v) {
  return v & ~(getpagesize() - 1);
}

static inline unsigned long page_offset(unsigned long v) {
  return v & (getpagesize() - 1);
}

static inline unsigned long page_align(unsigned long v) {
  return (v + getpagesize() - 1) & ~(getpagesize() - 1);
}

static inline int32_t prot_flags(uint32_t p_flags) {
  return (p_flags & PF_R ? PROT_READ : 0) | (p_flags & PF_W ? PROT_WRITE : 0) |
         (p_flags & PF_X ? PROT_EXEC : 0);
}

static int elf_map(struct ld_ctx *ctx, int fd, Phdr *prog_headers,
                   size_t headers_num) {
  const size_t total_size = total_mapping_size(prog_headers, headers_num);

  if (total_size == 0) {
    log_error(ctx, "no program headers found in $NIX_LD (%s)", ctx->nix_ld);
    return -1;
  }

  ctx->load_addr = 0;
  ctx->mapping.addr = NULL;
  _cleanup_(munmapp) mmap_t total_mapping = {};

  for (size_t i = 0; i < headers_num; i++) {
    Phdr *ph = &prog_headers[i];
    // zero sized segments are valid but we won't mmap them
    if (ph->p_type != PT_LOAD || !ph->p_memsz) {
      continue;
    }
    const int32_t prot = prot_flags(ph->p_flags);

    unsigned long addr = ctx->load_addr + page_start(ph->p_vaddr);
    size_t size =
        page_align(ph->p_vaddr + ph->p_memsz) - page_start(ph->p_vaddr);
    if (!ctx->load_addr) {
      // mmap the whole library range to reserve the area,
      // later smaller parts will be mmaped over it.
      size = page_align(total_size);
    };
    off_t off_start = page_start(ph->p_offset);
    int flags = MAP_PRIVATE | MAP_FIXED;
    if (!ctx->load_addr) {
      flags = MAP_PRIVATE;
    };

    void *mapping = mmap((void *)addr, size, prot, flags, fd, off_start);
    if (mapping == MAP_FAILED) {
      log_error(ctx, "mmap segment of %s failed: %s", ctx->nix_ld,
                strerror(errno));
      return -1;
    }

    if (ph->p_memsz > ph->p_filesz && (ph->p_flags & PF_W)) {
      size_t brk = ctx->load_addr + ph->p_vaddr + ph->p_filesz;
      size_t pgbrk = page_align(brk);
      size_t this_max = page_align(ph->p_vaddr + ph->p_memsz);
      memset((void *)brk, 0, page_offset(pgbrk - brk));

      if (pgbrk - ctx->load_addr < this_max) {
        void *res = mmap((void *)pgbrk, ctx->load_addr + this_max - pgbrk, prot,
                         MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS, -1, 0);
        if (res == MAP_FAILED) {
          log_error(ctx, "mmap segment of %s failed: %s", ctx->nix_ld,
                    strerror(errno));
          return -1;
        };
      }
    }
    // useful for debugging
    // fprintf(stderr, "mmap 0x%lx (0x%lx) at %p (mmap_hint: 0x%lx) (vaddr:
    // 0x%lx, load_addr: 0x%lx, prot: ",
    //        size,
    //        ph->p_memsz,
    //        mapping,
    //        addr,
    //        ph->p_vaddr,
    //        ctx->load_addr);
    // fprintf(stderr, "%c%c%c",
    //       ph->p_flags & PF_R ? 'r' : '-',
    //       ph->p_flags & PF_W ? 'w' : '-',
    //       ph->p_flags & PF_X ? 'x' : '-');
    // fprintf(stderr, ")\n");
    if (ctx->load_addr == 0) {
      ctx->load_addr = (unsigned long)mapping - ph->p_vaddr;
      total_mapping.addr = mapping;
      total_mapping.size = size;
    }
  }
  ctx->mapping = total_mapping;
  total_mapping.addr = NULL;
  total_mapping.size = 0;
  return 0;
}

static int elf_load(struct ld_ctx *ctx) {
  const _cleanup_close_ int fd = open(ctx->nix_ld, O_RDONLY);
  if (fd < 0) {
    log_error(ctx, "cannot open $NIX_LD (%s): %s", ctx->nix_ld,
              strerror(errno));
    return -1;
  }
  Ehdr header = {};
  ssize_t res = read(fd, &header, sizeof(header));
  if (res < 0) {
    log_error(ctx, "cannot read elf header of $NIX_LD (%s): %s", ctx->nix_ld,
              strerror(errno));
    return -1;
  }

  if (memcmp(header.e_ident, ELFMAG, SELFMAG) != 0) {
    log_error(ctx, "$NIX_LD (%s) is not an elf file: %s", ctx->nix_ld,
              strerror(errno));
    return -1;
  }

  // TODO also support dynamic excutable
  if (header.e_type != ET_DYN) {
    log_error(ctx, "$NIX_LD (%s) is not a dynamic library", ctx->nix_ld,
              strerror(errno));
    return -1;
  }

  const size_t ph_size = sizeof(Phdr) * header.e_phnum;
  // XXX binfmt_elf also checks ELF_MIN_ALIGN here
  if (ph_size == 0 || ph_size > 65536) {
    log_error(ctx, "$NIX_LD (%s) has incorrect program header size: %zu",
              ctx->nix_ld, ph_size);
    return -1;
  }

  _cleanup_free_ Phdr *prog_headers = malloc(ph_size);
  if (!prog_headers) {
    log_error(ctx, "cannot allocate program headers");
    return -1;
  }
  res = read(fd, prog_headers, ph_size);

  if (res < 0) {
    log_error(ctx, "cannot read program headers of elf interpreter: %s",
              strerror(errno));
    return -1;
  }

  if ((size_t)res != ph_size) {
    log_error(
        ctx,
        "less program headers in elf interpreter than expected: %zu != %zu",
        (size_t)res, ph_size);
    return -1;
  }

  int r = elf_map(ctx, fd, prog_headers, header.e_phnum);
  if (r < 0) {
    // elf_map prints the error;
    return -1;
  }

  ctx->entry_point = ctx->load_addr + header.e_entry;
  return 0;
}

// Musl defines this as CRT_JMP in musl/arch/<cpuarch>/reloc.h
static inline _Noreturn void jmp_ld(void (*entry_point)(void), void *stackp) {
#if defined(__x86_64__)
  __asm__("mov %0, %%rsp; jmp *%1" ::"r"(stackp), "r"(entry_point) : "memory");
#elif defined(__i386__)
  __asm__("mov %0, %%esp; jmp *%1" ::"r"(stackp), "r"(entry_point) : "memory");
#elif defined(__aarch64__)
  __asm__("mov sp, %0; br %1" ::"r"(stackp), "r"(entry_point) : "memory");
#elif defined(__arm__)
  __asm__("mov sp, %0; bx %1" ::"r"(stackp), "r"(entry_point) : "memory");
#elif defined(__riscv)
  __asm__("mv sp, %0 ; jr %1" ::"r"(stackp), "r"(entry_point) : "memory");
#else
#error unsupported architecture
#endif
  __builtin_unreachable();
}

void insert_ld_library_path(struct ld_ctx *ctx, char **envp) {
  for (; *envp; envp++) {
    // we re-use the NIX_LD_LIBRARY_PATH slot to populate LD_LIBRARY_PATH
    if (strncmp(ctx->nix_lib_path_prefix, *envp,
                strlen(ctx->nix_lib_path_prefix)) != 0) {
      continue;
    }
    const size_t old_len = strlen(ctx->nix_lib_path_prefix);
    const size_t new_len = strlen("LD_LIBRARY_PATH=");

    // insert new shorter variable
    memcpy(*envp, "LD_LIBRARY_PATH=", new_len);
    // shift the old content left
    memmove(*envp + new_len, *envp + old_len,
            strlen(*envp) + new_len - old_len);
    return;
  }
  log_error(ctx, "BUG could not set LD_LIBRARY_PATH from NIX_LD_LIBRARY_PATH: "
                 "NIX_LD_LIBRARY_PATH not found");
  abort();
}

int update_ld_library_path(struct ld_ctx *ctx, char **envp) {
  const size_t prefix_len = strlen("LD_LIBRARY_PATH=");

  for (; *envp; envp++) {
    if (strncmp("LD_LIBRARY_PATH=", *envp, prefix_len) != 0) {
      continue;
    }
    const size_t var_len = strlen(*envp);
    const char *sep;
    if (var_len == prefix_len || (*envp)[var_len] == ':') {
      // empty library path or ends with :
      sep = "";
    } else {
      sep = ":";
    }
    const size_t new_size =
        var_len + strlen(sep) + strlen(ctx->nix_ld_lib_path) + 1;
    char *new_str = malloc(new_size);
    if (!new_str) {
      return -ENOMEM;
    }
    // same as LD_LIBRARY_PATH=oldvalue:$NIX_LD_LIBRARY_PATH
    snprintf(new_str, new_size, "%s%s%s", *envp, sep, ctx->nix_ld_lib_path);

    *envp = new_str;
    return 0;
  }

  log_error(
      ctx,
      "BUG could not append to LD_LIBRARY_PATH: LD_LIBRARY_PATH not found");
  abort();
}

void fix_auxv(size_t *auxv, size_t load_addr) {
  // AT_BASE points to us, we need to point it to the new interpreter
  for (; auxv[0]; auxv += 2) {
    size_t key = auxv[0];
    size_t *value = &auxv[1];
    if (key == AT_BASE) {
      *value = load_addr;
      break;
    }
  }
}

int main(int argc, char **argv) {
  struct ld_ctx ctx = init_ld_ctx(argv);

  if (!ctx.nix_ld) {
    log_error(&ctx, "NIX_LD or NIX_LD_" NIX_SYSTEM " is not set");
    return 1;
  }

  if (elf_load(&ctx) < 0) {
    // elf_load prints the error;
    return 1;
  }

  const size_t *stackp = ((size_t *)argv - 1);
  char **envp = &argv[argc + 1];

  size_t *auxv;
  for (auxv = (size_t *)envp; *auxv; auxv++) {
  }
  auxv++;
  fix_auxv(auxv, ctx.load_addr);

  if (ctx.nix_ld_lib_path) {
    if (ctx.ld_lib_path) {
      update_ld_library_path(&ctx, envp);
    } else {
      insert_ld_library_path(&ctx, envp);
    }
  }

  jmp_ld((void (*)(void))ctx.entry_point, (void *)stackp);

  return 0;
}
