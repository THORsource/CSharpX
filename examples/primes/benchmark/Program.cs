using System;
using System.Diagnostics;

long[] primes = new long[1000];
long start = 0;
long total = 0;
long pass = 0;
while (pass < 1020) {
    if (pass == 20) {
        total = 0;
        start = Stopwatch.GetTimestamp();
    }
    long count = 0;
    long candidate = 2;
    long sum = 0;
    while (count < 1000) {
        bool prime = true;
        long divisor = 2;
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
long elapsed = Stopwatch.GetTimestamp() - start;
Console.WriteLine(elapsed);
Console.WriteLine(Stopwatch.Frequency);
Console.WriteLine(total);
long count_out = 0;
while (count_out < 1000) {
    Console.WriteLine(primes[count_out]);
    count_out = count_out + 1;
}
