public static int Evaluate(MeterConfig config)
{
    if (config.Enabled())
    {
        return config.Units() * 3;
    }
    return 0;
}
