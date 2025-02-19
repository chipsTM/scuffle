#!/bin/bash

parse_cargo_updates() {
    unchanged_packages=()
    added_packages=()
    removed_packages=()
    updated_packages=()

    while IFS= read -r line; do
        line=$(echo "$line" | sed -E 's/\x1B\[[0-9;]*[mK]//g')

        if [[ $line =~ Updating ]]; then
            package=$(echo "$line" | awk '{print $2}')
            old_version=$(echo "$line" | awk '{print $3}' | tr -d 'v')
            new_version=$(echo "$line" | awk '{print $5}' | tr -d 'v')
            updated_packages+=("| [$package](https://crates.io/crates/$package) | $old_version | $new_version |")
        elif [[ $line =~ Adding ]]; then
            package=$(echo "$line" | awk '{print $2}')
            new_version=$(echo "$line" | awk '{print $3}' | tr -d 'v')
            added_packages+=("| [$package](https://crates.io/crates/$package) | $new_version |")
        elif [[ $line =~ Removing ]]; then
            package=$(echo "$line" | awk '{print $2}')
            old_version=$(echo "$line" | awk '{print $3}' | tr -d 'v')
            removed_packages+=("| [$package](https://crates.io/crates/$package) | $old_version |")
        elif [[ $line =~ Unchanged ]]; then
            package=$(echo "$line" | awk '{print $2}')
            old_version=$(echo "$line" | awk '{print $3}' | tr -d 'v')
            new_version=$(echo "$line" | awk -F '[()]' '{print $2}' | awk '{print $2}' | tr -d 'v')
            unchanged_packages+=("| [$package](https://crates.io/crates/$package) | $old_version | $new_version |")
        fi
    done

    if [[ ${#added_packages[@]} -gt 0 ]]; then
        echo "<details><summary>New packages (${#added_packages[@]})</summary>"
        echo
        echo "| Package | New Version |"
        echo "|---------|------------|"
        for pkg in "${added_packages[@]}"; do
            echo "$pkg"
        done
        echo
        echo "</details>"
        echo
    fi

    if [[ ${#updated_packages[@]} -gt 0 ]]; then
        echo "<details><summary>Updated packages (${#updated_packages[@]})</summary>"
        echo
        echo "| Package | Old Version | New Version |"
        echo "|---------|------------|------------|"
        for pkg in "${updated_packages[@]}"; do
            echo "$pkg"
        done
        echo
        echo "</details>"
        echo
    fi

    if [[ ${#removed_packages[@]} -gt 0 ]]; then
        echo "<details><summary>Removed packages (${#removed_packages[@]})</summary>"
        echo
        echo "| Package | Old Version |"
        echo "|---------|------------|"
        for pkg in "${removed_packages[@]}"; do
            echo "$pkg"
        done
        echo
        echo "</details>"
        echo
    fi

    if [[ ${#unchanged_packages[@]} -gt 0 ]]; then
        echo "<details><summary>Unchanged packages (${#unchanged_packages[@]})</summary>"
        echo
        echo "| Package | Old Version | New Version |"
        echo "|---------|------------|------------|"
        for pkg in "${unchanged_packages[@]}"; do
            echo "$pkg"
        done
        echo
        echo "</details>"
        echo
    fi

    if [[ ${#unchanged_packages[@]} -eq 0 && ${#added_packages[@]} -eq 0 && ${#updated_packages[@]} -eq 0 && ${#removed_packages[@]} -eq 0 ]]; then
        echo "No packages to update"
    fi
}

# Read input from stdin and process
parse_cargo_updates
