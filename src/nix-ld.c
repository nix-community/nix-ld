#include <linux/elf.h>
#include <linux/auxvec.h>
#include <linux/mman.h>

#include <nolibc.h>

#include <stdio.h>
#include <config.h>

#include "strerror.h"
#include "access.h"

#define alloca __builtin_alloca

static inline void closep(const int *fd) { close(*fd); }

#define _cleanup_(f) __attribute__((cleanup(f)))
#define _cleanup_close_ _cleanup_(closep)

#if PTR_SIZE == 4
#define UINTPTR_MAX
typedef Elf32_Ehdr Ehdr;
typedef Elf32_Phdr Phdr;
#elif PTR_SIZE == 8
#define UINTPTR_MAX 0xffffffffffffffffu
typedef Elf64_Ehdr Ehdr;
typedef Elf64_Phdr Phdr;
#else
#error unsupported word width
#endif

#define DEFAULT_NIX_LD "/run/current-system/sw/share/nix-ld/lib/ld.so"
#define DEFAULT_NIX_LD_LIBRARY_PATH "/run/current-system/sw/share/nix-ld/lib"

typedef struct {
  void *addr;
  size_t size;
} mmap_t;

struct ld_ctx {
  const char *prog_name;
  const char *nix_ld;
  char *nix_ld_lib_path;
  char *ld_lib_path;
  char **ld_lib_path_envp;
  const char *nix_ld_env_prefix;
  const char *nix_lib_path_prefix;

  // filled out by elf_load
  unsigned long load_addr;
  unsigned long entry_point;
  size_t page_size;
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

  // cannot overflow because vsnprintf leaves space for the null byte
  fprintf(stderr, "\n");
}

static char *get_env(char* env, const char* key) {
  size_t key_len = strlen(key);

  if (strlen(env) < key_len) {
    return NULL;
  }

  if (memcmp(key, env, key_len) == 0) {
    return &env[key_len];
  }

  return NULL;
}


static struct ld_ctx init_ld_ctx(int argc, char **argv, char** envp, size_t *auxv) {
  // AT_BASE points to us, we need to point it to the new interpreter
  struct ld_ctx ctx = {
      .prog_name = argc ? argv[0] : "nix-ld",
      .ld_lib_path = NULL,
      .nix_ld = NULL,
      .nix_ld_lib_path = NULL,
      .nix_lib_path_prefix = "NIX_LD_LIBRARY_PATH_" NIX_SYSTEM "=",
  };

  for (; auxv[0]; auxv += 2) {
    if (auxv[0] == AT_PAGESZ) {
      ctx.page_size = auxv[1];
      break;
    }
  }

  char *val;
  for (char **e = envp; *e; e++) {
    if ((val = get_env(*e, "NIX_LD_" NIX_SYSTEM "="))) {
      ctx.nix_ld = val;
    } else if (!ctx.nix_ld && (val = get_env(*e, "NIX_LD="))) {
      ctx.nix_ld = val;
    } else if ((val = get_env(*e, ctx.nix_lib_path_prefix))) {
      ctx.nix_ld_lib_path = val;
    } else if (!ctx.nix_ld_lib_path && (val = get_env(*e, "NIX_LD_LIBRARY_PATH="))) {
      ctx.nix_lib_path_prefix = "NIX_LD_LIBRARY_PATH=";
      ctx.nix_ld_lib_path = val;
    } else if ((val = get_env(*e, "LD_LIBRARY_PATH="))) {
      ctx.ld_lib_path = val;
      ctx.ld_lib_path_envp = e;
    }
  }

  return ctx;
}

