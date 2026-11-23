public static void Main()
{
    int[] values = { 10, 20, 30 };
    int[] alias = values;
    alias[1] = alias[0] + 32;

    var names = new[] { "dashboard", "logger" };
    bool[] ready = new bool[2];
    ready[0] = true;

    int[] zeros = new int[2];
    var dates = new DateOnly[] { new DateOnly(2028, 2, 29) };

    Console.WriteLine($"{names[0]}: {values[1]}, length {alias.Length}");
    Console.WriteLine($"{names[1]}: {ready[0]}, default {ready[1]}");
    Console.WriteLine($"{zeros[0]} / {dates[0].ToString("yyyy-MM-dd")}");
}
