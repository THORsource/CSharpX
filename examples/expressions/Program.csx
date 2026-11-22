public static void Main()
{
    int a = 12;
    int b = 3;
    Console.WriteLine(a / b + 2);
    try
    {
        throw "sensor failure";
    }
    catch (Exception ex)
    {
        Console.WriteLine(ex);
    }
    finally
    {
        Console.WriteLine("cleanup");
    }
}
