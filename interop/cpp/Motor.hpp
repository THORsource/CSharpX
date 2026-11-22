#pragma once
class Motor {
public:
    explicit Motor(int pin) : pin_(pin), speed_(0) {}
    void Start() { speed_ = 100; }
    void Stop() { speed_ = 0; }
    int GetSpeed() const { return speed_; }
private:
    int pin_;
    int speed_;
};
