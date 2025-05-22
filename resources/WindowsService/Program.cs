using System.ServiceProcess;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.Json.Schema;
using System.Text.Json.Serialization;
using System.Text.Json.Serialization.Metadata;

const int EXIT_OK = 0;
const int EXIT_INVALID_ARG = 1;
const int EXIT_INVALID_INPUT = 2;

if (args.Length < 1)
{
    WriteError("Usage: WindowsService.exe <action>");
    Environment.Exit(EXIT_INVALID_ARG);
}

var operation = args[0].ToLowerInvariant();
WindowsServices? windowsServices = null;
WriteDebug( $"Operation: {operation}");

switch (operation)
{
    case "get":
        if (args.Length < 3 || args[1].ToLowerInvariant() != "-input")
        {
            WriteError("Usage: WindowsService.exe get -input <jsonInput>");
            Environment.Exit(EXIT_INVALID_ARG);
        }
        var jsonInput = args[2];
        WriteTrace($"Input JSON: {jsonInput}");
        if (!string.IsNullOrEmpty(jsonInput))
        {
            try
            {
                windowsServices = JsonSerializer.Deserialize<WindowsServices>(jsonInput);
            }
            catch (JsonException ex)
            {
                WriteError($"Failed to deserialize JSON: {ex.Message}");
                Environment.Exit(EXIT_INVALID_ARG);
            }
        }

        WindowsServices resultServices = new WindowsServices();
        foreach (var service in windowsServices?.Services ?? Enumerable.Empty<WindowsService>())
        {
            if (service.Name is null)
            {
                WriteError("Property 'name' is required for 'get' operation.");
                Environment.Exit(EXIT_INVALID_ARG);
            }

            var resultService = GetService(service.Name);
            if (resultService is not null)
            {
                resultServices.Services.Add(resultService);
            }
        }

        var options = new JsonSerializerOptions
        {
            DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        };
        string json = JsonSerializer.Serialize(resultServices, options);
        Console.WriteLine(json);

        break;
    case "schema":
        var jsonOptions = JsonSerializerOptions.Default;
        JsonNode schema = jsonOptions.GetJsonSchemaAsNode(typeof(WindowsServices));
        Console.WriteLine(schema.ToString());
        break;
    case "set":
        if (args.Length < 3 || args[1].ToLowerInvariant() != "-input")
        {
            WriteError("Usage: WindowsService.exe set -input <jsonInput>");
            Environment.Exit(EXIT_INVALID_ARG);
        }
        jsonInput = args[2];
        WriteTrace($"Input JSON: {jsonInput}");
        if (!string.IsNullOrEmpty(jsonInput))
        {
            try {
                windowsServices = JsonSerializer.Deserialize<WindowsServices>(jsonInput);
                WriteTrace($"Deserialized JSON: {JsonSerializer.Serialize(windowsServices)}");
            } catch (JsonException ex) {
                WriteError($"Failed to deserialize JSON: {ex.Message}");
                Environment.Exit(EXIT_INVALID_ARG);
            }
        }
        foreach (var service in windowsServices?.Services ?? Enumerable.Empty<WindowsService>())
        {
            if (string.IsNullOrEmpty(service.Name))
            {
                WriteError("Property 'name' is required for 'set' operation.");
                Environment.Exit(EXIT_INVALID_INPUT);
            }
            if (service.Status is null)
            {
                WriteError("Property 'status' is required for 'set' operation.");
                Environment.Exit(EXIT_INVALID_INPUT);
            }
            if (service.Exist is not null && service.Exist == false)
            {
                WriteError($"Resource does not support removing a service.");
                Environment.Exit(EXIT_INVALID_INPUT);
            }
            if (service.StartType is not null)
            {
                // TODO: Need to P/Invoke, use WMI, use sc.exe, use PowerShell, or write directly to registry to set the start type.
                WriteError($"Setting service start type is not supported.");
                Environment.Exit(EXIT_INVALID_INPUT);
            }
            if (service.Status is not null)
            {
                WriteDebug($"Setting service status to {service.Status}");
                var serviceController = new ServiceController(service.Name);
                try
                {
                    if (service.Status == ServiceControllerStatus.Running && serviceController.Status != ServiceControllerStatus.Running)
                    {
                        serviceController.Start();
                    }
                    else if (service.Status == ServiceControllerStatus.Stopped && serviceController.Status != ServiceControllerStatus.Stopped)
                    {
                        serviceController.Stop();
                    }
                    serviceController.WaitForStatus(service.Status.Value);
                }
                catch (Exception ex)
                {
                    WriteError($"Failed to set service status: {ex.Message}");
                    if (ex.InnerException != null)
                    {
                        WriteError($"Inner exception: {ex.InnerException.Message}");
                    }
                    Environment.Exit(EXIT_INVALID_ARG);
                }
            }
        }
        break;
    case "export":
        WindowsServices services = new WindowsServices();
        var allServices = ServiceController.GetServices();
        foreach (var service in allServices)
        {
            var windowsService = new WindowsService
            {
                Name = service.ServiceName,
                DisplayName = service.DisplayName,
                Status = service.Status,
                StartType = service.StartType
            };
            services.Services.Add(windowsService);
        }
        var exportOptions = new JsonSerializerOptions
        {
            DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        };
        string exportJson = JsonSerializer.Serialize(services, exportOptions);
        Console.WriteLine(exportJson);
        break;
    default:
        WriteError("Invalid action. Use get, export, or schema.");
        Environment.Exit(EXIT_INVALID_ARG);
        break;
}

