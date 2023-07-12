struct A
{
    int a;
    char b;
    int c;
};

int main()
{
    int x = 1;
    struct A a = (struct A){ .a = x, .b = 'b', .c = 2 };
}

