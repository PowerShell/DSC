using System.ServiceProcess;
using System.Text.Json;
using System.Text.Json.Serialization;

var services = ServiceController.GetServices();
var jsonSerializerOptions = new JsonSerializerOptions
{
    PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
    Converters =
    {
        new JsonStringEnumConverter(JsonNamingPolicy.CamelCase)
    }
};

foreach (var service in services)
{
    var windowsService = new WindowsService
    {
        ServiceName = service.ServiceName,
        DisplayName = service.DisplayName,
        Status = service.Status,
        StartType = service.StartType
    };


    string json = JsonSerializer.Serialize(windowsService, jsonSerializerOptions);
    Console.WriteLine(json);
}

class WindowsService
{
    public string ServiceName { get; set; }
    public string DisplayName { get; set; }
    public ServiceControllerStatus Status { get; set; }
    public ServiceStartMode StartType { get; set; }
}
