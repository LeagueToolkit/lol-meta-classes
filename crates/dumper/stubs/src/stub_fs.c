#define __USE_LARGEFILE64 1
#define _GNU_SOURCE 1
#include <dirent.h>
#include <fcntl.h>
#include <glob.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#include <common.h>

typedef int32_t $darwin$dev_t;
typedef uint16_t $darwin$mode_t;
typedef uint16_t $darwin$nlink_t;
typedef uint64_t $darwin$ino64_t;
typedef uint32_t $darwin$uid_t;
typedef uint32_t $darwin$gid_t;
typedef uint32_t $darwin$sigset_t;
typedef struct timespec $darwin$timespec;

struct $darwin$dirent64 {
  uint64_t d_ino;
  uint64_t d_seekoff;
  uint16_t d_reclen;
  uint16_t d_namlen;
  uint8_t d_type;
  char d_name[1024];
};

struct $darwin$DIR {
  void *real;
  struct $darwin$dirent64 tmp;
};

struct $darwin$stat64 {
  $darwin$dev_t st_dev;
  $darwin$mode_t st_mode;
  $darwin$nlink_t st_nlink;
  $darwin$ino64_t st_ino;
  $darwin$uid_t st_uid;
  $darwin$gid_t st_gid;
  $darwin$dev_t st_rdev;
  $darwin$timespec st_atimespec;
  $darwin$timespec st_mtimespec;
  $darwin$timespec st_ctimespec;
  $darwin$timespec st_birthtimespec;
  off_t st_size;
  blkcnt_t st_blocks;
  blksize_t st_blksize;
  uint32_t st_flags;
  uint32_t st_gen;
  int32_t st_lspare;
};

static void stat64_from_linux(struct $darwin$stat64 *into, struct stat64 *from) {
  into->st_dev = ($darwin$dev_t)from->st_dev;
  into->st_mode = ($darwin$mode_t)from->st_mode;
  into->st_nlink = ($darwin$nlink_t)from->st_nlink;
  into->st_ino = ($darwin$ino64_t)from->st_ino;
  into->st_uid = ($darwin$uid_t)from->st_uid;
  into->st_gid = ($darwin$gid_t)from->st_gid;
  into->st_rdev = ($darwin$dev_t)from->st_rdev;
  into->st_atimespec = from->st_atim;
  into->st_mtimespec = from->st_mtim;
  into->st_ctimespec = from->st_ctim;
  into->st_size = from->st_size;
  into->st_blocks = from->st_blocks;
  into->st_blksize = from->st_blksize;
}

static void dirent64_from_linux(struct $darwin$dirent64 *into, struct dirent64 *from) {
  into->d_ino = from->d_ino;
  into->d_reclen = from->d_reclen;
  into->d_type = from->d_type;
  into->d_namlen = strlen(from->d_name);
  into->d_seekoff = 0;
  strcpy(into->d_name, from->d_name);
}

// FIXME: glob$INODE64
int $darwin$glob$INODE64(const char *pattern, int flags, int (*errfunc)(const char *, int), glob64_t *pglob) { return glob64(pattern, flags, errfunc, pglob); }

// TODO: provide more complete filesystem emulation/overlay ?
struct $darwin$DIR *$darwin$opendir$INODE64(const char *name) {
  DIR *real = opendir(name);
  if (!real) return NULL;

  struct $darwin$DIR *stream = (struct $darwin$DIR *)malloc(sizeof(struct $darwin$DIR));
  stream->real = real;
  return stream;
}

int $darwin$closedir(struct $darwin$DIR *stream) {
  if (!stream) return 0;
  int e = closedir((DIR *)stream->real);
  if (e) return e;
  free(stream);
  return 0;
}

struct $darwin$dirent64 *$darwin$readdir$INODE64(struct $darwin$DIR *dirp) {
  struct dirent64 *from = readdir64((DIR *)dirp->real);
  if (!from) return NULL;
  dirent64_from_linux(&dirp->tmp, from);
  return &dirp->tmp;
}

int $darwin$stat$INODE64(char const *path, struct $darwin$stat64 *s) {
  if (!s) return stat64(path, (struct stat64 *)s);
  struct stat64 tmp;
  int ret = stat64(path, &tmp);
  stat64_from_linux(s, &tmp);
  return ret;
}

int $darwin$lstat$INODE64(char const *path, struct $darwin$stat64 *s) {
  if (!s) return lstat64(path, (struct stat64 *)s);
  struct stat64 tmp;
  int ret = lstat64(path, &tmp);
  stat64_from_linux(s, &tmp);
  return ret;
}

int $darwin$fstat$INODE64(int fd, struct $darwin$stat64 *s) {
  if (!s) return fstat64(fd, (struct stat64 *)s);
  struct stat64 tmp;
  int ret = fstat64(fd, &tmp);
  stat64_from_linux(s, &tmp);
  return ret;
}

char *$darwin$realpath$DARWIN_EXTSN(const char *name, char *resolved_name) {
  // FIXME: Defining _DARWIN_C_SOURCE or _DARWIN_BETTER_REALPATH before including stdlib.h
  /// will cause the provided implementation of realpath() to use F_GETPATH from fcntl(2) to discover the path.
  return realpath(name, resolved_name);
}

#include <unistd.h>
// FILE* $darwin$fopen(char const* path, char const* mode) {
//   char dir[PATH_MAX];
//   getcwd(dir, sizeof(dir));
//   printf("opening %s with %s from %s\n", path, mode, dir);
//   return fopen(path, mode);
// }

int $darwin$chdir(const char* path) {
  // workaround stupidity
  if (strcmp(path, ".") == 0 || strcmp(path, "../../..") == 0)
    return 0;
  return chdir(path);
}
