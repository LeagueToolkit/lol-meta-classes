#include <sys/select.h>
#include <unistd.h>

#include <common.h>

int $darwin$select$1050(int nfds, fd_set *readfds, fd_set *writefds, fd_set *exceptfds, struct timeval *timeout) {
  // FIXME: is this correct?
  return select(nfds, readfds, writefds, exceptfds, timeout);
}

int $darwin$connectx(int, const void *, void *, unsigned int, const void *, unsigned int, size_t *, void *) {
  // FIXME: ???
  panic("conntectx not working");
  return 0;
}

int $darwin$close$NOCANCEL(int fd) {
  // FIXME: do we need to ensure anything else here?
  // See https://issues.chromium.org/issues/41033222
  return close(fd);
}
