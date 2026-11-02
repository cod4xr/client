# Stop only the development client executable in the configured existing target.
$ErrorActionPreference = 'Stop'
$repositoryPath = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$metadata = cargo metadata --no-deps --format-version 1 --locked --manifest-path (Join-Path $repositoryPath 'Cargo.toml') | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw 'Cannot resolve the client Cargo target.' }
$targetPath = [IO.Path]::GetFullPath($metadata.target_directory)
$executables = @(
    [IO.Path]::GetFullPath((Join-Path $targetPath 'debug\cod4xr-client.exe')),
    [IO.Path]::GetFullPath((Join-Path $targetPath 'release\cod4xr-client.exe')),
    [IO.Path]::GetFullPath((Join-Path $targetPath 'debug\cod4xr-client-inspect.exe')),
    [IO.Path]::GetFullPath((Join-Path $targetPath 'release\cod4xr-client-inspect.exe'))
)
foreach ($candidate in @(Get-Process -Name @('cod4xr-client', 'cod4xr-client-inspect') -ErrorAction SilentlyContinue)) {
    if ($candidate.HasExited -or $candidate.Path -notin $executables) { continue }
    Stop-Process -InputObject $candidate -Force -ErrorAction Stop
    if (-not $candidate.WaitForExit(5000)) { throw 'Development client did not stop.' }
    Write-Output "Stopped development client PID $($candidate.Id)"
}