Environment.Exit(EXIT_OK);

WindowsService? GetService(string? name)
{
    var services = ServiceController.GetServices();
    WindowsService? windowsService = null;
    foreach (var service in services)
    {
        if (name is not null && service.ServiceName is not null && !service.ServiceName.Equals(name, StringComparison.OrdinalIgnoreCase))
        {
            continue;
        }

        windowsService = new WindowsService
        {
            Name = service.ServiceName,
            DisplayName = service.DisplayName,
            Status = service.Status,
            StartType = service.StartType
        };
    }

    if (name is not null && windowsService is null)
    {
        windowsService = new WindowsService
        {
            Name = name,
            Exist = false
        };
    }

    return windowsService;
}

void WriteError(string message)
{
    var errorMessage = new ErrorMessage { Error = message };
    string json = JsonSerializer.Serialize(errorMessage);
    Console.Error.WriteLine(json);
}

void WriteDebug(string message)
{
    var debugMessage = new DebugMessage { Debug = message };
    string json = JsonSerializer.Serialize(debugMessage);
    Console.Error.WriteLine(json);
}

void WriteTrace(string message)
{
    var traceMessage = new TraceMessage { Trace = message };
    string json = JsonSerializer.Serialize(traceMessage);
    Console.Error.WriteLine(json);
}

record ErrorMessage
{
    [JsonPropertyName("error")]
    public string Error { get; set; } = string.Empty;
}

record DebugMessage
{
    [JsonPropertyName("debug")]
    public string Debug { get; set; } = string.Empty;
}

record TraceMessage
{
    [JsonPropertyName("trace")]
    public string Trace { get; set; } = string.Empty;
}

record WindowsServices
{
    [JsonPropertyName("services")]
    public List<WindowsService> Services { get; set; } = new List<WindowsService>();
}

record WindowsService
{
    [JsonPropertyName("name")]
    public string? Name { get; set; }
    [JsonPropertyName("displayName")]
    public string? DisplayName { get; set; }
    [JsonConverter(typeof(JsonStringEnumConverter))]
    [JsonPropertyName("status")]
    public ServiceControllerStatus? Status { get; set; }
    [JsonConverter(typeof(JsonStringEnumConverter))]
    [JsonPropertyName("startType")]
    public ServiceStartMode? StartType { get; set; }
    [JsonPropertyName("_exist")]
    public bool? Exist { get; set; }
}
