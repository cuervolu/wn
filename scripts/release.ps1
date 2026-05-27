<#
.SYNOPSIS
    Gestiona tags de release para WN++.

.DESCRIPTION
    Script equivalente a scripts/release.sh para Windows.
    Permite crear, borrar y recrear tags de Git con validaciones básicas,
    y pushearlos al remoto 'origin'.

    Requiere Git instalado y disponible en el PATH.

.PARAMETER Command
    Acción a ejecutar: create | delete | recreate | list

.PARAMETER Tag
    Tag en formato vX.Y.Z (ej: v0.2.0). Requerido para create, delete y recreate.

.EXAMPLE
    .\release.ps1 create v0.2.0

.EXAMPLE
    .\release.ps1 delete v0.1.0-beta

.EXAMPLE
    .\release.ps1 recreate v0.1.0

.EXAMPLE
    .\release.ps1 list

.NOTES
    Autor: Cuervolu
    Repo:  https://github.com/cuervolu/wn
#>

[CmdletBinding()]
param (
    [Parameter(Position = 0)]
    [ValidateSet('create', 'delete', 'recreate', 'list')]
    [string]$Command,

    [Parameter(Position = 1)]
    [string]$Tag
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'


function Write-Info    ([string]$msg) { Write-Host "[info]  $msg" -ForegroundColor Green }
function Write-Warn    ([string]$msg) { Write-Host "[warn]  $msg" -ForegroundColor Yellow }
function Write-Section ([string]$msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Write-Fatal   ([string]$msg) {
    Write-Host "[error] $msg" -ForegroundColor Red
    exit 1
}

function Invoke-Git {
    <#
    .SYNOPSIS
        Ejecuta un comando git y devuelve su salida como string.
        Lanza un error si git retorna código distinto de 0.
    #>
    param([string[]]$Args)

    $output = & git @Args 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "git $($Args -join ' ') falló con código $LASTEXITCODE`n$output"
    }
    return $output
}

function Test-TagExistsLocal ([string]$tag) {
    $result = & git tag --list $tag 2>&1
    return ($LASTEXITCODE -eq 0) -and ($result -match [regex]::Escape($tag))
}

function Test-TagExistsRemote ([string]$tag) {
    $result = & git ls-remote --tags origin "refs/tags/$tag" 2>&1
    return ($LASTEXITCODE -eq 0) -and ($result -match [regex]::Escape($tag))
}


function Assert-TagFormat ([string]$tag) {
    if ($tag -notmatch '^v\d+\.\d+\.\d+') {
        Write-Fatal "El tag '$tag' no tiene formato válido. Usa vX.Y.Z (ej: v0.2.0)"
    }
}

function Assert-CleanWorkdir {
    $dirty = & git status --porcelain 2>&1
    if ($dirty) {
        Write-Warn "Tienes cambios sin commitear:"
        & git status --short
        $resp = Read-Host "`n¿Continuar de todas formas? [s/N]"
        if ($resp -notmatch '^[sS]$') {
            Write-Fatal "Abortado. Commitea los cambios primero y luego vienes a wear"
        }
    }
}

function Assert-RemoteExists {
    $null = & git remote get-url origin 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Fatal "No hay remote 'origin' configurado papito!"
    }
}

function Invoke-Create ([string]$tag) {
    Assert-TagFormat $tag
    Assert-CleanWorkdir
    Assert-RemoteExists

    Write-Section "Creando tag $tag"

    if (Test-TagExistsLocal $tag) {
        Write-Fatal "El tag '$tag' ya existe localmente. Usa 'recreate' para reemplazarlo."
    }
    if (Test-TagExistsRemote $tag) {
        Write-Fatal "El tag '$tag' ya existe en el remoto. Usa 'recreate' para reemplazarlo."
    }

    $commit = (Invoke-Git 'rev-parse', '--short', 'HEAD').Trim()
    $msg    = (Invoke-Git 'log', '-1', '--format=%s', 'HEAD').Trim()
    Write-Info "Commit: $commit — $msg"

    Invoke-Git 'tag', '-a', $tag, '-m', "WN++ $tag" | Out-Null
    Write-Info "Tag creado localmente"

    Invoke-Git 'push', 'origin', 'main', '--follow-tags' | Out-Null
    Write-Info "Tag pusheado a origin"

    Write-Section "¡Listo!"
    Write-Host "  Tag " -NoNewline
    Write-Host $tag -ForegroundColor White -NoNewline
    Write-Host " creado y pusheado."
    Write-Host "  Revisa el pipeline en: https://github.com/cuervolu/wn/actions`n"
}

