param(
    [string]$PythonPath = "python",
    [string]$ConfigPath = "$(Resolve-Path ../config.yaml)",
    [string]$ServiceName = "RecallPipeline"
)

$binPath = "`"$PythonPath`" -m recall_pipeline.cli run --config `"$ConfigPath`""

if (Get-Service -Name $ServiceName -ErrorAction SilentlyContinue) {
    Stop-Service -Name $ServiceName -ErrorAction SilentlyContinue
    sc.exe delete $ServiceName | Out-Null
}

New-Service -Name $ServiceName -BinaryPathName $binPath -Description "Recall pipeline background service" -StartupType Automatic
Write-Host "Installed service $ServiceName"
