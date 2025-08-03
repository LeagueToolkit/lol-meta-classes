#include <ctype.h>
#include <string.h>
#include <sys/mman.h>

#include <common.h>

int $darwin$__tolower(int c) { return tolower(c); }

int $darwin$__toupper(int c) { return toupper(c); }

size_t $darwin$strlcpy(char *dst, const char *src, size_t dst_size) {
  size_t src_size = strlen(src);
  if (dst_size) {
    size_t n = (src_size + 1) < dst_size ? (src_size + 1) : dst_size;
    memcpy(dst, src, n - 1);
    dst[n - 1] = 0;
  }
  return src_size;
}

size_t $darwin$strlcat(char *dst, const char *src, size_t size) {
  if (!size) return 0;
  dst[size - 1] = 0;
  strncat(dst, src, size - strlen(dst) - 1);
  return strlen(dst);
}

int $darwin$vsnprintf_l(char *str, size_t size, locale_t loc, const char *format, va_list ap) {
  // used for sentry, don't really care
  return vsnprintf(str, size, format, ap);
}

void $darwin$__bzero(void *dst, size_t size) { memset(dst, 0, size); }

void $darwin$memset_pattern16(void *b, const void *pattern16, size_t len) {
  char *dst = (char *)b;
  size_t rem = len;
  while (rem >= 16) {
    memcpy(dst, pattern16, 16);
    dst += 16;
    rem -= 16;
  }
  memcpy(dst, pattern16, rem);
}

void *$darwin$mmap(void *addr, size_t len, int prot, int flags, int fd, off_t offset) {
  flags = (flags & 0x1f) | (flags & 0x1000 ? MAP_ANONYMOUS : 0);
  void *ret = mmap(addr, len, prot, flags, fd, offset);
  // printf("mmap(%p, %zx, %x, %x, %d, %zx) = %p\n", addr, len, prot, flags, fd, offset, ret);
  return ret;
}
