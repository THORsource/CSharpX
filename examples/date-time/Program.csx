using System;

public static void Main()
{
    DateOnly date = new DateOnly(2028, 2, 29);
    TimeOnly time = new TimeOnly(14, 5, 9);
    DateTime timestamp = new DateTime(2028, 2, 29, 14, 5, 9);
    TimeSpan duration = new TimeSpan(1, 2, 3, 4);

    Console.WriteLine(date.ToString("yyyy-MM-dd"));
    Console.WriteLine(time.ToString("HH:mm:ss"));
    Console.WriteLine(timestamp.ToString("yyyy-MM-dd HH:mm:ss"));
    Console.WriteLine(duration.ToString("c"));
    Console.WriteLine(timestamp.Year);
    Console.WriteLine(duration.Hours);
}
