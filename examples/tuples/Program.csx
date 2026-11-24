using System;

public static void Main()
{
    var reading = (Value: 41, Unit: "kPa");
    reading.Value = reading.Item1 + 1;
    Console.WriteLine(reading);

    (int Value, string Unit) copy = (reading.Item1, reading.Item2);
    var (value, unit) = copy;
    Console.WriteLine($"{value} {unit}");

    var nested = (Reading: copy, Valid: true);
    Console.WriteLine(nested.Reading.Unit);
    Console.WriteLine(nested.Valid);
}
