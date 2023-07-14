struct Nested
{
    char a;
    char b;
    int c;
};

struct A
{
    int a;
    struct Nested nested;
    int c;
};

int f(struct A value)
{
    return value.nested.c;
}

int main()
{
    struct A a = (struct A){
        .a = 1,
        .nested = (struct Nested){
            .a = 'a',
            .b = 'b',
            .c = 5
        },
        .c = 2
    };

    return f(a) + 1 + 2;
}

