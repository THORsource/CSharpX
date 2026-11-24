public static class AlternateProgram
{
    public static void Main()
    {
        TemperatureReading reading = new TemperatureReading();
        Console.WriteLine("Alternative");
        Console.WriteLine(reading.Value());
    }
}
