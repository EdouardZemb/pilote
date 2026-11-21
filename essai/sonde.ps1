# Sonde jetable du runner Windows : bureau interactif, SendInput, premier plan, UIA, PrintWindow.
# Branche essai/runner, jamais fusionnee. Fichier en ASCII seul (Windows PowerShell 5.1).
$ErrorActionPreference = 'Continue'
$sortie = Join-Path $env:RUNNER_TEMP 'sonde'
New-Item -ItemType Directory -Force -Path $sortie | Out-Null
function Dire([string] $cle, $valeur) { Write-Host ("SONDE {0}={1}" -f $cle, $valeur) }

Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
public static class W {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] public static extern IntPtr OpenInputDesktop(uint flags, bool inherit, uint access);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern bool GetUserObjectInformationW(IntPtr h, int index, StringBuilder buf, int len, out int needed);
  [DllImport("user32.dll")] public static extern bool CloseDesktop(IntPtr h);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll", SetLastError = true)] public static extern uint SendInput(uint n, INPUT[] inputs, int size);
  [DllImport("user32.dll")] public static extern int GetSystemMetrics(int i);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowEx(IntPtr parent, IntPtr after, string cls, string title);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr SendMessage(IntPtr h, uint msg, IntPtr w, StringBuilder l);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr SendMessage(IntPtr h, uint msg, IntPtr w, string l);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Explicit, Size = 40)] public struct INPUT {
    [FieldOffset(0)] public uint type; [FieldOffset(8)] public ushort wVk; [FieldOffset(10)] public ushort wScan;
    [FieldOffset(12)] public uint dwFlags; [FieldOffset(16)] public uint time; [FieldOffset(24)] public IntPtr extra; }

  public static string Bureau() {
    IntPtr d = OpenInputDesktop(0, false, 1);
    if (d == IntPtr.Zero) return "OpenInputDesktop=echec:" + Marshal.GetLastWin32Error();
    StringBuilder b = new StringBuilder(256); int n;
    bool ok = GetUserObjectInformationW(d, 2, b, 512, out n);
    CloseDesktop(d);
    return ok ? b.ToString() : "nom-illisible";
  }
  public static bool Activer(IntPtr h) {
    if (IsIconic(h)) ShowWindow(h, 9);
    keybd_event(0x12, 0, 0, UIntPtr.Zero); keybd_event(0x12, 0, 2, UIntPtr.Zero);
    SetForegroundWindow(h);
    for (int i = 0; i < 10; i++) { if (GetForegroundWindow() == h) return true; Thread.Sleep(100); }
    return GetForegroundWindow() == h;
  }
  static bool Attendre(IntPtr h) {
    for (int i = 0; i < 10; i++) { if (GetForegroundWindow() == h) return true; Thread.Sleep(100); }
    return GetForegroundWindow() == h;
  }
  static void Touche(byte vk, bool haut) { keybd_event(vk, 0, haut ? 2u : 0u, UIntPtr.Zero); }
  public static bool Methode(string m, IntPtr h) {
    if (IsIconic(h)) ShowWindow(h, 9);
    if (m == "nu") { SetForegroundWindow(h); }
    else if (m == "alt") { Touche(0x12, false); Touche(0x12, true); SetForegroundWindow(h); }
    else if (m == "alt-entoure") { Touche(0x12, false); SetForegroundWindow(h); Touche(0x12, true); }
    else if (m == "alt-intercale") { Touche(0x12, false); Touche(0x88, false); Touche(0x88, true); Touche(0x12, true); SetForegroundWindow(h); }
    else if (m == "souris") {
      INPUT[] t = new INPUT[1]; t[0].type = 0; // INPUT_MOUSE, dx=dy=0, MOUSEEVENTF_MOVE
      t[0].dwFlags = 0; // champs de MOUSEINPUT a d'autres decalages : voir SourisVide
      SourisVide(); SetForegroundWindow(h);
    }
    return Attendre(h);
  }
  [StructLayout(LayoutKind.Explicit, Size = 40)] public struct MINPUT {
    [FieldOffset(0)] public uint type; [FieldOffset(8)] public int dx; [FieldOffset(12)] public int dy;
    [FieldOffset(16)] public uint mouseData; [FieldOffset(20)] public uint dwFlags; [FieldOffset(24)] public uint time; [FieldOffset(32)] public IntPtr extra; }
  [DllImport("user32.dll", SetLastError = true)] public static extern uint SendInput(uint n, MINPUT[] inputs, int size);
  public static uint SourisVide() {
    MINPUT[] t = new MINPUT[1]; t[0].type = 0; t[0].dwFlags = 1; // MOUSEEVENTF_MOVE, deplacement nul
    return SendInput(1, t, Marshal.SizeOf(typeof(MINPUT)));
  }
  public static string Texte(IntPtr e) { StringBuilder b = new StringBuilder(4096); SendMessage(e, 0x000D, (IntPtr)4096, b); return b.ToString(); }
  public static void Vider(IntPtr e) { SendMessage(e, 0x000C, IntPtr.Zero, ""); }
  public static uint Taper(string s) {
    INPUT[] t = new INPUT[s.Length * 2];
    for (int i = 0; i < s.Length; i++) {
      t[2 * i].type = 1; t[2 * i].wScan = s[i]; t[2 * i].dwFlags = 4;
      t[2 * i + 1].type = 1; t[2 * i + 1].wScan = s[i]; t[2 * i + 1].dwFlags = 6;
    }
    return SendInput((uint)t.Length, t, Marshal.SizeOf(typeof(INPUT)));
  }
  public static int Capturer(IntPtr h, string chemin) {
    RECT r; GetWindowRect(h, out r);
    int w = r.R - r.L, hh = r.B - r.T;
    if (w <= 0 || hh <= 0) return -1;
    using (Bitmap bmp = new Bitmap(w, hh, PixelFormat.Format32bppArgb)) {
      using (Graphics g = Graphics.FromImage(bmp)) { IntPtr dc = g.GetHdc(); PrintWindow(h, dc, 2); g.ReleaseHdc(dc); }
      bmp.Save(chemin, ImageFormat.Png);
      System.Collections.Generic.HashSet<int> c = new System.Collections.Generic.HashSet<int>();
      for (int y = 0; y < hh; y += 4) for (int x = 0; x < w; x += 4) c.Add(bmp.GetPixel(x, y).ToArgb());
      return c.Count;
    }
  }
}
'@

