using System;
using System.Collections.Generic;
using System.Text.RegularExpressions;

namespace CSharpX.VisualStudio.Editor
{
    internal enum SyntaxLanguage { CSharpX, CSharp, C, Rust }

    internal enum SyntaxKind { Keyword, Comment, String, Number, Type, Method, Operator, Punctuation }

    internal readonly struct SyntaxSpan
    {
        public SyntaxSpan(int start, int end, SyntaxKind kind)
        {
            Start = start;
            End = end;
            Kind = kind;
        }

        public int Start { get; }
        public int End { get; }
        public SyntaxKind Kind { get; }
    }

    internal sealed class SyntaxLexer
    {
        private static readonly HashSet<string> CSharpKeywords = new HashSet<string>(
            ("abstract as AS async await base bool break byte case catch char checked class const continue " +
             "decimal default delegate do double else enum event explicit extern false finally fixed float for " +
             "foreach goto if implicit in int interface internal is lock long namespace new null object operator out " +
             "override params private protected public readonly ref return sbyte sealed short sizeof stackalloc " +
             "static string struct switch this throw true try typeof uint ulong unchecked unsafe ushort using " +
             "var virtual void volatile while where select from let join orderby ascending descending group into").Split(' '));
        private static readonly HashSet<string> CKeywords = new HashSet<string>(
            ("auto break case char const continue default do double else enum extern float for goto if inline int " +
             "long register restrict return short signed sizeof static struct switch typedef union unsigned void " +
             "volatile while _Alignas _Alignof _Atomic _Bool _Complex _Generic _Noreturn _Static_assert _Thread_local " +
             "bool true false NULL nullptr").Split(' '));
        private static readonly HashSet<string> RustKeywords = new HashSet<string>(
            ("as async await break const continue crate dyn else enum extern false fn for if impl in let loop match " +
             "mod move mut pub ref return self Self static struct super trait true type union unsafe use where while " +
             "i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32 f64 bool char str").Split(' '));
        private static readonly HashSet<string> BuiltinTypes = new HashSet<string>(
            "Console String DateTime DateOnly TimeOnly TimeSpan DateTimeOffset Stopwatch Enumerable Exception".Split(' '));
        private static readonly Regex Number = new Regex(
            @"\G(?:0[xX][0-9a-fA-F_]+[uUlL]*|0[bB][01_]+[uUlL]*|\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d[\d_]*)?[fFdDmMuUlL]*)",
            RegexOptions.Compiled);

        private readonly string text;
        private readonly List<SyntaxSpan> spans = new List<SyntaxSpan>();
        private readonly SyntaxLanguage language;
        private readonly HashSet<string> keywords;
        private readonly HashSet<string> types;
        private int index;
        private bool nextType;

        private SyntaxLexer(string text, SyntaxLanguage language)
        {
            this.text = text;
            this.language = language;
            switch (language)
            {
                case SyntaxLanguage.C:
                    keywords = CKeywords;
                    types = new HashSet<string> { "FILE", "size_t", "ptrdiff_t", "int32_t", "uint32_t", "int64_t", "uint64_t" };
                    break;
                case SyntaxLanguage.Rust:
                    keywords = RustKeywords;
                    types = new HashSet<string> { "String", "Vec", "Box", "Option", "Result", "Some", "None", "Ok", "Err" };
                    break;
                default:
                    keywords = CSharpKeywords;
                    types = new HashSet<string>(BuiltinTypes);
                    break;
            }
        }

        public static SyntaxLanguage? LanguageFromExtension(string extension)
        {
            switch (extension?.ToLowerInvariant())
            {
                case ".csx": return SyntaxLanguage.CSharpX;
                case ".cs": return SyntaxLanguage.CSharp;
                case ".c":
                case ".h": return SyntaxLanguage.C;
                case ".rs": return SyntaxLanguage.Rust;
                default: return null;
            }
        }

        public static IReadOnlyList<SyntaxSpan> Scan(string text, SyntaxLanguage language = SyntaxLanguage.CSharpX)
        {
            var lexer = new SyntaxLexer(text, language);
            while (lexer.index < text.Length) lexer.ReadToken(0);
            return lexer.spans;
        }

        private char At(int lookahead = 0) =>
            index + lookahead < text.Length ? text[index + lookahead] : '\0';

        private void Add(int start, SyntaxKind kind)
        {
            if (index > start) spans.Add(new SyntaxSpan(start, index, kind));
        }

