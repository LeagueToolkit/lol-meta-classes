#define _GNU_SOURCE
#include <errno.h>
#include <pthread.h>
#include <sched.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include <time.h>

#include <common.h>

typedef struct $darwin$pthread_cond_s {
  // uint64_t sig;
  // char opaque[40];
  pthread_cond_t native[1];
} $darwin$pthread_cond_t;

typedef struct $darwin$pthread_once_s {
  uint64_t sig;
  pthread_once_t native[1];
} $darwin$pthread_once_t;

typedef struct $darwin$pthread_condattr_s {
  uint64_t sig;
  pthread_condattr_t native[1];
} $darwin$pthread_condattr_t;

typedef struct $darwin$pthread_mutex_s {
  uint64_t sig;
  uint64_t sig_lock;  // we can fit extra lock here
  pthread_mutex_t native[1];
} $darwin$pthread_mutex_t;

typedef struct $darwin$pthread_mutexattr_s {
  uint64_t sig;
  pthread_mutexattr_t native[1];
} $darwin$pthread_mutexattr_t;

typedef struct $darwin$pthread_attr_s {
  uint64_t sig;
  pthread_attr_t native[1];
} $darwin$pthread_attr_t;

typedef struct $darwin$pthread_rwlock_s {
  uint64_t sig;
  pthread_rwlock_t native[1];
  // char extra[136];
} $darwin$pthread_rwlock_t;

typedef struct $darwin$pthread_rwlockattr_s {
  uint64_t sig;
  pthread_rwlockattr_t native[1];
} $darwin$pthread_rwlockattr_t;

int $darwin$pthread_atfork(void (*prepare)(void), void (*parent)(void), void (*child)(void)) {
  // NOTE: forks not supported anyways
  return 0;
}

// TODO: check sigs https://opensource.apple.com/source/Libc/Libc-825.40.1/pthreads/pthread_internals.h.auto.html

static pthread_mutexattr_t *pthread_mutexattr_to_linux($darwin$pthread_mutexattr_t *attr) {
  if (!attr) return NULL;
  return attr->native;
}

static void pthread_mutex_fixup($darwin$pthread_mutex_t *mutex) {
  uint64_t sig = mutex->sig;
  if (sig == 0x32AAABA7) {
    pthread_mutex_t expected_value = PTHREAD_MUTEX_INITIALIZER;
    memcpy(mutex->native, &expected_value, sizeof(expected_value));
  } else if (sig == 0x32AAABA1) {
    pthread_mutex_t expected_value = PTHREAD_ERRORCHECK_MUTEX_INITIALIZER_NP;
    memcpy(mutex->native, &expected_value, sizeof(expected_value));
  } else if (sig == 0x32AAABA2) {
    pthread_mutex_t expected_value = PTHREAD_ERRORCHECK_MUTEX_INITIALIZER_NP;
    memcpy(mutex->native, &expected_value, sizeof(expected_value));
  } else if (sig == 0x32AAABA3) {
    pthread_mutex_t expected_value = PTHREAD_ADAPTIVE_MUTEX_INITIALIZER_NP;
    memcpy(mutex->native, &expected_value, sizeof(expected_value));
  }
}

static pthread_mutex_t *pthread_mutex_to_linux($darwin$pthread_mutex_t *mutex) {
  if (!mutex) return NULL;
  // FIXME: properly lock this and initialize
  return mutex->native;
}

static pthread_attr_t *pthread_attr_to_linux($darwin$pthread_attr_t *attr) {
  if (!attr) return NULL;
  return attr->native;
}

static pthread_cond_t *pthread_cond_to_linux($darwin$pthread_cond_t *cond) {
  // TODO: check:
  // PTHREAD_COND_INITIALIZER
  if (!cond) return NULL;
  return cond->native;
}

static pthread_condattr_t *pthread_condattr_to_linux($darwin$pthread_condattr_t *attr) {
  if (!attr) return NULL;
  return attr->native;
}

static pthread_rwlockattr_t *pthread_rwlockattr_to_linux($darwin$pthread_rwlockattr_t *attr) {
  if (!attr) return NULL;
  return attr->native;
}

static pthread_rwlock_t *pthread_rwlock_to_linux($darwin$pthread_rwlock_t *rwlock) {
  // TODO: check:
  // PTHREAD_RWLOCK_INITIALIZER
  // PTHREAD_RWLOCK_WRITER_NONRECURSIVE_INITIALIZER_NP
  if (!rwlock) return NULL;
  return rwlock->native;
}

int $darwin$pthread_mutexattr_destroy($darwin$pthread_mutexattr_t *attr) {
  // NOTE: does nothing
  return pthread_mutexattr_destroy(pthread_mutexattr_to_linux(attr));
}

