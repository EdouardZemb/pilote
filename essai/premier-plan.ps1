# Un processus neuf, qui n'a envoye aucune frappe, tente de donner le premier plan a une fenetre.
# Comme un appel de pilote. Code 0 si la fenetre est au premier plan apres, 1 sinon.
param([string] $Methode, [long] $Fenetre)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Threading;
public static class P {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [StructLayout(LayoutKind.Explicit, Size = 40)] public struct MINPUT {
    [FieldOffset(0)] public uint type; [FieldOffset(8)] public int dx; [FieldOffset(12)] public int dy;
    [FieldOffset(16)] public uint mouseData; [FieldOffset(20)] public uint dwFlags; [FieldOffset(24)] public uint time; [FieldOffset(32)] public IntPtr extra; }
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, MINPUT[] inputs, int size);
  public static bool Essayer(string m, IntPtr h) {
    if (m == "souris") { MINPUT[] t = new MINPUT[1]; t[0].type = 0; t[0].dwFlags = 1; SendInput(1, t, Marshal.SizeOf(typeof(MINPUT))); }
    if (m == "alt-intercale") { keybd_event(0x12, 0, 0, UIntPtr.Zero); keybd_event(0x88, 0, 0, UIntPtr.Zero); keybd_event(0x88, 0, 2, UIntPtr.Zero); keybd_event(0x12, 0, 2, UIntPtr.Zero); }
    bool r = SetForegroundWindow(h);
    for (int i = 0; i < 10; i++) { if (GetForegroundWindow() == h) return true; Thread.Sleep(100); }
    return GetForegroundWindow() == h;
  }
}
'@
$ok = [P]::Essayer($Methode, [IntPtr] $Fenetre)
Write-Host ("SONDE frais-{0}={1}" -f $Methode, $ok)
if ($ok) { exit 0 } else { exit 1 }
