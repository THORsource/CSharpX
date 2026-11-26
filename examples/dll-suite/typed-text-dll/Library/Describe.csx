public static string Describe(TextConfig config, string text)
{
    if (text.Length == 0)
    {
        throw "Text is required.";
    }

    string[] parts = new string[] { "", "" };
    try
    {
        if (text.Length == 1)
        {
            throw "short input";
        }
        parts[0] = text;
    }
    catch (Exception message)
    {
        parts[0] = $"recovered: {message}";
    }
    finally
    {
        parts[1] = "|";
    }

    string result = "";
    int index = 0;
    while (index < config.Copies())
    {
        result = result + parts[0] + parts[1];
        index = index + 1;
    }

    if (config.Decorate())
    {
        return Decorate(result);
    }
    return result;
}
