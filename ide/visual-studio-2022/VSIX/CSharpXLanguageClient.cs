using System;
using System.ComponentModel.Composition;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.VisualStudio.LanguageServer.Client;
using Microsoft.VisualStudio.Utilities;

namespace CSharpX.VisualStudio
{
    [Export(typeof(ILanguageClient))]
    [ContentType("CSharpX")]
    internal sealed class CSharpXLanguageClient : ILanguageClient
    {
        public string Name => "CSharpX Language Server";
        public IEnumerable<string> ConfigurationSections => null;
        public object InitializationOptions => null;
        public IEnumerable<string> FilesToWatch => null;
        public bool ShowNotificationOnInitializeFailed => true;
        public event AsyncEventHandler<EventArgs> StartAsync;
        public event AsyncEventHandler<EventArgs> StopAsync;

        public async Task<Connection> ActivateAsync(CancellationToken token)
        {
            // Set CSHARPX_LSP_PYTHON and CSHARPX_LSP_PATH for custom installations.
            string python = Environment.GetEnvironmentVariable("CSHARPX_LSP_PYTHON") ?? "python";
            string script = Environment.GetEnvironmentVariable("CSHARPX_LSP_PATH") ??
                Path.Combine(Path.GetDirectoryName(typeof(CSharpXLanguageClient).Assembly.Location), "server.py");
            if (!File.Exists(script)) throw new FileNotFoundException("CSharpX language server not found", script);
            var p = new Process();
            p.StartInfo = new ProcessStartInfo(python, "\"" + script + "\"") {
                RedirectStandardInput = true, RedirectStandardOutput = true,
                RedirectStandardError = true, UseShellExecute = false, CreateNoWindow = true
            };
            if (!p.Start()) throw new InvalidOperationException("Failed to launch CSharpX language server");
            return await Task.FromResult(new Connection(p.StandardOutput.BaseStream, p.StandardInput.BaseStream));
        }
        public Task OnLoadedAsync() => Task.CompletedTask;
        public Task OnServerInitializedAsync() => Task.CompletedTask;
        public Task OnServerInitializeFailedAsync(Exception e) => Task.CompletedTask;
        public Task<InitializationFailureContext> OnServerInitializeFailedAsync(ILanguageClientInitializationInfo info) => Task.FromResult<InitializationFailureContext>(null);
        public Task OnServerStoppedAsync() => Task.CompletedTask;
    }
}
