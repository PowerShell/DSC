using System.ServiceProcess;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.Json.Schema;
using System.Text.Json.Serialization;
using System.Text.Json.Serialization.Metadata;

if (args.Length < 1)
{
    Console.WriteLine("Usage: WindowsService.exe <action>");
    Environment.Exit(1);
}

var operation = args[0].ToLowerInvariant();
WindowsService? windowsService = null;

switch (operation)
{
    case "get":
        if (args.Length < 3 || args[1].ToLowerInvariant() != "-input")
        {
            Console.Error.WriteLine("Usage: WindowsService.exe get -input <jsonInput>");
            Environment.Exit(1);
        }
        var jsonInput = args[2];
        if (!string.IsNullOrEmpty(jsonInput))
        {
            try {
                windowsService = JsonSerializer.Deserialize<WindowsService>(jsonInput);
            } catch (JsonException ex) {
                Console.Error.WriteLine($"Failed to deserialize JSON: {ex.Message}");
                Environment.Exit(1);
            }
        }

        if (windowsService == null || string.IsNullOrEmpty(windowsService.Name))
        {
            Console.Error.WriteLine("Property 'name' is required for 'get' operation.");
            Environment.Exit(1);
        }
        GetServices(windowsService.Name);
        break;
    case "schema":
        var jsonOptions = JsonSerializerOptions.Default;
        JsonNode schema = jsonOptions.GetJsonSchemaAsNode(typeof(WindowsService));
        Console.WriteLine(schema.ToString());
        break;
    case "set":
        // TODO: Left as an exercise for the reader.
        break;
    case "export":
        GetServices(null);
        break;
    default:
        Console.Error.WriteLine("Invalid action. Use get, export, or schema.");
        Environment.Exit(1);
        break;
}

Environment.Exit(0);

void GetServices(string? name)
{
    var services = ServiceController.GetServices();
    var foundService = false;
    foreach (var service in services)
    {
        if (name is not null && service.ServiceName is not null && !service.ServiceName.Equals(name, StringComparison.OrdinalIgnoreCase))
        {
            continue;
        }

        foundService = true;

        var windowsService = new WindowsService
        {
            Name = service.ServiceName,
            DisplayName = service.DisplayName,
            Status = service.Status,
            StartType = service.StartType
        };


        string json = JsonSerializer.Serialize(windowsService);
        Console.WriteLine(json);
    }

    if (name is not null && !foundService)
    {
        Console.Error.WriteLine($"Service '{name}' not found.");
        Environment.Exit(1);
    }
}


record WindowsService
{
    [JsonPropertyName("name")]
    public string? Name { get; set; }
    [JsonPropertyName("displayName")]
    public string? DisplayName { get; set; }
    [JsonConverter(typeof(JsonStringEnumConverter))]
    [JsonPropertyName("status")]
    public ServiceControllerStatus Status { get; set; }
    [JsonConverter(typeof(JsonStringEnumConverter))]
    [JsonPropertyName("startType")]
    public ServiceStartMode StartType { get; set; }
}
