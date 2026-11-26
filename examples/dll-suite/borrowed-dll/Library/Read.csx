public static int Read(SampleConfig config)
{
    if (config.Active())
    {
        return config.Count() + 1;
    }
    return 0;
}
