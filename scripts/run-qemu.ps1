$ErrorActionPreference = "Stop"

Set-Location -LiteralPath (Split-Path -Parent $PSScriptRoot)
cargo run --target x86_64-auraos.json
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
