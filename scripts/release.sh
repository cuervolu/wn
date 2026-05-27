#!/usr/bin/env bash
# scripts/release.sh — Gestiona tags de release para WN++
# Uso:
#   ./scripts/release.sh create v0.2.0        # crea y pushea el tag
#   ./scripts/release.sh delete v0.2.0        # borra local y remoto
#   ./scripts/release.sh recreate v0.2.0      # borra y vuelve a crear (fix del CI ql)

set -e

BOLD="\033[1m"
GREEN="\033[32m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"

info()    { printf "${GREEN}[info]${RESET} %s\n" "$1"; }
warn()    { printf "${YELLOW}[warn]${RESET} %s\n" "$1"; }
error()   { printf "${RED}[error]${RESET} %s\n" "$1" >&2; exit 1; }
section() { printf "\n${BOLD}==> %s${RESET}\n" "$1"; }

check_tag_format() {
    local tag="$1"
    if ! echo "$tag" | grep -qE '^v[0-9]+\.[0-9]+\.[0-9]+'; then
        error "El tag '$tag' no tiene formato válido. Usa vX.Y.Z (ej: v0.2.0)"
    fi
}

check_clean_workdir() {
    if ! git diff --quiet || ! git diff --cached --quiet; then
        warn "Tienes cambios sin commitear:"
        git status --short
        printf "\n¿Continuar de todas formas? [s/N] "
        read -r respuesta
        if ! echo "$respuesta" | grep -qiE '^s$'; then
            error "Abortado. Commitea los cambios primero y luego vienes a wear"
        fi
    fi
}

check_remote_exists() {
    if ! git remote get-url origin > /dev/null 2>&1; then
        error "No hay remote 'origin' configurado papito!."
    fi
}

tag_exists_local() {
    git tag --list "$1" | grep -q "$1"
}

tag_exists_remote() {
    git ls-remote --tags origin "refs/tags/$1" | grep -q "$1"
}


cmd_create() {
    local tag="$1"

    check_tag_format "$tag"
    check_clean_workdir
    check_remote_exists

    section "Creando tag $tag"

    if tag_exists_local "$tag"; then
        error "El tag '$tag' ya existe localmente. Usa 'recreate' para reemplazarlo."
    fi

    if tag_exists_remote "$tag"; then
        error "El tag '$tag' ya existe en el remoto. Usa 'recreate' para reemplazarlo."
    fi

    # Muestra el commit sobre el que se creará el tag
    local commit
    commit=$(git rev-parse --short HEAD)
    local msg
    msg=$(git log -1 --format="%s" HEAD)
    info "Commit: $commit — $msg"

    git tag -a "$tag" -m "WN++ $tag"
    info "Tag creado localmente"

    git push origin main --follow-tags
    info "Tag pusheado a origin"

    section "¡Listo!"
    printf "  Tag ${BOLD}%s${RESET} creado y pusheado.\n" "$tag"
    printf "  Revisa el pipeline en: https://github.com/cuervolu/wn/actions\n\n"
}

cmd_delete() {
    local tag="$1"

    check_tag_format "$tag"
    check_remote_exists

    section "Borrando tag $tag"

    local deleted=0

    if tag_exists_local "$tag"; then
        git tag -d "$tag"
        info "Tag borrado localmente"
        deleted=1
    else
        warn "El tag '$tag' no existe localmente, saltando..."
    fi

    if tag_exists_remote "$tag"; then
        git push origin --delete "$tag"
        info "Tag borrado del remoto"
        deleted=1
    else
        warn "El tag '$tag' no existe en el remoto, saltando..."
    fi

    if [ $deleted -eq 0 ]; then
        warn "El tag '$tag' no existía en ningún lado."
    else
        section "¡Listo!"
        printf "  Tag ${BOLD}%s${RESET} eliminado.\n\n" "$tag"
        warn "Si había un Release en GitHub, bórralo manualmente:"
        printf "  https://github.com/cuervolu/wn/releases/tag/%s\n\n" "$tag"
    fi
}

cmd_recreate() {
    local tag="$1"

    check_tag_format "$tag"

    section "Recreando tag $tag"
    warn "Esto borrará el tag existente y lo recreará en HEAD."
    printf "¿Continuar? [s/N] "
    read -r respuesta
    if ! echo "$respuesta" | grep -qiE '^s$'; then
        error "Abortado."
    fi

    cmd_delete "$tag"
    cmd_create "$tag"
}

cmd_list() {
    section "Tags locales"
    git tag --sort=-version:refname | head -10

    section "Último tag"
    local last
    last=$(git describe --tags --abbrev=0 2>/dev/null || echo "ninguno")
    info "Último tag: $last"
}


usage() {
    printf '\n%sUso:%s\n' "$BOLD" "$RESET"
    printf '  %s <comando> [tag]\n\n' "$0"
    printf '%sComandos:%s\n' "$BOLD" "$RESET"
    printf '  create   <vX.Y.Z>   Crea y pushea el tag\n'
    printf '  delete   <vX.Y.Z>   Borra el tag local y remoto\n'
    printf '  recreate <vX.Y.Z>   Borra y vuelve a crear (para fixes de CI)\n'
    printf '  list                Muestra los últimos tags\n\n'
    printf '%sEjemplos:%s\n' "$BOLD" "$RESET"
    printf '  %s create v0.2.0\n' "$0"
    printf '  %s recreate v0.1.0   # después de un fix en CI\n' "$0"
    printf '  %s delete v0.1.0-beta\n\n' "$0"
}

COMMAND="${1:-}"
TAG="${2:-}"

case "$COMMAND" in
    create)
        [ -z "$TAG" ] && { usage; error "Falta el tag. Ej: $0 create v0.2.0"; }
        cmd_create "$TAG"
        ;;
    delete)
        [ -z "$TAG" ] && { usage; error "Falta el tag. Ej: $0 delete v0.1.0"; }
        cmd_delete "$TAG"
        ;;
    recreate)
        [ -z "$TAG" ] && { usage; error "Falta el tag. Ej: $0 recreate v0.1.0"; }
        cmd_recreate "$TAG"
        ;;
    list)
        cmd_list
        ;;
    *)
        usage
        exit 1
        ;;
esac
