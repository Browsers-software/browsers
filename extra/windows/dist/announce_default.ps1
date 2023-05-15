
# Announce that there is potential new browser and let user choose it if its a first installation
# https://learn.microsoft.com/en-us/windows/win32/shell/default-programs#registeredapplications

$code = @'
[System.Runtime.InteropServices.DllImport("Shell32.dll")]
private static extern int SHChangeNotify(int eventId, int flags, IntPtr item1, IntPtr item2);

// SHCNE_ASSOCCHANGED =  0x08000000
// SHCNF_DWORD = 0x0003
// SHCNF_FLUSH = 0x1000
public static void Refresh()  {
    SHChangeNotify(
        0x08000000,
        0x0003 | 0x1000,
        IntPtr.Zero,
        IntPtr.Zero
    );
    SHChangeNotify(0x8000000, 0x1000, IntPtr.Zero, IntPtr.Zero);
}
'@

Add-Type -MemberDefinition $code -Namespace WinAPI -Name Defaults
[WinAPI.Defaults]::Refresh()

# Maybe required, based on docs
Start-Sleep -Seconds 1