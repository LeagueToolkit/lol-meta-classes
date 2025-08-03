#include <common.h>
#include <pwd.h>

struct $darwin$passwd {
  char *pw_name;
  char *pw_passwd;
  uid_t pw_uid;
  gid_t pw_gid;
  time_t pw_change;
  char *pw_class;
  char *pw_gecos;
  char *pw_dir;
  char *pw_shell;
  time_t pw_expire;
};

struct $darwin$passwd *$darwin$getpwuid(uid_t uid) {
  // TODO
  panic("getpwuid");
  static struct $darwin$passwd lazy = {0};
  static int inited = 0;
  if (inited == 0) {
    struct passwd *gnu = getpwuid(uid);
    if (gnu == NULL) return NULL;
    lazy.pw_name = gnu->pw_name;
    lazy.pw_passwd = gnu->pw_passwd;
    lazy.pw_uid = gnu->pw_uid;
    lazy.pw_gid = gnu->pw_gid;
    lazy.pw_gecos = gnu->pw_gecos;
    lazy.pw_dir = gnu->pw_dir;
    lazy.pw_shell = gnu->pw_shell;
    inited = 1;
  }
  return &lazy;
}
