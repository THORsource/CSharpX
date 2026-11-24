#include "extra_file.h"
#include "c_file_access.h"

int csx_extra_file(const char *path, unsigned char *output, size_t capacity)
{
    return csx_c_file_round_trip(path, output, capacity);
}
