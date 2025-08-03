#include <errno.h>
#include <error.h>
#include <signal.h>
#include <string.h>
#include <time.h>

#include <common.h>

struct $darwin$utsname {
  char sysname[256];
  char nodename[256];
  char release[256];
  char version[256];
  char machine[256];
};

struct $darwin$sigaction {
  void *sigaction_u;
  int sa_flags;
  uint32_t sa_mask;
};

void $darwin$arc4random_buf(void *buf, size_t nbytes) {
  // chosen by fair dice roll.
  // guaranteed to be random.
  memset(buf, 4, nbytes);
}

int $darwin$sigaction(int sig, const struct $darwin$sigaction *act, struct $darwin$sigaction *oact) {
  struct sigaction gnu_act;
  struct sigaction gnu_oact;

  if (sig == SIGPIPE) {
    // getting action
    if (act == NULL && oact != NULL) {
      int result = sigaction(sig, NULL, &gnu_oact);
      oact->sigaction_u = gnu_oact.sa_handler;
      oact->sa_flags = gnu_oact.sa_flags;
      return result;
    }

    // setting action
    if (act != NULL && oact == NULL) {
      // GNU has restorer as well, need to get it
      sigaction(sig, NULL, &gnu_act);
      gnu_act.sa_handler = act->sigaction_u;
      gnu_act.sa_flags = act->sa_flags;
      int result = sigaction(sig, &gnu_act, NULL);
      return result;
    }
  }

  fprintf(stderr, "ignoring sigaction: %d...\n", sig);
  return 0;
}

int $darwin$sysctlbyname(const char *name, void *oldp, size_t *oldlenp, const void *newp, size_t newlen) {
  if (!name || !oldlenp) {
    errno = EINVAL;
    return -1;
  }
  if (!oldp && *oldlenp) {
    errno = EINVAL;
    return -1;
  }
  // FIXME: used for statistics, don't really care
  // hw.ncpu 4
  // hw.memsize 8
  // machdep.cpu.brand_string 1024
  // hw.cpufrequency 8
  // hw.model 256
  // kern.osproductversion 32
  // kern.osversion 32
  memset(oldp, 0, *oldlenp);
  errno = ENOENT;
  return -1;
}

void $darwin$uuid_generate(uint32_t out[2]) {
  // NOTE: don't really care in particular ?
  static uint64_t ticking = 0;
  out[0] = ++ticking;
  out[1] = random();
  out[2] = random();
  out[3] = ++ticking;
}

int $darwin$uname(struct $darwin$utsname *buf) {
  // TODO: make this nicer maybe?
  memset(buf, 0, sizeof(struct $darwin$utsname));
  return 0;
}

void $darwin$__cxa_throw() { panic("throwing dont work"); }
