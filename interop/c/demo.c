#include "sensor.h"
#include <stdio.h>
int main(void) { sensor_init(1); printf("%.1f\n", sensor_read(1)); sensor_shutdown(1); return 0; }
