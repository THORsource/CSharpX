using System;
using System.Collections.Generic;
using System.ComponentModel.Composition;
using System.IO;
using System.Threading.Tasks;
using Microsoft.VisualStudio.ProjectSystem;
using Microsoft.VisualStudio.ProjectSystem.Debug;
using Microsoft.VisualStudio.ProjectSystem.VS.Debug;

namespace CSharpX.VisualStudio
{
    [ExportDebugger("CSharpXDebugger")]
    [AppliesTo("CSharpX")]
    internal sealed class CSharpXLaunchProvider : DebugLaunchProviderBase
    {
        private readonly ConfiguredProject project;

        [ImportingConstructor]
        public CSharpXLaunchProvider(ConfiguredProject project) : base(project)
        {
            this.project = project;
        }

        private Task<string> GetTargetPathAsync() =>
            project.Services.ProjectPropertiesProvider.GetCommonProperties()
                .GetEvaluatedPropertyValueAsync("TargetPath");

        public override async Task<bool> CanLaunchAsync(DebugLaunchOptions options)
        {
            var path = await GetTargetPathAsync();
            return Path.IsPathRooted(path) &&
                string.Equals(Path.GetExtension(path), ".exe", StringComparison.OrdinalIgnoreCase);
        }

        public override async Task<IReadOnlyList<IDebugLaunchSettings>> QueryDebugTargetsAsync(
            DebugLaunchOptions options)
        {
            var executable = await GetTargetPathAsync();
            return new IDebugLaunchSettings[]
            {
                new DebugLaunchSettings(options)
                {
                    Executable = executable,
                    CurrentDirectory = Path.GetDirectoryName(project.UnconfiguredProject.FullPath),
                    LaunchOperation = DebugLaunchOperation.CreateProcess,
                    LaunchDebugEngineGuid = DebuggerEngines.NativeOnlyEngine,
                }
            };
        }
    }
}
