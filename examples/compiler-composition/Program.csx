using "c_file_access.h" as Native;

public static class Program
{
    public static void Main()
    {
        SampleReading reading = new SampleReading();
        int limit = reading.Limit();
        string label = reading.Label();
        int index = 0;
        if (reading.Enabled())
        {
            while (index < limit)
            {
                Console.WriteLine($"{reading.Label()} {index}");
                index = index + 1;
            }
        }

        string report = "";
        unsafe
        {
            report = Native.csx_c_file_round_trip("compiler-demo.txt");
        }
        Console.WriteLine(label);
        Console.WriteLine(report);
    }
}
