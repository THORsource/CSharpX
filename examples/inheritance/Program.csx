class Animal
{
    public virtual int Speak() { return 1; }
}
class Dog : Animal
{
    public override int Speak() { return 2; }
}
public static void Main()
{
    Animal a = new Dog();
    Animal b = a;
    Console.WriteLine(b.Speak());
    Dog exclusive = new Dog();
    Console.WriteLine(exclusive.Speak());
}
