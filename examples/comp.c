int main(int argc, char **argv)
{
    printf("%d\n", 11332);
    int x;
    int *x;
    int x = 1;
    int *x = 1;
    x = 1;
    *x = 1;

    if (x)
    {
        x = 2;
    }

    x = x + 1 + 2 + *x + x + *x;
    f(x);

    return 0;
}
