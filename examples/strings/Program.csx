using System;

public static void Main()
{
    String title = "  Hello CSharpX  ";
    System.String label = title.Trim();
    string emoji = "A🙂B";
    bool found = label.Contains("CSharpX");

    Console.WriteLine(label);
    Console.WriteLine(label.Replace("CSharpX", "World"));
    Console.WriteLine(label.Substring(0, 5).ToUpperInvariant());
    Console.WriteLine(label.ToLowerInvariant());
    Console.WriteLine(found);
    Console.WriteLine(label.StartsWith("Hello"));
    Console.WriteLine(label.EndsWith("CSharpX"));
    Console.WriteLine(label.Equals("Hello CSharpX"));
    Console.WriteLine(String.Concat(label, "!"));
    Console.WriteLine("tag: " + 10);
    Console.WriteLine(emoji.Length);
    Console.WriteLine(emoji.IndexOf("B"));
    Console.WriteLine(emoji.Substring(1, 2));
    Console.WriteLine(string.IsNullOrEmpty(String.Empty));
    Console.WriteLine(String.IsNullOrWhiteSpace(" "));
    Console.WriteLine(String.IsNullOrEmpty(null));

    string folder = @"C:\Temp\Reports";
    string quoted = @"He said ""Hello"".";
    Console.WriteLine(folder);
    Console.WriteLine(quoted);
    Console.WriteLine($"Label: {label}, length: {label.Length}");
    Console.WriteLine($"{{{label.Substring(0, 5)}}}");
    Console.WriteLine($@"{folder}\{label}.txt");
    Console.WriteLine(@$"Name: ""{label}""");
}
