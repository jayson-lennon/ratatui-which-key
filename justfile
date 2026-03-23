COPYRIGHT_NAME := "Jayson Lennon"
COPYRIGHT_YEAR := "2026"


clippy:
    cargo clippy --all-targets

semver:
    cargo semver-checks

test:
    cargo nextest run
    cargo test --doc

clean:
    rm -rfv .build/
    cargo clean -vv

coverage:
    cargo llvm-cov --lcov --output-path coverage.lcov

coverage-report:
    cargo llvm-cov report --html

debt: coverage
    debtmap analyze . --lcov coverage.lcov

apply-license:
    #!/bin/bash

    # --- CONFIGURATION ---
    NAME="{{COPYRIGHT_NAME}}"
    YEAR="{{COPYRIGHT_YEAR}}"
    # Add the extensions you want to target (space separated)
    EXTENSIONS=("rs")

    # The Header Template
    HEADER="Copyright (C) $YEAR $NAME

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as
    published by the Free Software Foundation, either version 3 of the
    License, or (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>."

    # Convert header to a commented block (using // for JS/CPP style)
    # If you use Python only, change '// ' to '# ' below.
    COMMENTED_HEADER=$(echo "$HEADER" | sed 's/^/\/\/ /')

    # --- EXECUTION ---
    for ext in "${EXTENSIONS[@]}"; do
        echo "Processing .$ext files..."

        # Find files with the extension, excluding node_modules or hidden git folders
        find . -type f -name "*.$ext" -not -path "*/.*" -not -path "*node_modules*" -not -path "./target/*" | while read -r file; do

            # Check if "Copyright" already exists in the first 5 lines
            if head -n 5 "$file" | grep -iq "Copyright"; then
                echo "  Skipping $file (Header already exists)"
            else
                echo "  Adding header to $file"
                # Create a temporary file with header + original content
                { echo "$COMMENTED_HEADER"; echo ""; cat "$file"; } > "$file.tmp" && mv "$file.tmp" "$file"
            fi
        done
    done

    echo "Done!"
