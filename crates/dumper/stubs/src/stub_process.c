#include <common.h>

#define $darwin$MAXPATHLEN 1024

struct $darwin$proc_fileinfo {
  uint32_t fi_openflags;
  uint32_t fi_status;
  off_t fi_offset;
  int32_t fi_type;
  uint32_t fi_guardflags;
};

struct $darwin$vinfo_stat {
  uint32_t vst_dev;
  uint16_t vst_mode;
  uint16_t vst_nlink;
  uint64_t vst_ino;
  uid_t vst_uid;
  gid_t vst_gid;
  int64_t vst_atime;
  int64_t vst_atimensec;
  int64_t vst_mtime;
  int64_t vst_mtimensec;
  int64_t vst_ctime;
  int64_t vst_ctimensec;
  int64_t vst_birthtime;
  int64_t vst_birthtimensec;
  off_t vst_size;
  int64_t vst_blocks;
  int32_t vst_blksize;
  uint32_t vst_flags;
  uint32_t vst_gen;
  uint32_t vst_rdev;
  int64_t vst_qspare[2];
};

struct $darwin$psem_info {
  struct $darwin$vinfo_stat psem_stat;
  char psem_name[$darwin$MAXPATHLEN];
};

struct $darwin$psem_fdinfo {
  struct $darwin$proc_fileinfo pfi;
  struct $darwin$psem_info pseminfo;
};

#define PROC_PIDFDPSEMINFO 4
#define PROC_PIDFDPSEMINFO_SIZE (sizeof(struct $darwin$psem_fdinfo), 1184)

void $darwin$proc_listpidspath() {
  // TODO
  panic("proc_listpidspath");
}

int $darwin$proc_pidinfo(int pid, int flavor, int fd, void* buffer, uint32_t buffersize, int32_t* retval) {
  if (flavor == 4) {
    return 0;
  }
  // TODO
  panic("proc_pidinfo");
}

void $darwin$proc_pidpath() {
  // TODO
  panic("proc_pidpath");
}

__pid_t $darwin$fork() {
  panic("fork not supported!");
  return -1;
}

int $darwin$posix_spawn_file_actions_addinherit_np(void* file_actions, int filedes) {
  // NOTE: used by crashpad, don't care
  panic("posix_spawn_file_actions_addinherit_np");
  return 0;
}

int $darwin$posix_spawn_file_actions_destroy() {
  // NOTE: used by crashpad, don't care
  panic("posix_spawn_file_actions_destroy");
  return 0;
}

int $darwin$posix_spawn_file_actions_init() {
  // NOTE: used by crashpad, don't care
  panic("posix_spawn_file_actions_init");
  return 0;
}
