#!/bin/bash
# Source: https://gitlab.gnome.org/GNOME/fractal/-/blob/ee6791518094c9c72a0a9630e334f73264f2aff9/scripts/checks.sh
# Slightly customized to make it easier to reuse for other projects.

resources="data/de.haeckerfelix.Shortwave.gresource.xml.in"
ui_files="data/gtk"

export LC_ALL=C

# Usage info
show_help() {
cat << EOF
Run conformity checks on the current Rust project.

If a dependency is not found, helps the user to install it.

USAGE: ${0##*/} [OPTIONS]

OPTIONS:
    -f, --force-install     Install missing dependencies without asking
    -v, --verbose           Use verbose output
    -h, --help              Display this help and exit

ERROR CODES:
    1                       Check failed
    2                       Missing dependency
EOF
}

# Style helpers
act="\e[1;32m"
err="\e[1;31m"
pos="\e[32m"
neg="\e[31m"
res="\e[0m"

# Common styled strings
Installing="${act}Installing${res}"
Checking="  ${act}Checking${res}"
Failed="    ${err}Failed${res}"
error="${err}error:${res}"
invalid="${neg}Invalid input${res}"
ok="${pos}ok${res}"
fail="${neg}fail${res}"

# Initialize variables
force_install=0
verbose=0

# Helper functions
# Sort to_sort in natural order.
sort() {
    local size=${#to_sort[@]}
    local swapped=0;

    for (( i = 0; i < $size-1; i++ ))
    do
        swapped=0
        for ((j = 0; j < $size-1-$i; j++ ))
        do
            if [[ "${to_sort[$j]}" > "${to_sort[$j+1]}" ]]
            then
                temp="${to_sort[$j]}";
                to_sort[$j]="${to_sort[$j+1]}";
                to_sort[$j+1]="$temp";
                swapped=1;
            fi
        done

        if [[ $swapped -eq 0 ]]; then
            break;
        fi
    done
}

# Remove common entries in to_diff1 and to_diff2.
diff() {
    for i in ${!to_diff1[@]}; do
        for j in ${!to_diff2[@]}; do
            if [[ "${to_diff1[$i]}" == "${to_diff2[$j]}" ]]; then
                unset to_diff1[$i]
                unset to_diff2[$j]
                break
            fi
        done
    done
}

# Check if rustup is available.
# Argument:
#   '-i' to install if missing.
check_rustup() {
    if ! which rustup &> /dev/null; then
        if [[ "$1" == '-i' ]]; then
            echo -e "$Installing rustup…"
            curl https://sh.rustup.rs -sSf  | sh -s -- -y --default-toolchain nightly
            export PATH=$PATH:$HOME/.cargo/bin
            if ! which rustup &> /dev/null; then
                echo -e "$Failed to install rustup"
                exit 2
            fi
        else
            exit 2
        fi
    fi
}

# Install cargo via rustup.
install_cargo() {
    check_rustup -i
    if ! which cargo >/dev/null 2>&1; then
        echo -e "$Failed to install cargo"
        exit 2
    fi
}

# Check if cargo is available. If not, ask to install it.
check_cargo() {
    if ! which cargo >/dev/null 2>&1; then
        echo "Unable to find cargo for pre-commit checks"

        if [[ $force_install -eq 1 ]]; then
            install_cargo
        elif [ ! -t 1 ]; then
            exit 2
        elif check_rustup; then
            echo -e "$error rustup is installed but the cargo command isn't available"
            exit 2
        else
            echo ""
            echo "y: Install cargo via rustup"
            echo "N: Don't install cargo and abort checks"
            echo ""
            while true; do
                echo -n "Install cargo? [y/N]: "; read yn < /dev/tty
                case $yn in
                    [Yy]* )
                        install_cargo
                        break
                        ;;
                    [Nn]* | "" )
                        exit 2
                        ;;
                    * )
                        echo $invalid
                        ;;
                esac
            done
        fi
    fi

    if [[ $verbose -eq 1 ]]; then
        echo ""
        rustc -Vv && cargo +nightly -Vv
    fi
}

# Install rustfmt with rustup.
install_rustfmt() {
    check_rustup -i

    echo -e "$Installing rustfmt…"
    rustup component add --toolchain nightly rustfmt
    if ! cargo +nightly fmt --version >/dev/null 2>&1; then
        echo -e "$Failed to install rustfmt"
        exit 2
    fi
}

