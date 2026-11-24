using System;

public static class Program
{
    public static void Main()
    {
        Reading reading = new TemperatureReading();
        Console.WriteLine(reading.Value());
    }
}