int $darwin$pthread_mutexattr_init($darwin$pthread_mutexattr_t *attr) {
  // NOTE: size is good.
  return pthread_mutexattr_init(pthread_mutexattr_to_linux(attr));
}

int $darwin$pthread_mutexattr_settype($darwin$pthread_mutexattr_t *attr, int type) {
  // NOTE: size is good, need to swap type
  switch (type) {
    case 0:
      type = PTHREAD_MUTEX_FAST_NP;
      break;
    case 1:
      type = PTHREAD_MUTEX_ERRORCHECK_NP;
      break;
    case 2:
      type = PTHREAD_MUTEX_RECURSIVE_NP;
      break;
  }
  return pthread_mutexattr_settype(pthread_mutexattr_to_linux(attr), type);
}

int $darwin$pthread_mutex_destroy($darwin$pthread_mutex_t *mutex) {
  // NOTE: need to convert mutex
  return pthread_mutex_destroy(pthread_mutex_to_linux(mutex));
}

int $darwin$pthread_mutex_init($darwin$pthread_mutex_t *mutex, $darwin$pthread_mutexattr_t *attr) {
  // NOTE: need to convert mutex
  pthread_mutex_fixup(mutex);
  return pthread_mutex_init(pthread_mutex_to_linux(mutex), pthread_mutexattr_to_linux(attr));
}

int $darwin$pthread_mutex_lock($darwin$pthread_mutex_t *mutex) {
  // NOTE: need to convert mutex
  return pthread_mutex_lock(pthread_mutex_to_linux(mutex));
}

int $darwin$pthread_mutex_trylock($darwin$pthread_mutex_t *mutex) {
  // NOTE: need to convert mutex
  return pthread_mutex_trylock(pthread_mutex_to_linux(mutex));
}

int $darwin$pthread_mutex_unlock($darwin$pthread_mutex_t *mutex) {
  // NOTE: need to convert mutex
  return pthread_mutex_unlock(pthread_mutex_to_linux(mutex));
}

int $darwin$pthread_attr_destroy($darwin$pthread_attr_t *attr) {
  // NOTE: size is good
  return pthread_attr_destroy(pthread_attr_to_linux(attr));
}

int $darwin$pthread_attr_getinheritsched($darwin$pthread_attr_t *attr, int *inherit) {
  // NOTE: size is good
  return pthread_attr_getinheritsched(pthread_attr_to_linux(attr), inherit);
}

int $darwin$pthread_attr_getschedparam($darwin$pthread_attr_t *attr, struct sched_param *param) {
  // NOTE: size is good, darwin 8, gnu 4
  return pthread_attr_getschedparam(pthread_attr_to_linux(attr), param);
}

int $darwin$pthread_attr_init($darwin$pthread_attr_t *attr) {
  // NOTE: size is good
  return pthread_attr_init(pthread_attr_to_linux(attr));
}

int $darwin$pthread_attr_setdetachstate($darwin$pthread_attr_t *attr, int detachstate) {
  // NOTE: size is good
  return pthread_attr_setdetachstate(pthread_attr_to_linux(attr), detachstate);
}

int $darwin$pthread_attr_setinheritsched($darwin$pthread_attr_t *attr, int inherit) {
  // NOTE: size is good
  return pthread_attr_setinheritsched(pthread_attr_to_linux(attr), inherit);
}

int $darwin$pthread_attr_setschedparam($darwin$pthread_attr_t *attr, const struct sched_param *param) {
  // NOTE: size is good, darwin 8, gnu 4
  return pthread_attr_setschedparam(pthread_attr_to_linux(attr), param);
}

int $darwin$pthread_attr_setschedpolicy($darwin$pthread_attr_t *attr, int policy) {
  // NOTE: size is good
  return pthread_attr_setschedpolicy(pthread_attr_to_linux(attr), policy);
}

int $darwin$pthread_attr_setstacksize($darwin$pthread_attr_t *attr, size_t stacksize) {
  // NOTE: size is good
  return pthread_attr_setstacksize(pthread_attr_to_linux(attr), stacksize);
}

int $darwin$pthread_condattr_destroy($darwin$pthread_condattr_t *attr) {
  // NOTE: does nothing
  return pthread_condattr_destroy(pthread_condattr_to_linux(attr));
}

int $darwin$pthread_condattr_init($darwin$pthread_condattr_t *attr) {
  // NOTE: size good
  return pthread_condattr_init(pthread_condattr_to_linux(attr));
}

int $darwin$pthread_cond_init($darwin$pthread_cond_t *cond, $darwin$pthread_condattr_t *attr) {
  // NOTE: size matches
  return pthread_cond_init(pthread_cond_to_linux(cond), pthread_condattr_to_linux(attr));
}

