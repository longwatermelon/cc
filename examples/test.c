#define bool int
#define true 1
#define false 0

int main()
{
    int x = 1;
    bool y = x == 1;

    if (y != true)
    {
        return 99;
    }
    return y;
}

