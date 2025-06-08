# Reg keys are documented here: https://learn.microsoft.com/en-us/mem/configmgr/core/servers/deploy/install/list-of-prerequisite-checks#pending-system-restart
$reasons = @()
if (Get-ChildItem "HKLM:\Software\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending" -EA Ignore) { 
  $reasons += "Component Based Servicing"
}
if (Get-Item "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired" -EA Ignore) { 
  $reasons += "Windows Update"
}
if (Get-ItemProperty "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager" -Name PendingFileRenameOperations -EA Ignore) { 
  $reasons += "Pending File Rename Operations"
}
try { 
  $util = [wmiclass]"\\.\root\ccm\clientsdk:CCM_ClientUtilities"
  $status = $util.DetermineIfRebootPending()
  if(($null -ne $status) -and $status.RebootPending){
    $reasons += "SCCM Client"
  }
}catch{}

$result = @{
  rebootPending = $reasons.Count -gt 0
  reason = if ($reasons.Count -gt 0) { $reasons } else { $null }
}
return $result | ConvertTo-Json -Compress