#include <stdarg.h>
#include <fcntl.h>

int sys_open(const char* filename, int flags, mode_t mode);

int open(const char* filename, int flags, ...) {
    mode_t mode = 0;
    va_list ap;
    va_start(ap, flags);
    mode = va_arg(ap, mode_t);
    va_end(ap);
    return sys_open(filename, flags, mode);
}

int sys_open64(const char* filename, int flags, mode_t mode);

int open64(const char* filename, int flags, ...) {
    mode_t mode = 0;
    va_list ap;
    va_start(ap, flags);
    mode = va_arg(ap, mode_t);
    va_end(ap);
    return sys_open(filename, flags, mode);
}

int sys_openat(int dirfd, const char* filename, int flags, mode_t mode);

int openat(int dirfd, const char* filename, int flags, ...) {
    mode_t mode = 0;
    va_list ap;
    va_start(ap, flags);
    mode = va_arg(ap, mode_t);
    va_end(ap);
    return sys_openat(dirfd, filename, flags, mode);
}

int sys_openat64(int dirfd, const char* filename, int flags, mode_t mode);

int openat64(int dirfd, const char* filename, int flags, ...) {
    mode_t mode = 0;
    va_list ap;
    va_start(ap, flags);
    mode = va_arg(ap, mode_t);
    va_end(ap);
    return sys_openat64(dirfd, filename, flags, mode);
}
