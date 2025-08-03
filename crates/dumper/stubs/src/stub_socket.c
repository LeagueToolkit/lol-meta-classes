#include <arpa/inet.h>
#include <bits/sockaddr.h>
#include <errno.h>
#include <ifaddrs.h>
#include <net/if.h>
#include <netdb.h>
#include <netinet/in.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <time.h>

#include <common.h>

#define $darwin$SOL_SOCKET 0xFFFF
#define $darwin$AF_UNSPEC 0
#define $darwin$AF_UNIX 1
#define $darwin$AF_INET 2
#define $darwin$AF_SNA 11
#define $darwin$AF_DECnet 12
#define $darwin$AF_APPLETALK 16
#define $darwin$AF_ROUTE 17
#define $darwin$AF_IPX 23
#define $darwin$AF_ISDN 28
#define $darwin$AF_INET6 30

struct $darwin$sockaddr {
  uint8_t sa_len;
  uint8_t sa_family;
  char sa_data[14];
};

struct $darwin$sockaddr_in {
  uint8_t sin_len;
  uint8_t sin_family;
  uint16_t sin_port;
  uint32_t sin_addr;
  char sin_zero[8];
};

struct $darwin$sockaddr_in6 {
  uint8_t sin6_len;
  uint8_t sin6_family;
  uint16_t sin6_port;
  uint32_t sin6_flowinfo;
  uint32_t sin6_addr[4];
  uint32_t sin6_scope_id;
};

struct $darwin$addrinfo {
  int ai_flags;
  int ai_family;
  int ai_socktype;
  int ai_protocol;
  uint32_t ai_addrlen;
  char *ai_canonname;
  struct sockaddr *ai_addr;
  struct $darwin$addrinfo *ai_next;
};

struct $darwin$ifaddrs {
  struct $darwin$ifaddrs *ifa_next;
  char *ifa_name;
  uint32_t ifa_flags;
  struct sockaddr *ifa_addr;
  struct sockaddr *ifa_netmask;
  struct sockaddr *ifa_dstaddr_;
  void *ifa_data;
};

// TODO: In many cases we are transforming layout of structs in place, which is very error prone.
//       Instead it would make more sense to copy data to a temporary or new structure.

static unsigned sa_family_to_gnu(unsigned a) {
  switch (a) {
    case $darwin$AF_UNSPEC:
      return AF_UNSPEC;
    case $darwin$AF_UNIX:
      return AF_UNIX;
    case $darwin$AF_INET:
      return AF_INET;
    case $darwin$AF_SNA:
      return AF_SNA;
    case $darwin$AF_DECnet:
      return AF_DECnet;
    case $darwin$AF_APPLETALK:
      return AF_APPLETALK;
    case $darwin$AF_ROUTE:
      return AF_ROUTE;
    case $darwin$AF_IPX:
      return AF_IPX;
    case $darwin$AF_ISDN:
      return AF_ISDN;
    case $darwin$AF_INET6:
      return AF_INET6;
    default:
      return a;
  }
}

static unsigned sa_family_to_darwin(unsigned a) {
  switch (a) {
    case AF_UNSPEC:
      return $darwin$AF_UNSPEC;
    case AF_UNIX:
      return $darwin$AF_UNIX;
    case AF_INET:
      return $darwin$AF_INET;
    case AF_SNA:
      return $darwin$AF_SNA;
    case AF_DECnet:
      return $darwin$AF_DECnet;
    case AF_APPLETALK:
      return $darwin$AF_APPLETALK;
    case AF_ROUTE:
      return $darwin$AF_ROUTE;
    case AF_IPX:
      return $darwin$AF_IPX;
    case AF_ISDN:
      return $darwin$AF_ISDN;
    case AF_INET6:
      return $darwin$AF_INET6;
    default:
      return a;
  }
}

static void sockaddr_to_gnu(struct sockaddr *a) {
  if (a == NULL) return;
  struct $darwin$sockaddr *darwin = (struct $darwin$sockaddr *)a;
  uint8_t tmp[2] = {sa_family_to_gnu(darwin->sa_family), 0};
  memcpy(a, tmp, 2);
}

static void sockaddr_to_darwin(struct sockaddr *a) {
  if (a == NULL) return;
  uint8_t tmp[2] = {0, (uint8_t)sa_family_to_darwin(a->sa_family)};
  if (tmp[1] == $darwin$AF_INET6) {
    tmp[0] = sizeof(struct $darwin$sockaddr_in6);
  } else {
    tmp[0] = sizeof(struct $darwin$sockaddr_in);
  }
  memcpy(a, tmp, 2);
}

