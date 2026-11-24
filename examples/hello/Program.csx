public static void Main()
{
    int i = 0;
    Mensch f = new Frau();
    i = 10;
    Console.ReadLine();
    Console.WriteLine(f.GetName());
    Console.WriteLine(i);
    Console.ReadLine();
}

public abstract class Mensch
{
    public abstract string GetName();
}

public class Mann : Mensch
{
    public override string GetName()
    {
        return "Max";
    }
}

public class Frau : Mensch
{
    public override string GetName()
    {
        return "Marie";
    }
}
