struct A
{
    int a;
    char b;
    int c;
};

int main()
{
    struct A a = (struct A){ .a = 1, .b = 'b', .c = 2 };
    return a.c;
}

