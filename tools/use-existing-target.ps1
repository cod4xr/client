# Reuse one existing operator-owned Cargo target. Never create a replacement cache.
param([Parameter(Mandatory = $true)][string]$TargetPath)
$ErrorActionPreference = 'Stop'
$repositoryPath = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$resolvedTarget = (Resolve-Path -LiteralPath $TargetPath -ErrorAction Stop).Path
if (-not (Test-Path -LiteralPath $resolvedTarget -PathType Container)) {
    throw 'The shared Cargo target must already be a directory.'
}
$configurationPath = Join-Path $repositoryPath '.cargo'
[IO.Directory]::CreateDirectory($configurationPath) | Out-Null
$escapedTarget = $resolvedTarget.Replace('\', '/')
$encodedTarget = ConvertTo-Json -InputObject $escapedTarget -Compress
$configuration = "[build]`ntarget-dir = $encodedTarget`n"
$targetConfiguration = Join-Path $configurationPath 'config.toml'
if (Test-Path -LiteralPath $targetConfiguration) {
    $currentConfiguration = [IO.File]::ReadAllText($targetConfiguration).Replace("`r`n", "`n")
    $previousGenerated = "[build]`ntarget-dir = '$escapedTarget'`n"
    if ($currentConfiguration -ne $configuration -and $currentConfiguration -ne $previousGenerated) {
        throw 'Existing Cargo configuration has operator edits. Set build.target-dir manually; it was preserved.'
    }
}
[IO.File]::WriteAllText($targetConfiguration, $configuration, [Text.UTF8Encoding]::new($false))
Write-Output "Using existing Cargo target: $resolvedTarget"
