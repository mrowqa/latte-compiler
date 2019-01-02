if [[ "$1" == "--opt" ]]; then
    grep -riE "todo|fixme" . --include "*.rs" --include "*.lalrpop" --include README
elif [[ "$1" == "--no-ext" ]]; then
    grep -riE "todo|fixme" . --include "*.rs" --include "*.lalrpop" --include README | grep -v "\(optional\)" | grep -v "\(ext\)"
else
    grep -riE "todo|fixme" . --include "*.rs" --include "*.lalrpop" --include README | grep -v "\(optional\)"
fi

