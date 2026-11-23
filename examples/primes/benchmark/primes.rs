use std::time::Instant;

fn main() {
    let origin = Instant::now();
    let timestamp = || origin.elapsed().as_nanos() as i64;
    let mut primes = [0i64; 1000];
    let mut start: i64 = 0;
    let mut total: i64 = 0;
    let mut pass: i64 = 0;
    while pass < 1020 {
        if pass == 20 {
            total = 0;
            start = timestamp();
        }
        let mut count: i64 = 0;
        let mut candidate: i64 = 2;
        let mut sum: i64 = 0;
        while count < 1000 {
            let mut prime = true;
            let mut divisor: i64 = 2;
            while divisor * divisor <= candidate {
                if candidate % divisor == 0 {
                    prime = false;
                    break;
                }
                divisor = divisor + 1;
            }
            if prime {
                primes[count as usize] = candidate;
                count = count + 1;
                sum = sum + candidate;
            }
            candidate = candidate + 1;
        }
        total = total + sum;
        pass = pass + 1;
    }
    let elapsed: i64 = timestamp() - start;
    println!("{}", elapsed);
    println!("{}", 1_000_000_000);
    println!("{}", total);
    let mut count_out: i64 = 0;
    while count_out < 1000 {
        println!("{}", primes[count_out as usize]);
        count_out = count_out + 1;
    }
}
