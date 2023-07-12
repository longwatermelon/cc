struct A
{
    int a;
    char b;
    int c;
};

void f(struct A tmp)
{
}

int main()
{
    f((struct A){ .a = 1, .b = 'b', .c = 2 });
}

