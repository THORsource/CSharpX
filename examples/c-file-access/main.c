#define _CRT_SECURE_NO_WARNINGS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "c_file_access.h"

static int write_text(const char *path, const char *text)
{
    FILE *file = fopen(path, "wbx");
    if (file == NULL) {
        perror(path);
        return 0;
    }

    const size_t length = strlen(text);
    const size_t written = fwrite(text, 1, length, file);
    const int closed = fclose(file);
    if (written != length || closed == EOF) {
        fprintf(stderr, "Could not write %s completely.\n", path);
        return 0;
    }
    return 1;
}

static int read_text(const char *path, char *buffer, size_t capacity, size_t *bytes_read)
{
    FILE *file = fopen(path, "rb");
    if (file == NULL) {
        perror(path);
        return 0;
    }

    const size_t length = fread(buffer, 1, capacity, file);
    const int failed = ferror(file) || length == capacity;
    const int closed = fclose(file);
    if (failed || closed == EOF) {
        fprintf(stderr, "Could not read %s completely.\n", path);
        return 0;
    }

    *bytes_read = length;
    return 1;
}

int csx_c_file_round_trip(const char *path, unsigned char *output, size_t capacity)
{
    const char text[] = "Hallo aus C!\nDatei geschrieben und wieder gelesen.\n";
    const size_t length = sizeof text - 1;
    size_t bytes_read;
    if (capacity < sizeof text || !write_text(path, text) ||
        !read_text(path, (char *)output, capacity, &bytes_read)) {
        return -1;
    }
    if (bytes_read != length || memcmp(text, output, length) != 0) {
        fprintf(stderr, "The file content differs from the original.\n");
        return -1;
    }
    return (int)length;
}

#ifndef CSX_C_FILE_LIBRARY
int main(int argc, char *argv[])
{
    if (argc != 2) {
        fprintf(stderr, "Usage: c-file-access <new-file>\n");
        return EXIT_FAILURE;
    }

    unsigned char buffer[256];
    const int length = csx_c_file_round_trip(argv[1], buffer, sizeof buffer);
    if (length < 0) {
        return EXIT_FAILURE;
    }
    return fwrite(buffer, 1, (size_t)length, stdout) == (size_t)length
        ? EXIT_SUCCESS : EXIT_FAILURE;
}
#endif
