
get_os_version() {
    if command -v lsb_release &> /dev/null; then
        echo "$(lsb_release -d | cut -f2)"
    else
        echo "MacOS $(sw_vers -productVersion)"
    fi
}

echo -e "OS: \n  $(get_os_version)"
echo -e "Solana CLI: \n  $(solana -V)"
echo -e "Anchor: \n  $(anchor --version)"
echo -e "Node: \n  $(node --version)"
echo -e "Rust: \n  $(rustc -V)"
# Cut the first line of the output
echo -e "build-sbf version: \n  $(cargo build-sbf --version | head -n 1)"

echo -e "\n\nPath: \n$(echo $PATH | tr ':' '\n' | sort)"