function Invoke-Delete ([string]$tag) {
    Assert-TagFormat $tag
    Assert-RemoteExists

    Write-Section "Borrando tag $tag"

    $deleted = $false

    if (Test-TagExistsLocal $tag) {
        Invoke-Git 'tag', '-d', $tag | Out-Null
        Write-Info "Tag borrado localmente"
        $deleted = $true
    } else {
        Write-Warn "El tag '$tag' no existe localmente, saltando..."
    }

    if (Test-TagExistsRemote $tag) {
        Invoke-Git 'push', 'origin', '--delete', $tag | Out-Null
        Write-Info "Tag borrado del remoto"
        $deleted = $true
    } else {
        Write-Warn "El tag '$tag' no existe en el remoto, saltando..."
    }

    if (-not $deleted) {
        Write-Warn "El tag '$tag' no existía en ningún lado."
    } else {
        Write-Section "¡Listo!"
        Write-Host "  Tag " -NoNewline
        Write-Host $tag -ForegroundColor White -NoNewline
        Write-Host " eliminado."
        Write-Warn "Si había un Release en GitHub, bórralo manualmente:"
        Write-Host "  https://github.com/cuervolu/wn/releases/tag/$tag`n"
    }
}

function Invoke-Recreate ([string]$tag) {
    Assert-TagFormat $tag

    Write-Section "Recreando tag $tag"
    Write-Warn "Esto borrará el tag existente y lo recreará en HEAD."

    $resp = Read-Host "¿Continuar? [s/N]"
    if ($resp -notmatch '^[sS]$') {
        Write-Fatal "Abortado."
    }

    Invoke-Delete $tag
    Invoke-Create $tag
}

function Invoke-List {
    Write-Section "Tags locales (últimos 10)"
    & git tag --sort=-version:refname | Select-Object -First 10

    Write-Section "Último tag"
    $last = & git describe --tags --abbrev=0 2>$null
    if ($LASTEXITCODE -ne 0 -or -not $last) { $last = "ninguno" }
    Write-Info "Último tag: $last"
}

function Show-Usage {
    Write-Host @"

Uso:
  .\release.ps1 <comando> [tag]

Comandos:
  create   <vX.Y.Z>   Crea y pushea el tag
  delete   <vX.Y.Z>   Borra el tag local y remoto
  recreate <vX.Y.Z>   Borra y vuelve a crear (para fixes de CI)
  list                Muestra los últimos tags

Ejemplos:
  .\release.ps1 create v0.2.0
  .\release.ps1 recreate v0.1.0    # después de un fix en CI
  .\release.ps1 delete v0.1.0-beta

"@
}

if (-not $Command) {
    Show-Usage
    exit 1
}

switch ($Command) {
    'create'   {
        if (-not $Tag) { Show-Usage; Write-Fatal "Falta el tag. Ej: .\release.ps1 create v0.2.0" }
        Invoke-Create $Tag
    }
    'delete'   {
        if (-not $Tag) { Show-Usage; Write-Fatal "Falta el tag. Ej: .\release.ps1 delete v0.1.0" }
        Invoke-Delete $Tag
    }
    'recreate' {
        if (-not $Tag) { Show-Usage; Write-Fatal "Falta el tag. Ej: .\release.ps1 recreate v0.1.0" }
        Invoke-Recreate $Tag
    }
    'list'     { Invoke-List }
}