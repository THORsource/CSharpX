using "c_file_access.h" as Native;

static string Decorate(string text)
{
    string output = "";
    unsafe
    {
        output = Native.csx_c_file_round_trip("native-report.txt");
    }
    return $"[{text}]";
}
