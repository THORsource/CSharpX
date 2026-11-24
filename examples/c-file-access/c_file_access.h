#ifndef CSX_C_FILE_ACCESS_H
#define CSX_C_FILE_ACCESS_H

#include <stddef.h>

/* Borrows path and output for the call. Returns bytes read, or -1 on failure. */
int csx_c_file_round_trip(const char *path, unsigned char *output, size_t capacity);

#endif
