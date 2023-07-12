struct A
{
    int a;
    char b;
    int c;
};

int f(struct A tmp)
{
    return tmp.a;
}

int main()
{
    return f((struct A){ .a = 1, .b = 'b', .c = 2 });
}

