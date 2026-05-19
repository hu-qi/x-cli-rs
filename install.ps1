$ErrorActionPreference = "Stop"

$Repo = if ($env:XCLI_RS_REPO) { $env:XCLI_RS_REPO } else { "hu-qi/x-cli-rs" }
$Version = if ($env:XCLI_RS_VERSION) { $env:XCLI_RS_VERSION } else { "latest" }
$InstallDir = if ($env:XCLI_RS_INSTALL_DIR) { $env:XCLI_RS_INSTALL_DIR } else { Join-Path $HOME ".local\bin" }
$Target = "x86_64-pc-windows-msvc"
$Archive = "x-cli-rs-$Target.zip"
$Checksum = "$Archive.sha256"
$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "x-cli-rs-install-$PID"
$Bins = @("x.exe", "chatgpt-image-cli.exe", "google-cli.exe", "baidu-cli.exe", "nanobanana-cli.exe", "xiaohongshu-cli.exe")

function Say($Message) {
  Write-Host $Message
}

function Fail($Message) {
  Write-Error $Message
  exit 1
}

if ($Version -eq "latest") {
  $BaseUrl = "https://github.com/$Repo/releases/latest/download"
} else {
  $BaseUrl = "https://github.com/$Repo/releases/download/$Version"
}

New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

try {
  Say "Installing x-cli-rs"
  Say "  repo:    $Repo"
  Say "  version: $Version"
  Say "  target:  $Target"
  Say "  dir:     $InstallDir"

  Say "Downloading $Archive"
  Invoke-WebRequest -Uri "$BaseUrl/$Archive" -OutFile (Join-Path $TmpDir $Archive)

  Say "Downloading $Checksum"
  Invoke-WebRequest -Uri "$BaseUrl/$Checksum" -OutFile (Join-Path $TmpDir $Checksum)

  $Expected = (Get-Content (Join-Path $TmpDir $Checksum) | Select-Object -First 1).Split(" ")[0].Trim().ToLowerInvariant()
  $Actual = (Get-FileHash -Algorithm SHA256 (Join-Path $TmpDir $Archive)).Hash.ToLowerInvariant()
  if ($Expected -ne $Actual) {
    Fail "checksum mismatch"
  }

  Expand-Archive -Force -Path (Join-Path $TmpDir $Archive) -DestinationPath (Join-Path $TmpDir "bin")

  foreach ($Bin in $Bins) {
    $Source = Join-Path (Join-Path $TmpDir "bin") $Bin
    if (!(Test-Path $Source)) {
      Fail "missing binary in archive: $Bin"
    }
    Copy-Item -Force $Source (Join-Path $InstallDir $Bin)
  }

  Say "Installed:"
  foreach ($Bin in $Bins) {
    Say "  $(Join-Path $InstallDir $Bin)"
  }
  Say ""
  Say "Make sure $InstallDir is on your PATH."
} finally {
  if (Test-Path $TmpDir) {
    Remove-Item -Recurse -Force $TmpDir
  }
}
