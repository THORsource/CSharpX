using Microsoft.VisualStudio.Language.StandardClassification;
using System;
using System.Collections.Generic;
using System.ComponentModel.Composition;
using Microsoft.VisualStudio.Text;
using Microsoft.VisualStudio.Text.Classification;
using Microsoft.VisualStudio.Utilities;

namespace CSharpX.VisualStudio.Editor
{
    [Export(typeof(IClassifierProvider))]
    [ContentType("text")]
    internal sealed class CSharpXClassifierProvider : IClassifierProvider
    {
        [Import] internal IClassificationTypeRegistryService Registry = null;
        [Import] internal ITextDocumentFactoryService Documents = null;

        public IClassifier GetClassifier(ITextBuffer buffer)
        {
            bool isCSharpX = buffer.ContentType.IsOfType("CSharpX");
            var extension = Documents.TryGetTextDocument(buffer, out var document)
                ? System.IO.Path.GetExtension(document.FilePath)
                : null;
            var language = SyntaxLexer.LanguageFromExtension(extension);
            if (language == null && !isCSharpX) return null;
            // Keep the installed language service in charge on native editor buffers.
            if (language == SyntaxLanguage.CSharp && buffer.ContentType.IsOfType("CSharp") ||
                language == SyntaxLanguage.C && (buffer.ContentType.IsOfType("C/C++") || buffer.ContentType.IsOfType("C++")) ||
                language == SyntaxLanguage.Rust && buffer.ContentType.IsOfType("Rust"))
                return null;

            return buffer.Properties.GetOrCreateSingletonProperty(
                () => new CSharpXClassifier(buffer, Registry, language ?? SyntaxLanguage.CSharpX));
        }
    }

    internal sealed class CSharpXClassifier : IClassifier
    {
        private readonly IClassificationType[] classifications;
        private readonly SyntaxLanguage language;
        private readonly object cacheLock = new object();
        private ITextSnapshot cachedSnapshot;
        private IReadOnlyList<SyntaxSpan> cachedTokens;

        public CSharpXClassifier(ITextBuffer buffer, IClassificationTypeRegistryService registry, SyntaxLanguage language)
        {
            this.language = language;
            var identifier = registry.GetClassificationType(PredefinedClassificationTypeNames.Identifier);
            classifications = new[]
            {
                registry.GetClassificationType(PredefinedClassificationTypeNames.Keyword),
                registry.GetClassificationType(PredefinedClassificationTypeNames.Comment),
                registry.GetClassificationType(PredefinedClassificationTypeNames.String),
                registry.GetClassificationType(PredefinedClassificationTypeNames.Number),
                registry.GetClassificationType("class name") ?? identifier,
                registry.GetClassificationType("method name") ?? identifier,
                registry.GetClassificationType(PredefinedClassificationTypeNames.Operator),
                registry.GetClassificationType("punctuation") ?? identifier,
            };
            buffer.ChangedLowPriority += OnBufferChanged;
        }

        public event EventHandler<ClassificationChangedEventArgs> ClassificationChanged;

        private void OnBufferChanged(object sender, TextContentChangedEventArgs args)
        {
            // An edit can open a block comment or verbatim string across later lines.
            ClassificationChanged?.Invoke(this,
                new ClassificationChangedEventArgs(new SnapshotSpan(args.After, 0, args.After.Length)));
        }

        public IList<ClassificationSpan> GetClassificationSpans(SnapshotSpan span)
        {
            var results = new List<ClassificationSpan>();
            if (span.IsEmpty) return results;

            IReadOnlyList<SyntaxSpan> tokens;
            lock (cacheLock)
            {
                if (cachedSnapshot != span.Snapshot)
                {
                    cachedTokens = SyntaxLexer.Scan(span.Snapshot.GetText(), language);
                    cachedSnapshot = span.Snapshot;
                }
                tokens = cachedTokens;
            }

            int lo = 0, hi = tokens.Count;
            while (lo < hi)
            {
                int mid = lo + (hi - lo) / 2;
                if (tokens[mid].End <= span.Start.Position) lo = mid + 1;
                else hi = mid;
            }
            for (int i = lo; i < tokens.Count && tokens[i].Start < span.End.Position; i++)
            {
                var token = tokens[i];
                var classification = classifications[(int)token.Kind];
                if (classification == null) continue;
                int start = Math.Max(token.Start, span.Start.Position);
                int end = Math.Min(token.End, span.End.Position);
                results.Add(new ClassificationSpan(new SnapshotSpan(span.Snapshot, start, end - start), classification));
            }
            return results;
        }
    }
}