# Run rustfmt to enforce code style.
run_rustfmt() {
    if ! cargo +nightly fmt --version >/dev/null 2>&1; then
        if [[ $force_install -eq 1 ]]; then
            install_rustfmt
        elif [ ! -t 1 ]; then
            echo "Unable to check code style, because rustfmt could not be run"
            exit 2
        else
            echo "Rustfmt is needed to check code style, but it isn’t available"
            echo ""
            echo "y: Install rustfmt via rustup"
            echo "N: Don't install rustfmt and abort checks"
            echo ""
            while true; do
                echo -n "Install rustfmt? [y/N]: "; read yn < /dev/tty
                case $yn in
                    [Yy]* )
                        install_rustfmt
                        break
                        ;;
                    [Nn]* | "" )
                        exit 2
                        ;;
                    * )
                        echo $invalid
                        ;;
                esac
            done
        fi
    fi

    echo -e "$Checking code style…"

    if [[ $verbose -eq 1 ]]; then
        echo ""
        cargo +nightly fmt --version
        echo ""
    fi

    if ! cargo +nightly fmt --all -- --check; then
        echo -e "  Checking code style result: $fail"
        echo "Please fix the above issues, either manually or by running: cargo fmt --all"
        exit 1
    else
        echo -e "  Checking code style result: $ok"
    fi
}

# Install cargo-fmt with cargo.
install_cargo_deny() {
    echo -e "$Installing cargo-deny…"
    cargo install cargo-deny
    if ! cargo-deny --version >/dev/null 2>&1; then
        echo -e "$Failed to install cargo-deny"
        exit 2
    fi
}

# Run cargo-fmt to lint dependencies.
run_cargo_deny() {
    if ! cargo-deny --version >/dev/null 2>&1; then
        if [[ $force_install -eq 1 ]]; then
            install_cargo_deny
        elif [ ! -t 1 ]; then
            echo "Unable to lint dependencies, because cargo-deny could not be run"
            exit 2
        else
            echo "Cargo-deny is needed to lint dependencies, but it isn’t available"
            echo ""
            echo "y: Install cargo-deny via cargo"
            echo "N: Don't install cargo-deny and abort checks"
            echo ""
            while true; do
                echo -n "Install cargo-deny? [y/N]: "; read yn < /dev/tty
                case $yn in
                    [Yy]* )
                        install_cargo_deny
                        break
                        ;;
                    [Nn]* | "" )
                        exit 2
                        ;;
                    * )
                        echo $invalid
                        ;;
                esac
            done
        fi
    fi

    echo -e "$Checking dependencies…"

    if [[ $verbose -eq 1 ]]; then
        echo ""
        cargo-deny --version
        echo ""
    fi

    echo -n "  "

    if ! cargo-deny --log-level error check; then
        echo -e "  Checking dependencies: $fail"
        echo "Please fix the above issues"
        exit 1
    fi
}

# Install typos with cargo.
install_typos() {
    echo -e "$Installing typos…"
    cargo install typos-cli
    if ! typos --version >/dev/null 2>&1; then
        echo -e "$Failed to install typos"
        exit 2
    fi
}

# Run typos to check for spelling mistakes.
run_typos() {
    if ! typos --version >/dev/null 2>&1; then
        if [[ $force_install -eq 1 ]]; then
            install_typos
        elif [ ! -t 1 ]; then
            echo "Unable to check spelling mistakes, because typos could not be run"
            exit 2
        else
            echo "Typos is needed to check spelling mistakes, but it isn’t available"
            echo ""
            echo "y: Install typos via cargo"
            echo "N: Don't install typos and abort checks"
            echo ""
            while true; do
                echo -n "Install typos? [y/N]: "; read yn < /dev/tty
                case $yn in
                    [Yy]* )
                        install_typos
                        break
                        ;;
                    [Nn]* | "" )
                        exit 2
                        ;;
                    * )
                        echo $invalid
                        ;;
                esac
            done
        fi
    fi

    echo -e "$Checking spelling mistakes…"

    if [[ $verbose -eq 1 ]]; then
        echo ""
        typos --version
        echo ""
    fi

    if ! typos --color always; then
        echo -e "  Checking spelling mistakes result: $fail"
        echo "Please fix the above issues, either manually or by running: typos -w"
        exit 1
    else
        echo -e "  Checking spelling mistakes result: $ok"
    fi
}

