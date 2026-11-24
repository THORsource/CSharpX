using System;

using "Greeting.cs";
using "TextTools.cs" AS Text;

public static void Main()
{
    string greeting = "";
    unsafe
    {
        greeting = Greet("CSharpX");
    }
    Console.WriteLine(greeting);

    string uppercase = "";
    unsafe
    {
        uppercase = Text.Upper("Grüße aus C#");
    }
    Console.WriteLine(uppercase);
    Console.ReadLine();
}
