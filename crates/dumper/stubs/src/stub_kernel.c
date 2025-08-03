#include <errno.h>
#include <fcntl.h>
#include <linux/limits.h>
#include <pthread.h>
#include <semaphore.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <time.h>

#include <common.h>

typedef struct mach_timebase_info {
  uint32_t numer;
  uint32_t denom;
} *mach_timebase_info_t;

typedef int clock_res_t;

typedef struct mach_timespec {
  unsigned int tv_sec;
  unsigned int tv_nsec;
} mach_timespec_t;

typedef int kern_return_t;

typedef intptr_t task_t;

typedef uint32_t semaphore_t;

// NOTE: used by crashpad, don't really care?
// FIXME: better definition
intptr_t $darwin$NDR_record = 0;

task_t $darwin$mach_task_self_ = -1;

// TODO: make a somewhat better stack protector?
long $darwin$__stack_chk_guard[8] = {0, 0, 0, 0, 0, 0, 0, 0};

int $darwin$Gestalt(int select, int32_t *response) {
  // NOTE: used for version reporting
  // TODO: implement 'sys1', 'sys2', 'sys3'
  return 0;
}

int $darwin$__darwin_check_fd_set_overflow(int n, const void *fd_set, int unlimited_select) {
  if (n >= FD_SETSIZE) {
    // FIXME: actually scan stack?
    // panic("__darwin_check_fd_set_overflow");
  }
  return 1;
}

int *$darwin$__error() {
  // NOTE: this is correct
  return __errno_location();
}

void $darwin$bootstrap_port() {
  // TODO
  panic("bootstrap_port");
}

void $darwin$host_get_exception_ports() {
  // TODO
  panic("host_get_exception_ports");
}

void $darwin$host_set_exception_ports() {
  // TODO
  panic("host_set_exception_ports");
}

int $darwin$host_statistics(void *host_priv, int flavor, void *host_info_out, unsigned *host_info_outCnt) {
  // TODO
  return KERN_SUCCESS;
}

void $darwin$host_swap_exception_ports() {
  // TODO
  panic("host_swap_exception_ports");
}

void $darwin$mach_error_string() {
  // TODO
  panic("mach_error_string");
}

intptr_t $darwin$mach_host_self() {
  // I dont care really.
  return 1;
}

void $darwin$mach_msg() {
  // TODO
  panic("mach_msg");
}

void $darwin$mach_msg_destroy() {
  // TODO
  panic("mach_msg_destroy");
}

void $darwin$mach_port_allocate() {
  // TODO
  panic("mach_port_allocate");
}

void $darwin$mach_port_deallocate() {
  // TODO
  panic("mach_port_deallocate");
}

void $darwin$mach_port_extract_right() {
  // TODO
  panic("mach_port_extract_right");
}

void $darwin$mach_port_mod_refs() {
  // TODO
  panic("mach_port_mod_refs");
}

void $darwin$mach_port_request_notification() {
  // TODO
  panic("mach_port_request_notification");
}

void $darwin$mach_thread_self() {
  // TODO
  panic("mach_thread_self");
}

int $darwin$mach_timebase_info(mach_timebase_info_t info) {
  if (!info) return KERN_INVALID_ARGUMENT;
  // NOTE: pretend that absolute time is allways in nanoseconds
  info->numer = 1;
  info->denom = 1;
  return KERN_SUCCESS;
}

uint64_t $darwin$mach_absolute_time() {
  // return clock();
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return ts.tv_nsec + ts.tv_sec * 1000000000;
}

#define SEM_TABLE_MAX 4096
// FIXME: replace lock with atomic
static pthread_mutex_t _sem_table_lock = PTHREAD_MUTEX_INITIALIZER;
static sem_t _sem[SEM_TABLE_MAX] = {0};
static _Bool _sem_bits[SEM_TABLE_MAX] = {0};

__attribute__((constructor)) static void _sem_lock_init() { pthread_mutex_init(&_sem_table_lock, NULL); }

kern_return_t $darwin$semaphore_create(task_t task, semaphore_t *semaphore, int policy, int value) {
  if (!semaphore) return KERN_INVALID_ARGUMENT;
  pthread_mutex_lock(&_sem_table_lock);
  for (int i = 1; i < SEM_TABLE_MAX; ++i) {
    if (!_sem_bits[i]) {
      int result = sem_init(&_sem[i], 0, value);
      if (!result) {
        pthread_mutex_unlock(&_sem_table_lock);
        _sem_bits[i] = 1;
        *semaphore = i;
        return KERN_SUCCESS;
      }
      break;
    }
  }
  pthread_mutex_unlock(&_sem_table_lock);
  return KERN_NOT_SUPPORTED;
}

