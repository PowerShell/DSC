using System.ServiceProcess;
using System.Text.Json;
using System.Text.Json.Serialization;

if (args.Length < 2)
{
    Console.WriteLine("Usage: WindowsService.exe <action> <jsonInput>");
    Environment.Exit(1);
}

var operation = args[0].ToLowerInvariant();
var jsonInput = args[1];
WindowsService? windowsService = null;
var jsonSerializerOptions = new JsonSerializerOptions
{
    PropertyNamingPolicy = JsonNamingPolicy.CamelCase
};

if (!string.IsNullOrEmpty(jsonInput))
{
    try {
        windowsService = JsonSerializer.Deserialize<WindowsService>(jsonInput, jsonSerializerOptions);
    } catch (JsonException ex) {
        Console.Error.WriteLine($"Failed to deserialize JSON: {ex.Message}");
        Environment.Exit(1);
    }
}

switch (operation)
{
    case "get":
        if (windowsService == null || string.IsNullOrEmpty(windowsService.Name))
        {
            Console.Error.WriteLine("Property 'name' is required for 'get' operation.");
            Environment.Exit(1);
        }
        GetServices(windowsService.Name);
        break;
    case "set":
        // TODO: Left as an exercise for the reader.
        break;
    case "export":
        GetServices(null);
        break;
    default:
        Console.Error.WriteLine("Invalid action. Use get, set, or export.");
        Environment.Exit(1);
        break;
}

Environment.Exit(0);

void GetServices(string? name)
{
    var services = ServiceController.GetServices();
    var jsonSerializerOptions = new JsonSerializerOptions
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        Converters =
        {
            new JsonStringEnumConverter(JsonNamingPolicy.CamelCase)
        }
    };

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


        string json = JsonSerializer.Serialize(windowsService, jsonSerializerOptions);
        Console.WriteLine(json);
    }

    if (name is not null && !foundService)
    {
        Console.Error.WriteLine($"Service '{name}' not found.");
        Environment.Exit(1);
    }
}

class WindowsService
{
    public string? Name { get; set; }
    public string? DisplayName { get; set; }
    public ServiceControllerStatus Status { get; set; }
    public ServiceStartMode StartType { get; set; }
}
