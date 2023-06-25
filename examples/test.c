#include "examples/header.h"

void f() {
}

int main() {
    f((struct A){ .a = 1, .b = 2 });
    return 0;
}

