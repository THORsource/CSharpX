using System.Diagnostics;

public static void Main()
{
    int[] primes = new int[1000];
    int start = 0;
    int total = 0;
    int pass = 0;
    while (pass < 1020) {
        if (pass == 20) {
            total = 0;
            start = Stopwatch.GetTimestamp();
        }
        int count = 0;
        int candidate = 2;
        int sum = 0;
        while (count < 1000) {
            bool prime = true;
            int divisor = 2;
            while (divisor * divisor <= candidate) {
                if (candidate % divisor == 0) {
                    prime = false;
                    break;
                }
                divisor = divisor + 1;
            }
            if (prime) {
                primes[count] = candidate;
                count = count + 1;
                sum = sum + candidate;
            }
            candidate = candidate + 1;
        }
        total = total + sum;
        pass = pass + 1;
    }
    int elapsed = Stopwatch.GetTimestamp() - start;
    Console.WriteLine(elapsed);
    Console.WriteLine(Stopwatch.Frequency);
    Console.WriteLine(total);
    int count_out = 0;
    while (count_out < 1000) {
        Console.WriteLine(primes[count_out]);
        count_out = count_out + 1;
    }
}