Dire 'os' ((Get-CimInstance Win32_OperatingSystem).Caption)
Dire 'version' ([Environment]::OSVersion.VersionString)
Dire 'UserInteractive' ([Environment]::UserInteractive)
Dire 'session' ((Get-Process -Id $PID).SessionId)
Dire 'utilisateur' (whoami)
Dire 'integrite' ((whoami /groups | Select-String 'Mandatory Label') -replace '\s+', ' ')
Dire 'bureau' ([W]::Bureau())
Dire 'ecran' ('{0}x{1}' -f [W]::GetSystemMetrics(0), [W]::GetSystemMetrics(1))
Dire 'ForegroundLockTimeout' ((Get-ItemProperty 'HKCU:\Control Panel\Desktop' -ErrorAction SilentlyContinue).ForegroundLockTimeout)
$wv = Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' -ErrorAction SilentlyContinue
Dire 'webview2' $(if ($wv) { $wv.pv } else { 'absent' })
$edge = @('C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe', 'C:\Program Files\Microsoft\Edge\Application\msedge.exe') | Where-Object { Test-Path $_ } | Select-Object -First 1
Dire 'edge' $(if ($edge) { (Get-Item $edge).VersionInfo.ProductVersion } else { 'absent' })
Dire 'avant-plan-initial' ([W]::GetForegroundWindow())

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$AE = [System.Windows.Automation.AutomationElement]
$TS = [System.Windows.Automation.TreeScope]

# --- Bloc-notes : premier plan, frappe Unicode, relecture UIA, capture
$p = Start-Process notepad -PassThru
$h = [IntPtr]::Zero
for ($i = 0; $i -lt 50 -and $h -eq [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 200; $p.Refresh(); $h = $p.MainWindowHandle }
if ($h -eq [IntPtr]::Zero) {
  $q = Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
  if ($q) { $h = $q.MainWindowHandle }
}
Dire 'blocnotes-fenetre' $h
if ($h -ne [IntPtr]::Zero) {
  Dire 'blocnotes-activer' ([W]::Activer($h))
  $texte = 'Bonjour ' + [char]0x00E9 + [char]0x00EA + ' /barre ' + [char]0x00FC
  Dire 'blocnotes-sendinput' ([W]::Taper($texte))
  Start-Sleep -Milliseconds 500
  $racine = $AE::FromHandle($h)
  $champ = $null
  foreach ($t in @([System.Windows.Automation.ControlType]::Edit, [System.Windows.Automation.ControlType]::Document)) {
    if (-not $champ) { $champ = $racine.FindFirst($TS::Descendants, (New-Object System.Windows.Automation.PropertyCondition($AE::ControlTypeProperty, $t))) }
  }
  $lu = 'champ-introuvable'
  if ($champ) {
    $vp = $null
    if ($champ.TryGetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern, [ref] $vp)) { $lu = $vp.Current.Value }
    else { $tp = $null; if ($champ.TryGetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern, [ref] $tp)) { $lu = $tp.DocumentRange.GetText(-1) } }
  }
  Dire 'blocnotes-relu-egal' ($lu -eq $texte)
  Dire 'blocnotes-relu-codes' ((($lu.ToCharArray() | ForEach-Object { [int] $_ }) -join ','))
  $f = $AE::FocusedElement
  Dire 'focus' ('{0} <{1}>' -f $f.Current.ControlType.ProgrammaticName, $f.Current.Name)
  Dire 'blocnotes-couleurs' ([W]::Capturer($h, (Join-Path $sortie 'blocnotes.png')))
}