static void addrinfo_to_gnu(struct addrinfo *a) {
  struct $darwin$addrinfo *i = (struct $darwin$addrinfo *)a;
  while (i) {
    struct $darwin$addrinfo darwin;
    memcpy(&darwin, i, sizeof(struct $darwin$addrinfo));
    struct addrinfo gnu;
    gnu.ai_family = sa_family_to_gnu(darwin.ai_family);
    gnu.ai_socktype = darwin.ai_socktype;
    gnu.ai_protocol = darwin.ai_protocol;
    gnu.ai_flags = darwin.ai_flags;
    gnu.ai_addrlen = darwin.ai_addrlen;
    gnu.ai_canonname = darwin.ai_canonname;
    gnu.ai_addr = darwin.ai_addr;
    gnu.ai_next = (struct addrinfo *)darwin.ai_next;
    sockaddr_to_gnu(gnu.ai_addr);
    memcpy(i, &gnu, sizeof(struct addrinfo));
    i = darwin.ai_next;
  }
}

static void addrinfo_to_darwin(struct addrinfo *a) {
  struct addrinfo *i = a;
  while (i) {
    struct addrinfo gnu;
    memcpy(&gnu, i, sizeof(struct addrinfo));
    struct $darwin$addrinfo darwin;
    darwin.ai_family = sa_family_to_darwin(gnu.ai_family);
    darwin.ai_socktype = gnu.ai_socktype;
    darwin.ai_protocol = gnu.ai_protocol;
    darwin.ai_flags = gnu.ai_flags;
    darwin.ai_addrlen = gnu.ai_addrlen;
    darwin.ai_canonname = gnu.ai_canonname;
    darwin.ai_addr = gnu.ai_addr;
    darwin.ai_next = (struct $darwin$addrinfo *)gnu.ai_next;
    sockaddr_to_darwin(darwin.ai_addr);
    memcpy(i, &darwin, sizeof(struct addrinfo));
    i = gnu.ai_next;
  }
}

static void ifaddrs_to_gnu(struct ifaddrs *a) {
  struct $darwin$ifaddrs *i = (struct $darwin$ifaddrs *)a;
  while (i) {
    sockaddr_to_gnu(i->ifa_addr);
    sockaddr_to_gnu(i->ifa_netmask);
    sockaddr_to_gnu(i->ifa_dstaddr_);
    i = i->ifa_next;
  }
}

static void ifaddrs_to_darwin(struct ifaddrs *a) {
  struct ifaddrs *i = (struct ifaddrs *)a;
  while (i) {
    sockaddr_to_darwin(i->ifa_addr);
    sockaddr_to_darwin(i->ifa_netmask);
    sockaddr_to_darwin((struct sockaddr *)&i->ifa_ifu);
    i = i->ifa_next;
  }
}

int $darwin$inet_pton(int af, const char *src, void *dst) { return inet_pton(sa_family_to_gnu(af), src, dst); }

const char *$darwin$inet_ntop(int af, const void *cp, char *buf, socklen_t len) { return inet_ntop(sa_family_to_gnu(af), cp, buf, len); }

int $darwin$bind(int fd, const struct $darwin$sockaddr *addr, socklen_t len) {
  char tmp[256];
  memcpy(tmp, addr, len);
  sockaddr_to_gnu((struct sockaddr *)tmp);
  int result = bind(fd, (struct sockaddr *)tmp, len);
  return result;
}

int $darwin$connect(int fd, const struct $darwin$sockaddr *addr, socklen_t len) {
  char tmp[256];
  memcpy(tmp, addr, len);
  sockaddr_to_gnu((struct sockaddr *)tmp);
  int result = connect(fd, (struct sockaddr *)tmp, len);
  return result;
}

ssize_t $darwin$sendto(int fd, const void *buf, size_t n, int flags, const struct $darwin$sockaddr *addr, socklen_t len) {
  char tmp[256];
  memcpy(tmp, addr, len);
  sockaddr_to_gnu((struct sockaddr *)tmp);
  return sendto(fd, buf, n, flags, (struct sockaddr *)tmp, len);
}

