public abstract class Reading<T, TLabel = string>
{
    public abstract T Value();
    public abstract TLabel Label();
}

public class NumericReading : Reading<int>
{
    public override int Value() { return 100; }
    public override string Label() { return "calibration"; }
}

public class TextReading : Reading<string, string>
{
    public override string Value() { return "42 kPa"; }
    public override string Label() { return "live"; }
}

public class FlaggedReading : Reading<int, bool>
{
    public override int Value() { return 42; }
    public override bool Label() { return true; }
}

public abstract class Snapshot<TDate, TTime, TStamp, TDuration = TimeSpan>
{
    public abstract TDate Date();
    public abstract TTime Time();
    public abstract TStamp Timestamp();
    public abstract TDuration Duration();
}

public class RecordedSnapshot : Snapshot<DateOnly, TimeOnly, DateTime>
{
    public override DateOnly Date() { return new DateOnly(2028, 2, 29); }
    public override TimeOnly Time() { return new TimeOnly(14, 5, 9); }
    public override DateTime Timestamp() { return new DateTime(2028, 2, 29, 14, 5, 9); }
    public override TimeSpan Duration() { return new TimeSpan(1, 2, 3, 4); }
}

public static void Main()
{
    Reading<int> calibration = new NumericReading();
    Console.WriteLine("Reading<int, string> - Box; default TLabel");
    Console.WriteLine(calibration.Value());
    Console.WriteLine(calibration.Label());

    Reading<string> acquisition = new TextReading();
    Reading<string, string> dashboard = acquisition;
    Reading<string> logger = dashboard;
    Console.WriteLine("Reading<string, string> - Rc; default and explicit TLabel");
    Console.WriteLine(acquisition.Value());
    Console.WriteLine(dashboard.Value());
    Console.WriteLine(logger.Value());
    Console.WriteLine(logger.Label());

    Reading<int, bool> flagged = new FlaggedReading();
    Console.WriteLine("Reading<int, bool> - Box; explicit override of TLabel");
    Console.WriteLine(flagged.Value());
    Console.WriteLine(flagged.Label());

    Snapshot<DateOnly, TimeOnly, DateTime> snapshot = new RecordedSnapshot();
    Console.WriteLine("Snapshot<DateOnly, TimeOnly, DateTime, TimeSpan> - Box; default TDuration");
    Console.WriteLine(snapshot.Date());
    Console.WriteLine(snapshot.Time());
    Console.WriteLine(snapshot.Timestamp());
    Console.WriteLine(snapshot.Duration());
}