int $darwin$pthread_cond_broadcast($darwin$pthread_cond_t *cond) {
  // NOTE size matches
  return pthread_cond_broadcast(pthread_cond_to_linux(cond));
}

int $darwin$pthread_cond_destroy($darwin$pthread_cond_t *cond) {
  // NOTE: size matches
  return pthread_cond_destroy(pthread_cond_to_linux(cond));
}

int $darwin$pthread_cond_signal($darwin$pthread_cond_t *cond) {
  // NOTE: size matches
  return pthread_cond_signal(pthread_cond_to_linux(cond));
}

int $darwin$pthread_cond_timedwait($darwin$pthread_cond_t *cond, $darwin$pthread_mutex_t *mutex, const struct timespec *abstime) {
  // NOTE: size matches
  return pthread_cond_timedwait(pthread_cond_to_linux(cond), pthread_mutex_to_linux(mutex), abstime);
}

int $darwin$pthread_cond_wait($darwin$pthread_cond_t *cond, $darwin$pthread_mutex_t *mutex) {
  // NOTE: matches
  return pthread_cond_wait(pthread_cond_to_linux(cond), pthread_mutex_to_linux(mutex));
}

int $darwin$pthread_create(pthread_t *thread, $darwin$pthread_attr_t *attr, void *(*start_routine)(void *), void *arg) {
  // NOTE: size is good
  return pthread_create(thread, pthread_attr_to_linux(attr), start_routine, arg);
}

int $darwin$pthread_detach(pthread_t th) {
  // NOTE: good
  return pthread_detach(th);
}

int $darwin$pthread_equal(pthread_t thread1, pthread_t thread2) {
  // NOTE: just equality comparison
  return pthread_equal(thread1, thread2);
}

int $darwin$pthread_join(pthread_t threadid, void **thread_return) {
  // NOTE: looks fine?
  return pthread_join(threadid, thread_return);
}

pthread_t $darwin$pthread_self() {
  // NOTE: ez
  return pthread_self();
}

int $darwin$pthread_setname_np(const char *name) {
  // NOTE darwin only allows self thread name set ???
  return pthread_setname_np(pthread_self(), name);
}

int $darwin$pthread_threadid_np(pthread_t thread, uint64_t *thread_id) {
  // NOTE: should be fine?
  if (!thread) return EINVAL;
  *thread_id = thread;
  return 0;
}

int $darwin$pthread_key_create(pthread_key_t *key, void (*destr)(void *)) {
  // NOTE: looks fine
  return pthread_key_create(key, destr);
}

int $darwin$pthread_key_delete(pthread_key_t key) {
  // NOTE:
  return pthread_key_delete(key);
}

void *$darwin$pthread_getspecific(pthread_key_t key) {
  // NOTE:
  return pthread_getspecific(key);
}

int $darwin$pthread_setspecific(pthread_key_t key, const void *value) {
  // NOTE: nothing to mesup here
  return pthread_setspecific(key, value);
}

int $darwin$pthread_once($darwin$pthread_once_t *once_control, void (*init_routine)(void)) {
  // NOTE: size is good.
  return pthread_once(once_control->native, init_routine);
}

int $darwin$pthread_rwlock_init($darwin$pthread_rwlock_t *rwlock, /*const*/ $darwin$pthread_rwlockattr_t *attr) {
  // NOTE: size is good
  return pthread_rwlock_init(pthread_rwlock_to_linux(rwlock), pthread_rwlockattr_to_linux(attr));
}

int $darwin$pthread_rwlock_destroy($darwin$pthread_rwlock_t *rwlock) {
  // NOTE: this is noop anyways
  return pthread_rwlock_destroy(pthread_rwlock_to_linux(rwlock));
}

int $darwin$pthread_rwlock_rdlock($darwin$pthread_rwlock_t *rwlock) {
  // NOTE: size is good
  return pthread_rwlock_rdlock(pthread_rwlock_to_linux(rwlock));
}

int $darwin$pthread_rwlock_unlock($darwin$pthread_rwlock_t *rwlock) {
  // NOTE: size is good
  return pthread_rwlock_unlock(pthread_rwlock_to_linux(rwlock));
}

int $darwin$pthread_rwlock_wrlock($darwin$pthread_rwlock_t *rwlock) {
  // NOTE: size is good
  return pthread_rwlock_wrlock(pthread_rwlock_to_linux(rwlock));
}

int $darwin$sched_get_priority_max(int policy) {
  // NOTE: darwin ignores policy
  return sched_get_priority_max(policy);
}

