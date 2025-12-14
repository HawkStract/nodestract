Get-ChildItem -Path "src" -Recurse -Include *.rs | ForEach-Object {
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host "FILE: $($_.FullName)" -ForegroundColor Yellow
    Write-Host "==========================================" -ForegroundColor Cyan
    Get-Content $_.FullName
    Write-Host "`n`n"
}