# Check if files in POTFILES.in are correct.
#
# This checks, in that order:
#   - All files exist
#   - All files with translatable strings are present and only those
#   - Files are sorted alphabetically
#
# This assumes the following:
#   - POTFILES is located at 'po/POTFILES.in'
#   - UI (Glade) files are located in $ui_files and use 'translatable="yes"'
#   - Rust files are located in 'src' and use '*gettext' methods or macros
check_potfiles() {
    echo -e "$Checking po/POTFILES.in…"

    local ret=0

    # Check that files in POTFILES exist.
    while read -r line; do
        if [[ -n $line &&  ${line::1} != '#' ]]; then
            if [[ ! -f $line ]]; then
                echo -e "$error File '$line' in POTFILES.in does not exist"
                ret=1
            fi
            if [[ ${line:(-3):3} == '.ui' ]]; then
                ui_potfiles+=($line)
            elif [[ ${line:(-3):3} == '.rs' ]]; then
                rs_potfiles+=($line)
            fi
        fi
    done < po/POTFILES.in

    if [[ ret -eq 1 ]]; then
        echo -e "  Checking po/POTFILES.in result: $fail"
        echo "Please fix the above issues"
        exit 1
    fi

    # Get UI files with 'translatable="yes"'.
    ui_files=(`grep -lIr 'translatable="yes"' $ui_files/*`)

    # Get Rust files with regex 'gettext(_f)?\(', except `src/i18n.rs`.
    rs_files=(`grep -lIrE 'gettext(_f)?\(' --exclude=i18n.rs src/*`)

    # Get Rust files with regex '.*i18n.*\(', except `src/i18n.rs`.
    rs_files=(`grep -lIrE '.*i18n.*\(' --exclude=i18n.rs src/*`)

    # Get Rust files with macros, regex 'gettext!\('.
    rs_macro_files=(`grep -lIrE 'gettext!\(' src/*`)

    # Remove common files
    to_diff1=("${ui_potfiles[@]}")
    to_diff2=("${ui_files[@]}")
    diff
    ui_potfiles=("${to_diff1[@]}")
    ui_files=("${to_diff2[@]}")

    to_diff1=("${rs_potfiles[@]}")
    to_diff2=("${rs_files[@]}")
    diff
    rs_potfiles=("${to_diff1[@]}")
    rs_files=("${to_diff2[@]}")

    potfiles_count=$((${#ui_potfiles[@]} + ${#rs_potfiles[@]}))
    if [[ $potfiles_count -eq 1 ]]; then
        echo ""
        echo -e "$error Found 1 file in POTFILES.in without translatable strings:"
        ret=1
    elif [[ $potfiles_count -ne 0 ]]; then
        echo ""
        echo -e "$error Found $potfiles_count files in POTFILES.in without translatable strings:"
        ret=1
    fi
    for file in ${ui_potfiles[@]}; do
        echo $file
    done
    for file in ${rs_potfiles[@]}; do
        echo $file
    done

    let files_count=$((${#ui_files[@]} + ${#rs_files[@]}))
    if [[ $files_count -eq 1 ]]; then
        echo ""
        echo -e "$error Found 1 file with translatable strings not present in POTFILES.in:"
        ret=1
    elif [[ $files_count -ne 0 ]]; then
        echo ""
        echo -e "$error Found $files_count files with translatable strings not present in POTFILES.in:"
        ret=1
    fi
    for file in ${ui_files[@]}; do
        echo $file
    done
    for file in ${rs_files[@]}; do
        echo $file
    done

    let rs_macro_count=$((${#rs_macro_files[@]}))
    if [[ $rs_macro_count -eq 1 ]]; then
        echo ""
        echo -e "$error Found 1 Rust file that uses a gettext-rs macro, use the corresponding i18n method instead:"
        ret=1
    elif [[ $rs_macro_count -ne 0 ]]; then
        echo ""
        echo -e "$error Found $rs_macro_count Rust files that use a gettext-rs macro, use the corresponding i18n method instead:"
        ret=1
    fi
    for file in ${rs_macro_files[@]}; do
        echo $file
    done

    if [[ ret -eq 1 ]]; then
        echo ""
        echo -e "  Checking po/POTFILES.in result: $fail"
        echo "Please fix the above issues"
        exit 1
    fi

    # Check sorted alphabetically
    to_sort=("${potfiles[@]}")
    sort
    for i in ${!potfiles[@]}; do
        if [[ "${potfiles[$i]}" != "${to_sort[$i]}" ]]; then
            echo -e "$error Found file '${potfiles[$i]}' before '${to_sort[$i]}' in POTFILES.in"
            ret=1
            break
        fi
    done

    if [[ ret -eq 1 ]]; then
        echo ""
        echo -e "  Checking po/POTFILES.in result: $fail"
        echo "Please fix the above issues"
        exit 1
    else
        echo -e "  Checking po/POTFILES.in result: $ok"
    fi
}

# Check if files in gresource xml are sorted alphabetically.
check_resources() {
    echo -e "$Checking $resources…"

    local ret=0

    # Get files.
    regex="<file .*>(.*)</file>"
    while read -r line; do
        if [[ $line =~ $regex ]]; then
            files+=("${BASH_REMATCH[1]}")
        fi
    done < $resources

    # Check sorted alphabetically
    to_sort=("${files[@]}")
    sort
    for i in ${!files[@]}; do
        if [[ "${files[$i]}" != "${to_sort[$i]}" ]]; then
            echo -e "$error Found file '${files[$i]#src/}' before '${to_sort[$i]#src/}' in gresource xml"
            ret=1
            break
        fi
    done

    if [[ ret -eq 1 ]]; then
        echo ""
        echo -e "  Checking $resources result: $fail"
        echo "Please fix the above issues"
        exit 1
    else
        echo -e "  Checking $resources result: $ok"
    fi
}

# Check arguments
while [[ "$1" ]]; do case $1 in
    -f | --force-install )
        force_install=1
        ;;
    -v | --verbose )
        verbose=1
        ;;
    -h | --help )
        show_help
        exit 0
        ;;
    *)
        show_help >&2
        exit 1
esac; shift; done

# Run
check_cargo
echo ""
run_rustfmt
echo ""
run_cargo_deny
echo ""
run_typos
echo ""
check_potfiles
echo ""
check_resources
echo ""
