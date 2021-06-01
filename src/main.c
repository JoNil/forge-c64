#include <stdint.h>
#include <stdio.h>

extern uint16_t factorial(uint16_t n);

int main()
{
    uint16_t n = 6;
    uint16_t result = factorial(n);
    printf("hello from rust, %u! is %u", n, result);
    return 0;
}