int $darwin$sched_get_priority_min(int policy) {
  // NOTE: darwin ignores policy
  return sched_get_priority_min(policy);
}

// C++ support

// _ZNSt3__15mutex
int $darwin$_ZNSt3__15mutex4lockEv($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_lock(m); }

int $darwin$_ZNSt3__15mutex8try_lockEv($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_trylock(m); }

int $darwin$_ZNSt3__15mutex6unlockEv($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_unlock(m); }

int $darwin$_ZNSt3__15mutexD2Ev($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_destroy(m); }

int $darwin$_ZNSt3__15mutexD1Ev($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_destroy(m); }

// _ZNSt3__115recursive_mutex
void $darwin$_ZNSt3__115recursive_mutexC2Ev($darwin$pthread_mutex_t *m) { panic("_ZNSt3__115recursive_mutexC2Ev"); }

void $darwin$_ZNSt3__115recursive_mutexC1Ev($darwin$pthread_mutex_t *m) { panic("_ZNSt3__115recursive_mutexC1Ev"); }

int $darwin$_ZNSt3__115recursive_mutex4lockEv($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_lock(m); }

int $darwin$_ZNSt3__115recursive_mutex8try_lockEv($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_trylock(m); }

int $darwin$_ZNSt3__115recursive_mutex6unlockEv($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_unlock(m); }

int $darwin$_ZNSt3__115recursive_mutexD2Ev($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_destroy(m); }

int $darwin$_ZNSt3__115recursive_mutexD1Ev($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_destroy(m); }

// _ZNSt3__121recursive_timed_mutex
void $darwin$_ZNSt3__121recursive_timed_mutexC2Ev($darwin$pthread_mutex_t *m) { *m = ($darwin$pthread_mutex_t){0x32AAABA7}; }

void $darwin$_ZNSt3__121recursive_timed_mutexC1Ev($darwin$pthread_mutex_t *m) { *m = ($darwin$pthread_mutex_t){0x32AAABA7}; }

int $darwin$_ZNSt3__121recursive_timed_mutex4lockEv($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_lock(m); }

int $darwin$_ZNSt3__121recursive_timed_mutex8try_lockEv($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_trylock(m); }

int $darwin$_ZNSt3__121recursive_timed_mutex6unlockEv($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_unlock(m); }

int $darwin$_ZNSt3__121recursive_timed_mutexD2Ev($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_destroy(m); }

int $darwin$_ZNSt3__121recursive_timed_mutexD1Ev($darwin$pthread_mutex_t *m) { return $darwin$pthread_mutex_destroy(m); }

// _ZNSt3__16thread
int $darwin$_ZNSt3__16thread4joinEv(pthread_t *t) { return $darwin$pthread_join(*t, NULL); }

int $darwin$_ZNSt3__16thread6detachEv(pthread_t *t) { return $darwin$pthread_detach(*t); }

void $darwin$_ZNSt3__16threadD2Ev(pthread_t *t) {
  if (*t) exit(0);
}

void $darwin$_ZNSt3__16threadD1Ev(pthread_t *t) {
  if (*t) exit(0);
}

// _ZNSt3__118condition_variable
int $darwin$_ZNSt3__118condition_variable10notify_allEv($darwin$pthread_cond_t *cond) { return $darwin$pthread_cond_broadcast(cond); }

int $darwin$_ZNSt3__118condition_variable10notify_oneEv($darwin$pthread_cond_t *cond) { return $darwin$pthread_cond_signal(cond); }

int $darwin$_ZNSt3__118condition_variable15__do_timed_waitERNS_11unique_lockINS_5mutexEEENS_6chrono10time_pointINS5_12system_clockENS5_8durationIxNS_5ratioILl1ELl1000000000EEEEEEE($darwin$pthread_cond_t *cond, $darwin$pthread_mutex_t** m, long long t) {
  t = t <= 0x59682F000000E941LL ? t : 0x59682F000000E941LL;
  struct timespec ts;
  ts.tv_sec = t / 1000000000;
  ts.tv_nsec = t % 1000000000;
  return $darwin$pthread_cond_timedwait(cond, *m, &ts);
}

int $darwin$_ZNSt3__118condition_variable4waitERNS_11unique_lockINS_5mutexEEE($darwin$pthread_cond_t *cond, $darwin$pthread_mutex_t** m) {
  return $darwin$pthread_cond_wait(cond, *m);
}

int $darwin$_ZNSt3__118condition_variableD2Ev($darwin$pthread_cond_t *cond) { return $darwin$pthread_cond_destroy(cond); }

int $darwin$_ZNSt3__118condition_variableD1Ev($darwin$pthread_cond_t *cond) { return $darwin$pthread_cond_destroy(cond); }
