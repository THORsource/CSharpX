using "c_file_access.h";
using "extra_file.h" AS Extra;

public static void Main()
{
    string text = "";
    unsafe
    {
        text = csx_c_file_round_trip("c-file-access-demo.txt");
    }
    Console.WriteLine(text);

    string extra = "";
    unsafe
    {
        extra = Extra.csx_extra_file("c-file-access-extra.txt");
    }
    Console.WriteLine(extra);
}
