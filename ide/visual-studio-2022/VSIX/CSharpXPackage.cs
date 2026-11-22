using System;
using System.Runtime.InteropServices;
using Microsoft.VisualStudio.Shell;

namespace CSharpX.VisualStudio
{
    [PackageRegistration(UseManagedResourcesOnly = true, AllowsBackgroundLoading = true)]
    [InstalledProductRegistration("CSharpX", "CSharpX language integration scaffold", "0.1")]
    [Guid("B9D3A6A0-4B1A-4C4E-8F89-CA7FAD2F89C3")]
    public sealed class CSharpXPackage : AsyncPackage { }
}