kern_return_t $darwin$semaphore_destroy(task_t task, semaphore_t semaphore) {
  if (!semaphore) return KERN_SUCCESS;
  if (semaphore < 1 || semaphore >= SEM_TABLE_MAX || !_sem_bits[semaphore]) return KERN_SEMAPHORE_DESTROYED;

  pthread_mutex_lock(&_sem_table_lock);
  sem_close(&_sem[semaphore]);
  _sem_bits[semaphore] = 0;
  pthread_mutex_unlock(&_sem_table_lock);
  return KERN_SUCCESS;
}

static kern_return_t sem_convert_result(int result) {
  if (result == 0) return KERN_SUCCESS;
  result = errno;
  switch (result) {
    case 0:
      return KERN_SUCCESS;
    case EACCES:
      return KERN_PROTECTION_FAILURE;
    case ETIMEDOUT:
      return KERN_OPERATION_TIMED_OUT;
    case EINTR:
      return KERN_ABORTED;
    case EINVAL:
      return KERN_INVALID_ARGUMENT;
    case EAGAIN:
      return KERN_ABORTED;
    default:
      return KERN_NOT_SUPPORTED;
  }
}

kern_return_t $darwin$semaphore_signal(semaphore_t semaphore) {
  if (semaphore < 1 || semaphore >= SEM_TABLE_MAX || !_sem_bits[semaphore]) {
    panic("bad!");
    return KERN_SEMAPHORE_DESTROYED;
  }
  int result = sem_post(&_sem[semaphore]);
  return sem_convert_result(result);
}

kern_return_t $darwin$semaphore_wait(semaphore_t semaphore) {
  if (semaphore < 1 || semaphore >= SEM_TABLE_MAX || !_sem_bits[semaphore]) {
    panic("bad!");
    return KERN_SEMAPHORE_DESTROYED;
  }
  int result = sem_wait(&_sem[semaphore]);
  return sem_convert_result(result);
}

kern_return_t $darwin$semaphore_timedwait(semaphore_t semaphore, mach_timespec_t wait_time) {
  if (semaphore < 1 || semaphore >= SEM_TABLE_MAX || !_sem_bits[semaphore]) {
    panic("bad!");
    return KERN_SEMAPHORE_DESTROYED;
  }

  // TODO: check 0 case
  struct timespec ts;
  clock_gettime(CLOCK_REALTIME, &ts);
  ts.tv_sec += wait_time.tv_sec;
  ts.tv_nsec += wait_time.tv_nsec;
  if (ts.tv_nsec >= 1000000000) {
    ts.tv_sec += ts.tv_nsec / 1000000000;
    ts.tv_nsec %= 1000000000;
  }
  int result = sem_timedwait(&_sem[semaphore], &ts);
  return sem_convert_result(result);
}

void $darwin$task_get_exception_ports() {
  // TODO
  panic("task_get_exception_ports");
}

int $darwin$task_info(intptr_t target_task, unsigned flavor, void *task_info_out, unsigned *task_info_outCn) {
  if (flavor == 22) {
    // used to log hardware statistics, i do not really care
    memset(task_info_out, 0, *task_info_outCn);
    return KERN_SUCCESS;
  }
  // TODO
  panic("task_info");
}

void $darwin$task_set_exception_ports() {
  // TODO
  panic("task_set_exception_ports");
}

void $darwin$task_swap_exception_ports() {
  // TODO
  panic("task_swap_exception_ports");
}

void $darwin$thread_get_exception_ports() {
  // TODO
  panic("thread_get_exception_ports");
}

void $darwin$thread_set_exception_ports() {
  // TODO
  panic("thread_set_exception_ports");
}

void $darwin$thread_swap_exception_ports() {
  // TODO
  panic("thread_swap_exception_ports");
}

void $darwin$vm_allocate() {
  // TODO
  panic("vm_allocate");
}

void $darwin$vm_deallocate() {
  // TODO
  panic("vm_deallocate");
}

void $darwin$vm_page_size() {
  // TODO
  panic("vm_page_size");
}

void $darwin$voucher_mach_msg_set() {
  // TODO
  panic("voucher_mach_msg_set");
}
