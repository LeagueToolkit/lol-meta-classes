#include <inttypes.h>
#include <common.h>

// NOTE: there is kevent library for linux, maybe use it if it ever becmes necesary.
struct __attribute__((packed)) $darwin$kevent {
  uintptr_t ident;
  int16_t filter;
  uint16_t flags;
  uint32_t fflags;
  intptr_t data;
  void *udata;
  // uint64_t ext[4];
};

int $darwin$kqueue(void) {
  // FIXME: doesn't seem to be used
  return 0;
}

int $darwin$kqueuex(unsigned flags) {
  // TODO
  panic("kqueuex");
}

int $darwin$kevent(int kq, struct $darwin$kevent *changelist, int nchanges, struct $darwin$kevent *eventlist, int nevents, const struct timespec *timeout) {
  // FIXME: doesn't seem to do much
  return 0;
}
