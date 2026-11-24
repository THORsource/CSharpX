using "Greeting.rs";
using "TextTools.rs" as Text;

public static void Main()
{
    string result = "";
    unsafe
    {
        result = Text.upper(greet("CSharpX"));
    }
    Console.WriteLine(result);
}
