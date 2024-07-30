 # Reg keys are documented here: https://learn.microsoft.com/en-us/mem/configmgr/core/servers/deploy/install/list-of-prerequisite-checks#pending-system-restart
 if (Get-ChildItem "HKLM:\Software\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending" -EA Ignore) { return @{ rebootPending = $true } | ConvertTo-Json -Compress }
 if (Get-Item "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired" -EA Ignore) { return @{ rebootPending = $true } | ConvertTo-Json -Compress }
 if (Get-ItemProperty "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager" -Name PendingFileRenameOperations -EA Ignore) { return @{ rebootPending = $true } | ConvertTo-Json -Compress }
 try { 
   $util = [wmiclass]"\\.\root\ccm\clientsdk:CCM_ClientUtilities"
   $status = $util.DetermineIfRebootPending()
   if(($status -ne $null) -and $status.RebootPending){
     return @{ rebootPending = $true } | ConvertTo-Json -Compress
   }
 }catch{}
 
return @{ rebootPending = $false } | ConvertTo-Json -Compress