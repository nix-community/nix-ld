#define _GNU_SOURCE
#include <elf.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/auxv.h>
#include <sys/mman.h>
#include <unistd.h>

#include "config.h"

static inline void closep(int *fd) { close(*fd); }
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
  const char *interp_path;

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
    fprintf(stderr,
            "cannot execute %s: no program headers found in $NIX_LD (%s)",
            ctx->prog_name, ctx->interp_path);
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
      fprintf(stderr, "cannot execute %s: mmap segment of %s failed: %s\n",
              ctx->prog_name, ctx->interp_path, strerror(errno));
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
          fprintf(stderr, "cannot execute %s: mmap segment of %s failed: %s\n",
                  ctx->prog_name, ctx->interp_path, strerror(errno));
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
  _cleanup_close_ int fd = open(ctx->interp_path, O_RDONLY);
  if (fd < 0) {
    fprintf(stderr, "cannot execute '%s': cannot open $NIX_LD (%s): %s\n",
            ctx->prog_name, ctx->interp_path, strerror(errno));
    return -1;
  }
  Ehdr header = {};
  ssize_t res = read(fd, &header, sizeof(header));
  if (res < 0) {
    fprintf(stderr,
            "cannot execute '%s': cannot read elf header of $NIX_LD (%s): %s\n",
            ctx->prog_name, ctx->interp_path, strerror(errno));
    return -1;
  }

  if (memcmp(header.e_ident, ELFMAG, SELFMAG) != 0) {
    fprintf(stderr,
            "cannot execute '%s': $NIX_LD (%s) is not an elf file: %s\n",
            ctx->prog_name, ctx->interp_path, strerror(errno));
    return -1;
  }

  // TODO also support dynamic excutable
  if (header.e_type != ET_DYN) {
    fprintf(stderr,
            "cannot execute '%s': $NIX_LD (%s) is not a dynamic library\n",
            ctx->prog_name, ctx->interp_path);
    return -1;
  }

  size_t ph_size = sizeof(Phdr) * header.e_phnum;
  // XXX binfmt_elf also checks ELF_MIN_ALIGN here
  if (ph_size == 0 || ph_size > 65536) {
    fprintf(
        stderr,
        "cannot execute %s: $NIX_LD (%s) has incorrect program header size: "
        "%zu\n",
        ctx->prog_name, ctx->interp_path, ph_size);
    return -1;
  }

  _cleanup_free_ Phdr *prog_headers = malloc(ph_size);
  if (!prog_headers) {
    fprintf(stderr, "cannot execute '%s': cannot allocate program headers\n",
            ctx->prog_name);
    return -1;
  }
  res = read(fd, prog_headers, ph_size);

  if (res < 0) {
    fprintf(stderr,
            "cannot execute '%s': cannot read program headers of elf "
            "interpreter: %s\n",
            ctx->prog_name, strerror(errno));
    return -1;
  }

  if ((size_t)res != ph_size) {
    fprintf(stderr,
            "cannot execute '%s': less program headers in elf interpreter than "
            "expected: %zu != %zu\n",
            ctx->prog_name, (size_t)res, ph_size);
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

int main(int argc, char **argv) {
  struct ld_ctx ctx = {};
  ctx.prog_name = argv[0];
  ctx.interp_path = secure_getenv("NIX_LD_" NIX_SYSTEM);
  if (!ctx.interp_path) {
    ctx.interp_path = secure_getenv("NIX_LD");
  }

  char *lib_path = secure_getenv("NIX_LD_LIBRARY_PATH_" NIX_SYSTEM);
  if (!lib_path) {
    lib_path = secure_getenv("NIX_LD_LIBRARY_PATH");
  }

  if (!ctx.interp_path) {
    fprintf(stderr, "cannot execute '%s': $NIX_LD is not set\n", argv[0]);
    return 1;
  }

  if (elf_load(&ctx) < 0) {
    // elf_load prints the error;
    return 1;
  }

  const size_t *stackp = ((size_t *)argv - 1);
  char **envp;
  for (envp = &argv[argc + 1]; *envp; envp++)
    ;

  size_t *auxv = (size_t *)(envp + 1);

  // AT_BASE points to us, we need to point it to the new interpreter
  for (; auxv[0]; auxv += 2) {
    size_t key = auxv[0];
    size_t *value = &auxv[1];
    if (key == AT_BASE) {
      *value = ctx.load_addr;
      break;
    }
  }

  jmp_ld((void (*)(void))ctx.entry_point, (void *)stackp);

  return 0;
}
