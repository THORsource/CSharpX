public static class Program
{
    public static void Main()
    {
        TextConfig config = new TextConfig();
        string response = Dll.Describe(config, "Grüße • 日本語");
        Console.WriteLine(response);
    }
}