# --- Edge en fenetre d'application : UIA de Chromium, capture PrintWindow
if ($edge) {
  $page = Join-Path $env:RUNNER_TEMP 'sonde.html'
  Set-Content -Path $page -Encoding ASCII -Value '<!doctype html><html lang="fr"><head><title>sonde-pilote</title></head><body style="background:#fff"><h1 style="color:#c00">Bonjour</h1><label>Nom <input id="nom" aria-keyshortcuts="/"></label><button>Valider</button><div aria-live="polite">zone</div></body></html>'
  $url = 'file:///' + ($page -replace '\\', '/')
  Start-Process $edge -ArgumentList @("--user-data-dir=$env:RUNNER_TEMP\edge", '--no-first-run', '--no-default-browser-check', "--app=$url") | Out-Null
  $he = [IntPtr]::Zero
  for ($i = 0; $i -lt 100 -and $he -eq [IntPtr]::Zero; $i++) {
    Start-Sleep -Milliseconds 200
    $q = Get-Process msedge -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -like '*sonde-pilote*' } | Select-Object -First 1
    if ($q) { $he = $q.MainWindowHandle }
  }
  Dire 'edge-fenetre' $he
  if ($he -ne [IntPtr]::Zero) {
    Start-Sleep -Seconds 2
    $re = $AE::FromHandle($he)
    $bouton = $null
    for ($i = 0; $i -lt 20 -and -not $bouton; $i++) {
      $bouton = $re.FindFirst($TS::Descendants, (New-Object System.Windows.Automation.AndCondition(
        (New-Object System.Windows.Automation.PropertyCondition($AE::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button)),
        (New-Object System.Windows.Automation.PropertyCondition($AE::NameProperty, 'Valider')))))
      if (-not $bouton) { Start-Sleep -Milliseconds 250 }
    }
    Dire 'edge-bouton-trouve' ([bool] $bouton)
    $champ = $re.FindFirst($TS::Descendants, (New-Object System.Windows.Automation.PropertyCondition($AE::AutomationIdProperty, 'nom')))
    if ($champ) { Dire 'edge-champ' ('<{0}> raccourci={1}' -f $champ.Current.Name, $champ.Current.AcceleratorKey) } else { Dire 'edge-champ' 'introuvable' }
    Dire 'edge-couleurs-arriere-plan' ([W]::Capturer($he, (Join-Path $sortie 'edge-arriere-plan.png')))
    Dire 'edge-activer' ([W]::Activer($he))
    if ($champ) { $champ.SetFocus(); Start-Sleep -Milliseconds 300; Dire 'edge-sendinput' ([W]::Taper('essai')); Start-Sleep -Milliseconds 300
      $vp = $null; if ($champ.TryGetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern, [ref] $vp)) { Dire 'edge-champ-valeur' $vp.Current.Value } }
    Dire 'edge-couleurs' ([W]::Capturer($he, (Join-Path $sortie 'edge.png')))
    if ($h -ne [IntPtr]::Zero) {
      Dire 'garde-avant-plan-est-edge' ([W]::GetForegroundWindow() -eq $he)
      Dire 'reactiver-blocnotes' ([W]::Activer($h))
    }
  }
}
# --- Le premier plan sans mode menu : chaque methode, depuis Edge au premier plan
if ($h -ne [IntPtr]::Zero -and $he -ne [IntPtr]::Zero) {
  $edit = [W]::FindWindowEx($h, [IntPtr]::Zero, 'Edit', $null)
  Dire 'blocnotes-edit' $edit
  foreach ($m in @('alt', 'nu', 'souris', 'alt-entoure', 'alt-intercale', 'souris', 'nu')) {
    $avant = [W]::Methode('alt-intercale', $he)
    [W]::Vider($edit)
    $ok = [W]::Methode($m, $h)
    $lu = ''
    if ($ok) { [void] [W]::Taper('Bonjour'); Start-Sleep -Milliseconds 400; $lu = [W]::Texte($edit) }
    Dire ("methode-$m") ('edge-devant={0} blocnotes-devant={1} lu=[{2}]' -f $avant, $ok, $lu)
  }
  # Chromium : le champ garde-t-il le focus apres chaque methode ?
  if ($champ) {
    foreach ($m in @('alt', 'souris', 'alt-intercale')) {
      [void] [W]::Methode('souris', $h)
      $vp = $null; [void] $champ.TryGetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern, [ref] $vp)
      $ok = [W]::Methode($m, $he)
      $champ.SetFocus(); Start-Sleep -Milliseconds 200
      [void] [W]::Methode('souris', $h)
      $ok = [W]::Methode($m, $he)
      $avantValeur = $vp.Current.Value
      if ($ok) { [void] [W]::Taper('xyz'); Start-Sleep -Milliseconds 400 }
      Dire ("edge-$m") ('devant={0} avant=[{1}] apres=[{2}]' -f $ok, $avantValeur, $vp.Current.Value)
    }
  }
}
Get-Process notepad, msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Dire 'fin' 'ok'
