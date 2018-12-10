if [[ "$1" == "--opt" ]]; then
    grep -riE "todo|fixme" . --include "*.rs" --include "*.lalrpop"
else
    grep -riE "todo|fixme" . --include "*.rs" --include "*.lalrpop" | grep -v "\(optional\)"
fi

