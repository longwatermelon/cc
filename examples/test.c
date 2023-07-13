struct A
{
    int a;
    char b;
    int c;
};

int f(int num, struct A tmp, char c)
{
    return num;
}

int main()
{
    return f(1, (struct A){ .a = 10, .b = 'a', .c = 20 }, 'a');
}

