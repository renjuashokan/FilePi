using System.Text.Json;
using FilePiWeb;
using FilePiWeb.Interfaces;
using FilePiWeb.Services;
using Microsoft.AspNetCore.Components.Web;
using Microsoft.AspNetCore.Components.WebAssembly.Hosting;
using Refit;
using Serilog;
using Syncfusion.Blazor;
using Syncfusion.Licensing;

// Configure Serilog for WebAssembly (browser console only)
Log.Logger = new LoggerConfiguration()
    .MinimumLevel.Debug()
    //.MinimumLevel.Override("Microsoft", LogEventLevel.Information)
    .Enrich.FromLogContext()
    .CreateLogger();

try
{
    Log.Information("Starting FilePi WebAssembly Application");

    var builder = WebAssemblyHostBuilder.CreateDefault(args);
    var syncfusionKey = builder.Configuration["SyncfusionLicenseKey"];
    if (!string.IsNullOrEmpty(syncfusionKey))
    {
        SyncfusionLicenseProvider.RegisterLicense(syncfusionKey);
        Log.Information("Syncfusion license registered successfully");
    }
    else
    {
        Log.Warning("Syncfusion license key not found in configuration");
    }

    // Add Serilog to the DI container
    builder.Services.AddLogging(loggingBuilder =>
        loggingBuilder.AddSerilog(dispose: true));

    builder.RootComponents.Add<App>("#app");
    builder.RootComponents.Add<HeadOutlet>("head::after");

    builder.Services.AddSyncfusionBlazor();

    // Configure Refit with proper JSON settings
    var refitSettings = new RefitSettings
    {
        ContentSerializer = new SystemTextJsonContentSerializer(
            new JsonSerializerOptions
            {
                PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower, PropertyNameCaseInsensitive = true
            })
    };

    // Get port from environment variable or use default with error handling
    string? portString = Environment.GetEnvironmentVariable("FILE_PI_PORT");
    int port = 8080; // default port

    if (!string.IsNullOrEmpty(portString) && int.TryParse(portString, out int parsedPort))
    {
        port = parsedPort;
    }

    // Build the API base URL using UriBuilder
    var baseAddress = new Uri(builder.HostEnvironment.BaseAddress);
    Uri apiBaseUrl = new UriBuilder { Scheme = baseAddress.Scheme, Host = baseAddress.Host, Port = port }.Uri;

    Log.Information("API Base URL: {ApiBaseUrl}", apiBaseUrl);

    // Register Refit client with dynamic port
    builder.Services.AddRefitClient<IFilePiApi>(refitSettings)
        .ConfigureHttpClient(c => c.BaseAddress = apiBaseUrl);

    builder.Services.AddSingleton<IFileService, FileService>();

    await builder.Build().RunAsync();
}
catch (Exception ex)
{
    Log.Fatal(ex, "Application terminated unexpectedly");
}
finally
{
    Log.CloseAndFlush();
}
