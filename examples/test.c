struct Nested
{
    int a;
    char b;
};

struct A
{
    int a;
    struct Nested nested;
    int c;
};

int main()
{
    struct A a = (struct A){
        .a = 1,
        .nested = (struct Nested){
            .a = 90,
            .b = 'a'
        },
        .c = 2
    };

    return a.nested.a;
}

