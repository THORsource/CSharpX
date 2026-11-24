using System;
using "c_file_access.h" as Native;
using "Summary.cs" as Managed;

public static void Main()
{
    string text = "";
    unsafe
    {
        text = Native.csx_c_file_round_trip("mixed-interop-demo.txt");
    }
    Console.WriteLine(text);

    string summary = "";
    unsafe
    {
        summary = Managed.Describe(text);
    }
    Console.WriteLine(summary);
}
