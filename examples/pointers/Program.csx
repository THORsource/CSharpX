using System;

public static void Main()
{
    int value = 10;
    bool enabled = false;

    unsafe
    {
        int* number = &value;
        bool* flag = &enabled;
        Console.WriteLine(*number);
        *number = *number + 32;
        *flag = true;
    }

    Console.WriteLine(value);
    Console.WriteLine(enabled);
}
