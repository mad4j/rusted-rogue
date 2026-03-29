#include <stdio.h>
#include <stdlib.h>

/* Legacy RNG functions from random.c */
void srrandom(int x);
long rrandom(void);
int get_rand(int x, int y);
int rand_percent(int percentage);
int coin_toss(void);

int main(int argc, char **argv) {
    int seed = 12345;
    int i;

    if (argc > 1) {
        seed = atoi(argv[1]);
    }

    srrandom(seed);

    printf("seed=%d\n", seed);
    printf("rrandom:");
    for (i = 0; i < 10; i++) {
        printf(" %ld", rrandom());
    }
    printf("\n");

    srrandom(seed);
    printf("get_rand_1_100:");
    for (i = 0; i < 10; i++) {
        printf(" %d", get_rand(1, 100));
    }
    printf("\n");

    srrandom(seed);
    printf("coin_toss:");
    for (i = 0; i < 10; i++) {
        printf(" %d", coin_toss());
    }
    printf("\n");

    srrandom(seed);
    printf("rand_percent_50:");
    for (i = 0; i < 10; i++) {
        printf(" %d", rand_percent(50));
    }
    printf("\n");

    return 0;
}
