#include <stdio.h>
#include <readline/readline.h>
#include <stdlib.h>

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
    scanf("%d", &a);
    return a;
}

const char* readString() {
    return readline(0);
}