static size_t total_mapping_size(const Phdr *phdrs, size_t phdr_num) {
  size_t addr_min = UINTPTR_MAX;
  size_t addr_max = 0;
  for (size_t i = 0; i < phdr_num; i++) {
    const Phdr *ph = &phdrs[i];
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

static inline unsigned long page_start(struct ld_ctx* ctx, unsigned long v) {
  return v & ~(ctx->page_size - 1);
}

static inline unsigned long page_offset(struct ld_ctx* ctx, unsigned long v) {
  return v & (ctx->page_size - 1);
}

static inline unsigned long page_align(struct ld_ctx* ctx, unsigned long v) {
  return (v + ctx->page_size - 1) & ~(ctx->page_size - 1);
}

static inline int32_t prot_flags(uint32_t p_flags) {
  return (p_flags & PF_R ? PROT_READ : 0) | (p_flags & PF_W ? PROT_WRITE : 0) |
         (p_flags & PF_X ? PROT_EXEC : 0);
}

static int elf_map(struct ld_ctx *ctx, int fd, const Phdr *prog_headers,
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
    const Phdr *ph = &prog_headers[i];
    // zero sized segments are valid but we won't mmap them
    if (ph->p_type != PT_LOAD || !ph->p_memsz) {
      continue;
    }
    const int32_t prot = prot_flags(ph->p_flags);

    unsigned long addr = ctx->load_addr + page_start(ctx, ph->p_vaddr);
    size_t size =
        page_align(ctx, ph->p_vaddr + ph->p_memsz) - page_start(ctx, ph->p_vaddr);
    if (!ctx->load_addr) {
      // mmap the whole library range to reserve the area,
      // later smaller parts will be mmaped over it.
      size = page_align(ctx, total_size);
    };
    off_t off_start = page_start(ctx, ph->p_offset);
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
      size_t pgbrk = page_align(ctx, brk);
      size_t this_max = page_align(ctx, ph->p_vaddr + ph->p_memsz);
      if (page_offset(ctx, pgbrk - brk)) {
        memset((void *)brk, 0, page_offset(ctx, pgbrk - brk));
      }

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
    // log_error(stderr, "mmap 0x%lx (0x%lx) at %p (mmap_hint: 0x%lx) (vaddr:
    // 0x%lx, load_addr: 0x%lx, prot: ",
    //        size,
    //        ph->p_memsz,
    //        mapping,
    //        addr,
    //        ph->p_vaddr,
    //        ctx->load_addr);
    // log_error(stderr, "%c%c%c",
    //       ph->p_flags & PF_R ? 'r' : '-',
    //       ph->p_flags & PF_W ? 'w' : '-',
    //       ph->p_flags & PF_X ? 'x' : '-');
    // log_error(stderr, ")\n");
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

static int open_ld(const char *path) {
  const int fd = open(path, O_RDONLY, 0);
  size_t l = strlen(path);
  // ${stdenv.cc}/nix-support/dynamic-linker contains trailing newline
  if (fd < 0 && errno == ENOENT && path[l - 1] == '\n') {
    char *path_trunc = alloca(l);
    if (!path_trunc) {
      return -1;
    }
    memcpy(path_trunc, path, l - 1);
    path_trunc[l - 1] = '\0';
    return open(path_trunc, O_RDONLY, 0);
  }
  return fd;
}

static int elf_load(struct ld_ctx *ctx) {
  const _cleanup_close_ int fd = open_ld(ctx->nix_ld);
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

  _cleanup_(munmapp) mmap_t header_mapping = {
    .addr = mmap(NULL, sizeof(Ehdr) + ph_size, PROT_READ, MAP_PRIVATE, fd, 0),
    .size = sizeof(Ehdr) + ph_size,
  };

  if (header_mapping.addr == MAP_FAILED) {
    log_error(ctx, "cannot mmap program headers: %s", strerror(errno));
    return -1;
  }

  Phdr *prog_headers = header_mapping.addr + sizeof(Ehdr);

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
#elif defined(__i386__) || defined(__i486__) || defined(__i586__) || defined(__i686__)
  __asm__("mov %0, %%esp; jmp *%1" ::"r"(stackp), "r"(entry_point) : "memory");
#elif defined(__aarch64__)
  __asm__("mov sp, %0; br %1" ::"r"(stackp), "r"(entry_point) : "memory");
#elif defined(__ARM_EABI__)
  __asm__("mov sp, %0; bx %1" ::"r"(stackp), "r"(entry_point) : "memory");
#elif defined(__riscv)
  __asm__("mv sp, %0 ; jr %1" ::"r"(stackp), "r"(entry_point) : "memory");
#else
#error unsupported architecture
#endif
  __builtin_unreachable();
}

static void insert_ld_library_path(struct ld_ctx *ctx) {
  const size_t old_len = strlen(ctx->nix_lib_path_prefix);
  const size_t new_len = strlen("LD_LIBRARY_PATH=");

  char *env = ctx->nix_ld_lib_path - old_len;

  // insert new shorter variable
  memcpy(env, "LD_LIBRARY_PATH=", new_len);
  // shift the old content left
  memmove(env + new_len, ctx->nix_ld_lib_path, strlen(env) + new_len - old_len);
}

static int update_ld_library_path(struct ld_ctx *ctx) {
  const size_t prefix_len = strlen("LD_LIBRARY_PATH=");

  char *env = ctx->ld_lib_path - prefix_len;
  const size_t var_len = prefix_len + strlen(ctx->ld_lib_path);
  const char *sep;
  if (var_len == prefix_len || env[var_len] == ':') {
    // empty library path or ends with :
    sep = "";
  } else {
    sep = ":";
  }
  const size_t new_size =
    var_len + strlen(sep) + strlen(ctx->nix_ld_lib_path) + 1;
  char *new_str = mmap(NULL, new_size, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);

  if (new_str == MAP_FAILED) {
    return -errno;
  }

  // same as LD_LIBRARY_PATH=oldvalue:$NIX_LD_LIBRARY_PATH
  strlcpy(new_str, env, new_size);
  strlcat(new_str, sep, new_size);
  strlcat(new_str, ctx->nix_ld_lib_path, new_size);

  *ctx->ld_lib_path_envp = new_str;
  return 0;
}

static void* get_at_base(size_t *auxv) {
  // AT_BASE points to us, we need to point it to the new interpreter
  for (; auxv[0]; auxv += 2) {
    size_t key = auxv[0];
    size_t *value = &auxv[1];
    if (key == AT_BASE) {
      return value;
    }
  }
  return NULL;
}


int main(int argc, char** argv, char** envp) {
  size_t *auxv;
  for (auxv = (size_t *)envp; *auxv; auxv++) {
  }
  auxv++;

  struct ld_ctx ctx = init_ld_ctx(argc, argv, envp, auxv);

  if (!ctx.nix_ld) {
    // FIXME: fallback to default ld.so
    // This requires however to also increase envp to have space for the new environment variable
    //if (access(DEFAULT_NIX_LD, R_OK) == 0) {
    //  ctx.nix_ld = DEFAULT_NIX_LD;
    //  // if no NIX_LD is set we also don't trust NIX_LD_LIBRARY_PATH since it may point to a different libc
    //  ctx.nix_lib_path_prefix = DEFAULT_NIX_LD_LIBRARY_PATH;
    //} else {
    //  log_error(&ctx, "You are trying to run an unpatched binary on nixos, but you have not configured NIX_LD or NIX_LD_" NIX_SYSTEM ". See https://github.com/Mic92/nix-ld for more details");
    //  return 1;
    //}
    log_error(&ctx, "You are trying to run an unpatched binary on nixos, but you have not configured NIX_LD or NIX_LD_" NIX_SYSTEM ". See https://github.com/Mic92/nix-ld for more details");
    return 1;
  }

  if (!ctx.page_size) {
    log_error(&ctx, "no page size (AT_PAGESZ) given by operating system in auxv.");
    return 1;
  }

  if (elf_load(&ctx) < 0) {
    // elf_load prints the error;
    return 1;
  }

  if (ctx.nix_ld_lib_path) {
    if (ctx.ld_lib_path) {
      update_ld_library_path(&ctx);
    } else {
      insert_ld_library_path(&ctx);
    }
  }

  size_t *at_base = get_at_base(auxv);
  if (at_base) {
    if (*at_base == 0) {
      // We have been executed as a dynamic executable, so we need to execute
      // the interpreter as a dynamic executable.
      execve(ctx.nix_ld, argv, envp);
    }
    *at_base = ctx.load_addr;
  }

  const size_t *stackp = ((size_t *)argv - 1);
  jmp_ld((void (*)(void))ctx.entry_point, (void *)stackp);

  return 0;
}
