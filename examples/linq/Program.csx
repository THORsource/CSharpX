using System.Linq;

public static void Main()
{
    int minimum = 3;
    int total = Enumerable.Range(1, 6)
        .Where(x => x > minimum)
        .Select(x => x * 2)
        .Sum();

    int count = Enumerable.Range(1, 6)
        .Where(x => x <= minimum)
        .Count();

    Console.WriteLine(total);
    Console.WriteLine(count);
}
