public static class Program
{
    public static void Main()
    {
        MeterConfig config = new MeterConfig();
        int total = Dll.Evaluate(config);
        Console.WriteLine(total);
    }
}
