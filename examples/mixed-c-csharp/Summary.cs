public static class Summary
{
    public static string Describe(string text)
    {
        int lineCount = text.TrimEnd('\r', '\n').Split('\n').Length;
        return $"C# hat {lineCount} Zeilen aus C erhalten.";
    }
}
