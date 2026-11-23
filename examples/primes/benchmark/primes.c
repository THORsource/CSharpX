#include <windows.h>
#include <stdbool.h>
#include <stdint.h>
#include <inttypes.h>
#include <stdio.h>

static int64_t timestamp(void)
{
    LARGE_INTEGER value;
    QueryPerformanceCounter(&value);
    return value.QuadPart;
}

static void print_value(int64_t value)
{
    printf("%" PRId64 "\n", value);
}

int main(void)
{
    LARGE_INTEGER frequency;
    QueryPerformanceFrequency(&frequency);
    int64_t primes[1000] = {0};
    int64_t start = 0;
    int64_t total = 0;
    int64_t pass = 0;
    while (pass < 1020) {
        if (pass == 20) {
            total = 0;
            start = timestamp();
        }
        int64_t count = 0;
        int64_t candidate = 2;
        int64_t sum = 0;
        while (count < 1000) {
            bool prime = true;
            int64_t divisor = 2;
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
    int64_t elapsed = timestamp() - start;
    print_value(elapsed);
    print_value(frequency.QuadPart);
    print_value(total);
    int64_t count_out = 0;
    while (count_out < 1000) {
        print_value(primes[count_out]);
        count_out = count_out + 1;
    }
    return 0;
}