int $darwin$accept(int fd, struct sockaddr *addr, socklen_t *len) {
  char tmp[256];
  int result = accept(fd, (struct sockaddr *)tmp, len);
  sockaddr_to_darwin((struct sockaddr *)tmp);
  if (addr) memcpy(addr, tmp, *len);
  return result;
}

ssize_t $darwin$recvfrom(int fd, void *buf, size_t n, int flags, struct sockaddr *addr, socklen_t *len) {
  char tmp[256];
  ssize_t result = recvfrom(fd, buf, n, flags, (struct sockaddr *)tmp, len);
  sockaddr_to_darwin((struct sockaddr *)tmp);
  if (addr) memcpy(addr, tmp, *len);
  return result;
}

int $darwin$getsockname(int fd, struct $darwin$sockaddr *addr, socklen_t *len) {
  char tmp[256];
  int result = getsockname(fd, (struct sockaddr *)tmp, len);
  sockaddr_to_darwin((struct sockaddr *)tmp);
  if (addr) memcpy(addr, tmp, *len);
  return result;
}

int $darwin$getpeername(int fd, struct $darwin$sockaddr *addr, socklen_t *len) {
  char tmp[256];
  int result = getpeername(fd, (struct sockaddr *)tmp, len);
  sockaddr_to_darwin((struct sockaddr *)tmp);
  if (addr) memcpy(addr, tmp, *len);
  return result;
}

void $darwin$freeaddrinfo(struct addrinfo *res) {
  addrinfo_to_gnu(res);
  freeaddrinfo(res);
}

int $darwin$getaddrinfo(const char *nodename, const char *servname, struct addrinfo *hints, struct addrinfo **res) {
  // TODO: allocate new block here instead of transforming layout.
  addrinfo_to_gnu(hints);
  int result = getaddrinfo(nodename, servname, hints, res);
  addrinfo_to_darwin(hints);
  if (result == 0) addrinfo_to_darwin(*res);
  return result;
}

void $darwin$freeifaddrs(struct ifaddrs *ifap) {
  // ifaddrs_to_gnu(ifap);
  freeifaddrs(ifap);
}

int $darwin$getifaddrs(struct ifaddrs **ifap) {
  // NOTE: layout seems to match.
  struct ifaddrs *tmp = NULL;
  int result = getifaddrs(&tmp);
  if (result == 0) {
    ifaddrs_to_darwin(tmp);
    *ifap = tmp;
  }
  return result;
}

int $darwin$getnameinfo(const struct $darwin$sockaddr *sa, socklen_t salen, char *host, size_t hostlen, char *serv, size_t servlen, int flags) {
  char tmp[256];
  memcpy(tmp, sa, salen);
  sockaddr_to_gnu((struct sockaddr *)tmp);
  return getnameinfo((struct sockaddr *)tmp, salen, host, hostlen, serv, servlen, flags);
}

unsigned int $darwin$if_nametoindex(const char *ifname) {
  // NOTE: args match
  return if_nametoindex(ifname);
}

static int to_darwin_optname(int optname, char const *reason) {
  switch (optname) {
    case 1:
      return SO_DEBUG;
    case 2:
      return SO_ACCEPTCONN;
    case 4:
      return SO_REUSEADDR;
    case 8:
      return SO_KEEPALIVE;
    case 16:
      return SO_DONTROUTE;
    case 27:
      return -1;
    case 32:
      return SO_BROADCAST;
    case 257:
      return -1;
    case 512:
      return SO_REUSEPORT;
    case 0x1007:
      return SO_ERROR;
    case 0x1022:
      return -1;  // SO_NOSIGPIPE
      // 27, 257, 512, 4103, 4130
  }
  panic("%s %d\n", reason, optname);
}

int $darwin$getsockopt(int fd, int level, int optname, void *optval, socklen_t *optlen) {
  if (level == $darwin$SOL_SOCKET) level = SOL_SOCKET;
  optname = to_darwin_optname(optname, __func__);
  if (optname == -1) return 0;  // ignore
  return getsockopt(fd, level, optname, optval, optlen);
}

int $darwin$setsockopt(int fd, int level, int optname, const void *optval, socklen_t optlen) {
  if (level == $darwin$SOL_SOCKET) level = SOL_SOCKET;
  optname = to_darwin_optname(optname, __func__);
  if (optname == -1) return 0;  // ignore
  return setsockopt(fd, level, optname, optval, optlen);
}
