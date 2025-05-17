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
WindowsService? windowsService = null;
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
            try {
                windowsService = JsonSerializer.Deserialize<WindowsService>(jsonInput);
            } catch (JsonException ex) {
                WriteError($"Failed to deserialize JSON: {ex.Message}");
                Environment.Exit(EXIT_INVALID_ARG);
            }
        }

        if (windowsService == null || string.IsNullOrEmpty(windowsService.Name))
        {
            WriteError("Property 'name' is required for 'get' operation.");
            Environment.Exit(EXIT_INVALID_ARG);
        }
        GetServices(windowsService.Name);
        break;
    case "schema":
        var jsonOptions = JsonSerializerOptions.Default;
        JsonNode schema = jsonOptions.GetJsonSchemaAsNode(typeof(WindowsService));
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
                windowsService = JsonSerializer.Deserialize<WindowsService>(jsonInput);
                WriteTrace($"Deserialized JSON: {JsonSerializer.Serialize(windowsService)}");
            } catch (JsonException ex) {
                WriteError($"Failed to deserialize JSON: {ex.Message}");
                Environment.Exit(EXIT_INVALID_ARG);
            }
        }
        if (windowsService == null || string.IsNullOrEmpty(windowsService.Name))
        {
            WriteError("Property 'name' is required for 'set' operation.");
            Environment.Exit(EXIT_INVALID_INPUT);
        }
        if (windowsService.Status is null)
        {
            WriteError("Property 'status' is required for 'set' operation.");
            Environment.Exit(EXIT_INVALID_INPUT);
        }
        if (windowsService.Exist is not null && windowsService.Exist == false)
        {
            WriteError($"Resource does not support removing a service.");
            Environment.Exit(EXIT_INVALID_INPUT);
        }
        if (windowsService.StartType is not null)
        {
            // TODO: Need to P/Invoke, use WMI, use sc.exe, use PowerShell, or write directly to registry to set the start type.
            WriteError($"Setting service start type is not supported.");
            Environment.Exit(EXIT_INVALID_INPUT);
        }
        if (windowsService.Status is not null)
        {
            WriteDebug($"Setting service status to {windowsService.Status}");
            var service = new ServiceController(windowsService.Name);
            try
            {
                if (windowsService.Status == ServiceControllerStatus.Running && service.Status != ServiceControllerStatus.Running)
                {
                    service.Start();
                }
                else if (windowsService.Status == ServiceControllerStatus.Stopped && service.Status != ServiceControllerStatus.Stopped)
                {
                    service.Stop();
                }
                service.WaitForStatus(windowsService.Status.Value);
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
        break;
    case "export":
        GetServices(null);
        break;
    default:
        WriteError("Invalid action. Use get, export, or schema.");
        Environment.Exit(EXIT_INVALID_ARG);
        break;
}

Environment.Exit(EXIT_OK);

void GetServices(string? name)
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

    string json = JsonSerializer.Serialize(windowsService);
    Console.WriteLine(json);
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
