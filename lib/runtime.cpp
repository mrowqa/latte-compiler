#include <cstdio>
#include <cstdlib>
#include <cstring>

extern "C" {

void printInt(int a) {
    printf("%d\n", a);
}

void printString(const char *a) {
    printf("%s\n", a);
}

void error() {
    printf("runtime error\n");
    exit(1);
}

int readInt() {
    int a;
    scanf("%d\n", &a);
    return a;
}

const char *readString() {
    char *line = 0;
    size_t len = 0;
    size_t read = getline(&line, &len, stdin);
    if (read <= 0) {
        return "";
    }

    if (line[read - 1] == '\n') {
        line[read - 1] = '\0';
    }
    return line;
}

const char *_bltn_string_concat(const char *a, const char *b) {
    size_t buf_size = strlen(a) + strlen(b) + 1;
    char *ptr = (char*) malloc(buf_size);
    strcpy(ptr, a);
    strcat(ptr, b);
    return ptr;
}

bool _bltn_string_eq(const char *a, const char *b) {
    return strcmp(a, b) == 0;
}

bool _bltn_string_ne(const char *a, const char *b) {
    return strcmp(a, b) != 0;
}

void *_bltn_malloc(int size) {
    if (size < 0) {
        error();
    }
    void *ptr = malloc(size);
    if (!ptr) {
        error();
    }
    memset(ptr, 0, size);
    return ptr;
}

}
