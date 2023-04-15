#include<stdio.h>

int conv(char c) {
    if (c >> 3) {
        c |= 0xf0;
    }

    switch(c){
        case -7: return -255;
        case -6: return -219;
        case -5: return -182;
        case -4: return -146;
        case -3: return -109;
        case -2: return -73;
        case -1: return -36;
        case 0: default: return 0;
        case 1: return 36;
        case 2: return 73;
        case 3: return 109;
        case 4: return 146;
        case 5: return 182;
        case 6: return 219;
        case 7: return 255;
    }
}

int main() {
    signed char c = 0b00000000;
    signed char l = c >> 4;
    signed char r = c & 0x0f;

    printf("%d %d\n", (int)l, (int)r);
    int l1 = conv(l);
    int r1 = conv(r);

    printf("%d %d\n", (int)l1, (int)r1);
}