        private void ReadToken(int interpolationDepth)
        {
            int start = index;
            char ch = At(), next = At(1);
            if (char.IsWhiteSpace(ch))
            {
                index++;
                return;
            }
            if (ch == '/' && (next == '/' || next == '*'))
            {
                index += 2;
                if (next == '/')
                {
                    while (index < text.Length && At() != '\r' && At() != '\n') index++;
                }
                else
                {
                    ReadBlockComment();
                }
                Add(start, SyntaxKind.Comment);
                return;
            }
            if (language == SyntaxLanguage.C && ch == '#')
            {
                index++;
                while (At() == ' ' || At() == '\t') index++;
                while (IsIdentifierPart(At())) index++;
                string directive = text.Substring(start, index - start);
                Add(start, SyntaxKind.Keyword);
                if (directive.Substring(1).Trim() == "include")
                {
                    while (At() == ' ' || At() == '\t') index++;
                    if (At() == '<')
                    {
                        int include = index++;
                        while (index < text.Length && At() != '\n' && At() != '\r' && At() != '>') index++;
                        if (At() == '>') index++;
                        Add(include, SyntaxKind.String);
                    }
                }
                return;
            }
            if (language == SyntaxLanguage.Rust && TryReadRustRawString()) return;
            if (language == SyntaxLanguage.Rust && ch == 'b' && (next == '"' || next == '\''))
            {
                index++;
                ReadToken(interpolationDepth);
                return;
            }
            if (TryReadString(interpolationDepth)) return;
            if (ch == '\'' && language == SyntaxLanguage.Rust && IsIdentifierStart(next) && At(2) != '\'')
            {
                index += 2;
                while (IsIdentifierPart(At())) index++;
                Add(start, SyntaxKind.Type);
                return;
            }
            if (ch == '\'')
            {
                index++;
                while (index < text.Length && At() != '\r' && At() != '\n')
                {
                    char item = At();
                    index++;
                    if (item == '\'') break;
                    if (item == '\\' && index < text.Length && At() != '\r' && At() != '\n') index++;
                }
                Add(start, SyntaxKind.String);
                return;
            }
            if (char.IsDigit(ch))
            {
                var number = Number.Match(text, index);
                index += number.Success ? number.Length : 1;
                Add(start, SyntaxKind.Number);
                return;
            }
            if (ch == '@' && IsIdentifierStart(next) || IsIdentifierStart(ch))
            {
                index++;
                while (IsIdentifierPart(At())) index++;
                string word = text.Substring(start, index - start);
                if (keywords.Contains(word))
                {
                    nextType = word == "class" || word == "interface" || word == "struct" ||
                               word == "enum" || word == "union" || word == "trait" || word == "type";
                    Add(start, SyntaxKind.Keyword);
                }
                else if (nextType || types.Contains(word))
                {
                    if (nextType) types.Add(word);
                    nextType = false;
                    Add(start, SyntaxKind.Type);
                }
                else
                {
                    int after = index;
                    while (after < text.Length && char.IsWhiteSpace(text[after])) after++;
                    if (after < text.Length && (text[after] == '(' ||
                        language == SyntaxLanguage.Rust && text[after] == '!'))
                        Add(start, SyntaxKind.Method);
                }
                return;
            }

            index++;
            nextType = false;
            if ("+-*/%=!<>?&|^~".IndexOf(ch) >= 0) Add(start, SyntaxKind.Operator);
            else if ("(){}[]:;,.".IndexOf(ch) >= 0) Add(start, SyntaxKind.Punctuation);
        }

        private void ReadBlockComment()
        {
            int depth = 1;
            while (index < text.Length)
            {
                if (At() == '*' && At(1) == '/')
                {
                    index += 2;
                    if (--depth == 0) return;
                }
                else if (language == SyntaxLanguage.Rust && At() == '/' && At(1) == '*')
                {
                    index += 2;
                    depth++;
                }
                else
                {
                    index++;
                }
            }
        }

        private bool TryReadRustRawString()
        {
            int offset = At() == 'r' ? 1 : At() == 'b' && At(1) == 'r' ? 2 : 0;
            if (offset == 0) return false;
            int hashes = 0;
            while (At(offset + hashes) == '#') hashes++;
            if (At(offset + hashes) != '"') return false;
            int start = index;
            string terminator = "\"" + new string('#', hashes);
            int end = text.IndexOf(terminator, index + offset + hashes + 1, StringComparison.Ordinal);
            index = end < 0 ? text.Length : end + terminator.Length;
            Add(start, SyntaxKind.String);
            return true;
        }

        private bool TryReadString(int interpolationDepth)
        {
            int start = index;
            bool verbatim = At() == '@' || At(1) == '@';
            bool interpolated = At() == '$' || At(1) == '$';
            bool csharp = language == SyntaxLanguage.CSharpX || language == SyntaxLanguage.CSharp;
            int prefix = csharp && (At() == '@' || At() == '$') ? 1 : 0;
            if (prefix == 1 && (At() == '@' && At(1) == '$' || At() == '$' && At(1) == '@')) prefix++;
            if (At(prefix) != '"') return false;
            // A prefix determines the literal mode. Adjacent text is outside the string.
            verbatim = prefix > 0 && verbatim;
            interpolated = prefix > 0 && interpolated;
            index += prefix + 1;
            int segment = start;
            while (index < text.Length)
            {
                char ch = At(), next = At(1);
                if (!verbatim && language != SyntaxLanguage.Rust && (ch == '\r' || ch == '\n')) break;
                if (ch == '"' && (!verbatim || next != '"'))
                {
                    index++;
                    break;
                }
                if (verbatim && ch == '"' && next == '"' ||
                    !verbatim && ch == '\\' && next != '\0' && next != '\r' && next != '\n' ||
                    interpolated && (ch == '{' || ch == '}') && next == ch)
                {
                    index += 2;
                    continue;
                }
                if (interpolated && ch == '{' && interpolationDepth < 64)
                {
                    index++;
                    Add(segment, SyntaxKind.String);
                    ReadInterpolation(interpolationDepth + 1);
                    segment = index;
                    continue;
                }
                index++;
            }
            Add(segment, SyntaxKind.String);
            return true;
        }

        private void ReadInterpolation(int depth)
        {
            int braces = 0;
            while (index < text.Length)
            {
                char ch = At();
                if (ch == '}' && braces == 0)
                {
                    int start = index++;
                    Add(start, SyntaxKind.String);
                    return;
                }
                if (ch == '{') braces++;
                if (ch == '}') braces--;
                ReadToken(depth);
            }
        }

        private static bool IsIdentifierStart(char ch) => ch == '_' || char.IsLetter(ch);
        private static bool IsIdentifierPart(char ch) => ch == '_' || char.IsLetterOrDigit(ch);
    }
}
