#!/usr/bin/env bash

set -e

PIOLA_REPO="cuervolu/piola"
PIOLA_BIN_DIR="${PIOLA_HOME:-$HOME/.piola}/bin"
PIOLA_VERSION="${PIOLA_VERSION:-latest}"

if [ -t 1 ]; then
    BOLD="$(printf '\033[1m')"
    GREEN="$(printf '\033[32m')"
    YELLOW="$(printf '\033[33m')"
    RED="$(printf '\033[31m')"
    RESET="$(printf '\033[0m')"
else
    BOLD="" GREEN="" YELLOW="" RED="" RESET=""
fi

info()    { printf "%s[info]%s %s\n"    "$GREEN"  "$RESET" "$1"; }
warn()    { printf "%s[advertencia]%s %s\n" "$YELLOW" "$RESET" "$1"; }
error()   { printf "%s[error]%s %s\n"   "$RED"    "$RESET" "$1" >&2; exit 1; }
section() { printf "\n%s==> %s%s\n"     "$BOLD"   "$1" "$RESET"; }

detect_target() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)
            case "$arch" in
                x86_64)  echo "x86_64-unknown-linux-gnu" ;;
                aarch64) echo "aarch64-unknown-linux-gnu" ;;
                armv7l)  echo "armv7-unknown-linux-gnueabihf" ;;
                *)       error "Arquitectura Linux no soportada: $arch" ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                x86_64) echo "x86_64-apple-darwin" ;;
                arm64)  echo "aarch64-apple-darwin" ;;   # ← M1/M2/M3
                *)      error "Arquitectura macOS no soportada: $arch" ;;
            esac
            ;;
        *)
            error "Sistema operativo no soportado: $os. Usa Windows con install.ps1"
            ;;
    esac
}

check_dependencies() {
    for cmd in curl tar; do
        if ! command -v "$cmd" > /dev/null 2>&1; then
            error "Necesitas '$cmd' instalado para continuar."
        fi
    done
}

get_latest_version() {
    if ! command -v curl > /dev/null 2>&1; then
        error "curl no encontrado"
    fi

    local url="https://api.github.com/repos/${PIOLA_REPO}/releases/latest"
    local version

    version=$(curl -fsSL "$url" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

    if [ -z "$version" ]; then
        error "No se pudo obtener la versión más reciente de GitHub. ¿Existe un release?"
    fi

    echo "$version"
}


install_piola() {
    local target version archive_url tmp_dir

    section "Detectando plataforma"
    target="$(detect_target)"
    info "Target: $target"

    section "Obteniendo versión"
    if [ "$PIOLA_VERSION" = "latest" ]; then
        version="$(get_latest_version)"
    else
        version="$PIOLA_VERSION"
    fi
    info "Versión: $version"

    archive_url="https://github.com/${PIOLA_REPO}/releases/download/${version}/piola-${version}-${target}.tar.gz"

    section "Descargando Piola $version"
    info "Desde: $archive_url"

    tmp_dir="$(mktemp -d)"
    # shellcheck disable=SC2064
    trap "rm -rf '$tmp_dir'" EXIT

    if ! curl -fsSL --progress-bar "$archive_url" -o "$tmp_dir/piola.tar.gz"; then
        error "No se pudo descargar $archive_url\n  ¿Existe este release para $target?"
    fi

    section "Instalando"
    tar -xzf "$tmp_dir/piola.tar.gz" -C "$tmp_dir"
    mkdir -p "$PIOLA_BIN_DIR"
    mv "$tmp_dir/piola" "$PIOLA_BIN_DIR/piola"
    chmod +x "$PIOLA_BIN_DIR/piola"
    info "Binario instalado en: $PIOLA_BIN_DIR/piola"

    section "Configurando PATH"
    configure_path
}

configure_path() {
    local shell_config export_line

    export_line="export PATH=\"\$PATH:$PIOLA_BIN_DIR\""

    case "${SHELL:-}" in
        */zsh)  shell_config="$HOME/.zshrc" ;;
        */bash) shell_config="$HOME/.bashrc" ;;
        */fish) shell_config="$HOME/.config/fish/config.fish"
                export_line="fish_add_path $PIOLA_BIN_DIR" ;;
        *)      shell_config="$HOME/.profile" ;;
    esac

    if ! grep -qF "$PIOLA_BIN_DIR" "$shell_config" 2>/dev/null; then
        printf "\n# Piola\n%s\n" "$export_line" >> "$shell_config"
        info "PATH actualizado en $shell_config"
    else
        info "PATH ya configurado en $shell_config"
    fi

    warn "Reinicia tu terminal o ejecuta: source $shell_config"
}

verify_installation() {
    if "$PIOLA_BIN_DIR/piola" --version > /dev/null 2>&1; then
        section "¡Listo!"
        printf "\n  %sPiola instalado exitosamente.%s\n\n" "$GREEN$BOLD" "$RESET"
        printf "  Ejecuta %spiola%s para abrir el REPL\n"     "$BOLD" "$RESET"
        printf "  o %spiola programa.cl%s para ejecutar un archivo\n\n" "$BOLD" "$RESET"
    else
        warn "El binario se instaló pero no respondió a --version."
        warn "Puede que necesites reiniciar tu terminal."
    fi
}

main() {
    printf "\n%sBienvenido al instalador de Piola%s\n" "$BOLD" "$RESET"

    check_dependencies
    install_piola
    verify_installation
}

main "$@"