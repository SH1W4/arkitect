# Caminhos
$src = "$PSScriptRoot\prototype_tmp"
$dest = "$PSScriptRoot"

# Função de cópia inteligente
function Copy-FilesRecursively($source, $destination) {
    Get-ChildItem -Path $source -Recurse | ForEach-Object {
        $targetPath = $_.FullName.Replace($source, $destination)
        if ($_.PSIsContainer) {
            if (!(Test-Path -Path $targetPath)) {
                New-Item -ItemType Directory -Path $targetPath | Out-Null
            }
        } else {
            Copy-Item $_.FullName -Destination $targetPath -Force
        }
    }
}

# Execução
Write-Host "🚀 Iniciando fusão simbiótica com ARKITECT_PROTOTYPE..."
Copy-FilesRecursively -source $src -destination $dest
Write-Host "✅ Integração completa! Seus arquivos foram mesclados com sucesso."

# Opcional: deletar o diretório temporário após a fusão
# Remove-Item -Recurse -Force $src
