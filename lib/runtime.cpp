#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cctype>

extern "C" {

void printInt(int a) {
    printf("%d\n", a);
}

void printString(const char *a) {
    printf("%s\n", a ? a : "");
}

void error() {
    printf("runtime error\n");
    exit(1);
}

int readInt() {
    char *line = 0;
    size_t len = 0;
    size_t read = getline(&line, &len, stdin);
    if (read <= 0) {
        error();
    }

    char *ptr = line;
    while (ptr < line+read && isspace(*ptr)) ptr++;
    if (ptr < line+read && *ptr == '-') ptr++;
    else if (ptr < line+read && *ptr == '+') ptr++;
    while (ptr < line+read && isspace(*ptr)) ptr++;
    if (!(ptr < line+read && isdigit(*ptr))) {
        error();
    }
    while (ptr < line+read && isdigit(*ptr)) ptr++;
    while (ptr < line+read && isspace(*ptr)) ptr++;
    if (ptr != line + read) {
        error();
    }

    int num = atoi(line);
    free(line);
    return num;
}

const char *readString() {
    char *line = 0;
    size_t len = 0;
    size_t read = getline(&line, &len, stdin);
    if (read <= 0) {
        return nullptr;
    }

    if (line[read - 1] == '\n') {
        line[read - 1] = '\0';
    }
    return line;
}

const char *_bltn_string_concat(const char *a, const char *b) {
    if (!a) {
        return b;
    }
    if (!b) {
        return a;
    }

    size_t buf_size = strlen(a) + strlen(b) + 1;
    char *ptr = (char*) malloc(buf_size);
    strcpy(ptr, a);
    strcat(ptr, b);
    return ptr;
}

bool _bltn_string_eq(const char *a, const char *b) {
    if (!a && !b) {
        return true;
    }
    if (!a || !b) {
        return false;
    }

    return strcmp(a, b) == 0;
}

bool _bltn_string_ne(const char *a, const char *b) {
    return !_bltn_string_eq(a, b);
}

void *_bltn_malloc(int size) {
    if (size <= 0) {
        error();
    }
    void *ptr = malloc(size);
    if (!ptr) {
        error();
    }
    memset(ptr, 0, size);
    return ptr;
}

void *_bltn_alloc_array(int elem_cnt, int elem_size) {
    static_assert(sizeof(int) == 4, "sizeof(int) == 4");
    if (elem_cnt <= 0 || elem_size <= 0) { // todo readme <-- alokacja co najmniej 1 bajtu
        error();
    }

    int header_size = sizeof(int);
    int size = elem_cnt * elem_size + header_size;
    int *header_ptr = static_cast<int*>(_bltn_malloc(size));
    *header_ptr = elem_cnt;
    return header_ptr + 1;
}

}
