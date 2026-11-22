#include "sensor.h"
static int active_id = -1;
int sensor_init(int device_id) { active_id = device_id; return 0; }
float sensor_read(int device_id) { return device_id == active_id ? 23.5f : -1.0f; }
void sensor_shutdown(int device_id) { if (device_id == active_id) active_id = -1; }
