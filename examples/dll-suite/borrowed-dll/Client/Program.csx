public static class Program
{
    public static void Main()
    {
        SampleConfig config = new SampleConfig();
        SampleConfig alias = config;
        Console.WriteLine(config.Count());
        Console.WriteLine(Dll.Read(config));
        Console.WriteLine(Dll.Read(alias));
    }
}
