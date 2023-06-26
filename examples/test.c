int func(int a)
{
    return a;
}

int func2(int a)
{
    return 49;
}

int main()
{
    int x = func2(func(1));
    return x;
}

