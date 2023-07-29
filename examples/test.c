void f(int *x)
{
    *x = 5;
}

int main()
{
    int x = 1;
    f(&x);
    return x;
}

