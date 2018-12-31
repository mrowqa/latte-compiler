if [[ "$1" == "--opt" ]]; then
    grep -riE "todo|fixme" . --include "*.rs" --include "*.lalrpop"
elif [[ "$1" == "--no-ext" ]]; then
    grep -riE "todo|fixme" . --include "*.rs" --include "*.lalrpop" | grep -v "\(optional\)" | grep -v "\(ext\)"
else
    grep -riE "todo|fixme" . --include "*.rs" --include "*.lalrpop" | grep -v "\(optional\)"
fi

