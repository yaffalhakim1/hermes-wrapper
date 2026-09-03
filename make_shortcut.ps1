$s = New-Object -ComObject WScript.Shell
$sc = $s.CreateShortcut('C:/Users/yafit/AppData/Roaming/Microsoft/Windows/Start Menu/Programs/Hermes Wrapper.lnk')
$sc.TargetPath = 'C:/Users/yafit/AppData/Local/Programs/HermesWrapper/hermes-wrapper.exe'
$sc.WorkingDirectory = 'C:/Users/yafit/AppData/Local/Programs/HermesWrapper'
$sc.Description = 'Hermes Wrapper'
$sc.Save()
Write-Host 'shortcut created'
