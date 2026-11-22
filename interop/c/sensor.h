#ifndef CSX_SENSOR_H
#define CSX_SENSOR_H
int sensor_init(int device_id);
float sensor_read(int device_id);
void sensor_shutdown(int device_id);
#endif
