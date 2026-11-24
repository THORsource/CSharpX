using Demo.Sensors;
using SensorReading = Demo.Sensors.Reading;
using Sim = Demo.Simulation;

namespace Demo.App;

public static class Program
{
    public static void Main()
    {
        Reading sensor = new SensorReading();
        Sim.Reading simulation = new Demo.Simulation.Reading();
        Console.WriteLine(sensor.Value());
        Console.WriteLine(simulation.Value());
    }
}
