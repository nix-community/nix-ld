#define MAP_FAILED ((void *) -1)

static void* sys_mmap(void *addr, size_t length, int prot, int flags,
                      int fd, off_t offset) {
  return (void*)my_syscall6(__NR_mmap, addr, length, prot, flags, fd, offset);
}

static int sys_munmap(void *addr, size_t length) {
  return my_syscall2(__NR_munmap, addr, length);
}

static void* mmap(void *addr, size_t length, int prot, int flags,
                  int fd, off_t offset) {
  void* ret = sys_mmap(addr, length, prot, flags, fd, offset);
  if ((ssize_t)ret < 0 && (ssize_t)ret >= -256) {
    SET_ERRNO(-(ssize_t)ret);
    return MAP_FAILED;
  }
  return ret;
}

static int munmap(void *addr, size_t length) {
  return sys_munmap(addr, length);
}
