using System.ServiceProcess;

var services = ServiceController.GetServices();
foreach (var service in services)
{
    Console.WriteLine($"Service Name: {service.ServiceName}");
    Console.WriteLine($"Display Name: {service.DisplayName}");
    Console.WriteLine($"Status: {service.Status}");
    Console.WriteLine($"Start Type: {service.StartType}\n");
}
