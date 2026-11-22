using System;
using System.Collections.Generic;
using System.ComponentModel.Composition;
using System.Text.RegularExpressions;
using Microsoft.VisualStudio.Text;
using Microsoft.VisualStudio.Text.Classification;
using Microsoft.VisualStudio.Utilities;

namespace CSharpX.VisualStudio.Editor
{
    [Export(typeof(IClassifierProvider))]
    [ContentType("CSharpX")]
    internal sealed class CSharpXClassifierProvider : IClassifierProvider
    {
        [Import] internal IClassificationTypeRegistryService Registry = null;
        public IClassifier GetClassifier(ITextBuffer buffer)
        {
            return buffer.Properties.GetOrCreateSingletonProperty(() => new CSharpXClassifier(Registry));
        }
    }

    internal sealed class CSharpXClassifier : IClassifier
    {
        private static readonly Regex Tokens = new Regex(
            @"//[^\r\n]*|/\*[\s\S]*?\*/|""(?:\\.|[^""\\])*""|\b(?:class|interface|struct|enum|public|private|protected|internal|static|virtual|override|abstract|sealed|new|return|void|int|long|float|double|bool|string|true|false|null|if|else|while|for|foreach|using|namespace|try|catch|finally|throw|unsafe|extern|base|this|var|async|await)\b|\b\d+(?:\.\d+)?\b",
            RegexOptions.Compiled);
        private readonly IClassificationType keyword, comment, str, number;
        public CSharpXClassifier(IClassificationTypeRegistryService registry)
        {
            keyword = registry.GetClassificationType(PredefinedClassificationTypeNames.Keyword);
            comment = registry.GetClassificationType(PredefinedClassificationTypeNames.Comment);
            str = registry.GetClassificationType(PredefinedClassificationTypeNames.String);
            number = registry.GetClassificationType(PredefinedClassificationTypeNames.Number);
        }
        public event EventHandler<ClassificationChangedEventArgs> ClassificationChanged { add {} remove {} }
        public IList<ClassificationSpan> GetClassificationSpans(SnapshotSpan span)
        {
            var results = new List<ClassificationSpan>();
            string text = span.Snapshot.GetText();
            foreach (Match match in Tokens.Matches(text))
            {
                if (match.Index + match.Length <= span.Start.Position || match.Index >= span.End.Position) continue;
                var kind = match.Value.StartsWith("//") || match.Value.StartsWith("/*") ? comment :
                           match.Value.StartsWith("\"") ? str : char.IsDigit(match.Value[0]) ? number : keyword;
                int start = Math.Max(match.Index, span.Start.Position);
                int end = Math.Min(match.Index + match.Length, span.End.Position);
                results.Add(new ClassificationSpan(new SnapshotSpan(span.Snapshot, start, end - start), kind));
            }
            return results;
        }
    }
}
