using System.ComponentModel.Composition;
using Microsoft.VisualStudio.Utilities;

namespace CSharpX.VisualStudio.Editor
{
    internal static class ContentTypeDefinitions
    {
        [Export]
        [Name("CSharpX")]
        [BaseDefinition("code")]
        internal static ContentTypeDefinition CSharpXContentType = null;

        [Export]
        [FileExtension(".csx")]
        [ContentType("CSharpX")]
        internal static FileExtensionToContentTypeDefinition CSharpXFileExtension = null;
    }
}
